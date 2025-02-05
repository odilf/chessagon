//! Utilities for fuller game implementations, that go beyond the rules of chessagon.
//!
//! See [`Game`].

mod tests;
mod time_control;

use crate::{Color, board::Board, mov::Move, piece::MoveError};
use jiff::Timestamp;
use std::{fmt, time::Duration};
pub use time_control::TimeControl;

/// A game of chessagon.
///
/// This type includes management of:
/// - Current [`Board`] state
/// - [`Move`] history
/// - Time controls ([`TimeControl`])
/// - Whether the game has finished ([`GameResult`])
///
// TODO: Document when exactly the timing is timed with (i.e., the authority)
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Game {
    /// The current state of the board
    board: Board,

    /// The time control for this game. To see when moves where played, use [`Self::moves`]
    time_control: TimeControl,

    /// The move history, given as a pair of the move itself and the instant it was played on.
    moves: Vec<(Move, Timestamp)>,

    /// The result of a game, if it has concluded.
    result: Option<GameResult>,

    /// Whether a draw has been offered, and by who.
    draw_offer: Option<Color>,
}

/// A possible action a player can take in a game.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Action {
    /// Make a move.
    Move(Move),

    /// Concede the win to the opponent.
    Resign,

    /// Make a draw offer, which if the opponent accepts via [`Self::AcceptDraw`] makes the game end in a draw.
    /// Retractable with [`Self::RetractDraw`].
    ///
    /// If the opponent had already offered a draw, their offer gets deleted the moment you offer a draw.
    OfferDraw,

    /// Retract a draw offered by [`Self::OfferDraw`].
    RetractDraw,

    /// Accept a draw offered by opponent with [`Self::OfferDraw`].
    AcceptDraw,
}

impl Game {
    /// Creates a new game from the default position with the given [`TimeControl`].
    ///
    /// See also [`Self::from_position`].
    pub fn new(time_control: TimeControl) -> Self {
        Self::from_position(Board::default(), time_control)
    }

    /// Creates a new game from a specific [`Board`] state with [`TimeControl`].
    ///
    /// See also [`Self::new`]
    pub fn from_position(board: Board, time_control: TimeControl) -> Self {
        Self {
            board,
            time_control,
            moves: Vec::new(),
            result: None,
            draw_offer: None,
        }
    }

    /// The color of the player that has to make a move
    pub fn turn(&self) -> Color {
        if self.moves.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    /// The current board state of the game
    pub fn board(&self) -> &Board {
        &self.board
    }

    /// The [`GameResult`] of the game, if it has concluded.
    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    /// The move history from a player's side.
    pub fn moves_from(&self, color: Color) -> impl Iterator<Item = &(Move, Timestamp)> {
        self.moves.iter().skip(color as usize).step_by(2)
    }

    /// How long it took or is taking to play the `i`th move of the game.
    ///
    /// Note that this is the `i`th move in general, for both colors. In other words,
    /// if `i` is even, it is a white move; if `i` is odd, it is a black move.
    ///
    /// If `i` corresponds to the last move, it will return the amount of time it has
    /// taken so far. This value will increase in subsequent invocations.
    ///
    /// If `i` corresponds to the first move of each color, the return value will be
    /// [`Duration::ZERO`], since first moves don't spend time (but if the move hasn't
    /// been played yet, it will return [`None`]).
    ///
    /// Returns [`None`] if the move hasn't occurred yet.
    pub fn move_duration(&self, i: usize) -> Option<Duration> {
        if i < 2 {
            return (i < self.moves.len()).then_some(Duration::ZERO);
        }

        let (_, start) = self.moves.get(i - 1)?;
        let end = self
            .moves
            .get(i)
            .map(|(_, end)| *end)
            .unwrap_or_else(|| Timestamp::now());

        Some(end.duration_since(*start).unsigned_abs())
    }

    /// The amount of time the player of the given color has to make a move when it's their turn.
    ///
    /// Returns [`Duration::ZERO`] if the player has ran out of time.
    // TODO: This keeps the timer running after resignations.
    pub fn time_remaining(&self, color: Color) -> Duration {
        let mut i = color as usize;
        let mut time_remaining = self.time_control.base_time[color];
        while let Some(move_duration) = self.move_duration(i) {
            // This is in two separate lines because otherwise you get an underflow error if `move_duration` is greater than increment.
            if time_remaining < move_duration {
                return Duration::ZERO;
            }
            time_remaining -= move_duration;
            time_remaining += self.time_control.increment[color];
            i += 2;
        }

        time_remaining
    }

    /// The winner of the game, if it has concluded. It is a nested option because it:
    /// - returns `None` if the game hasn't finished.
    /// - returns `Some(None)` if the game resulted in a draw
    /// - returns `Some(color)` if `color` has won the game
    pub fn winner(&self) -> Option<Option<Color>> {
        self.result.map(|result| match result {
            GameResult::Win { winner, .. } => Some(winner),
            GameResult::Draw { .. } => None,
        })
    }

    /// Whether the game has finished.
    #[inline]
    pub fn is_finished(&self) -> bool {
        self.winner().is_some()
    }

    /// Make a resignation from the player with the given color.
    ///
    /// Makes the player of the opposite color win.
    #[inline]
    pub fn resign(&mut self, color: Color) {
        self.result = Some(GameResult::Win {
            winner: color.other(),
            reason: WinReason::Resignation,
        });
    }

    /// Offer a draw from the player of the given color
    #[inline]
    // TODO: These should be specific errors that are `#[from]` in `ApplyActionError`
    pub fn offer_draw(&mut self, color: Color) -> Result<(), ApplyActionError> {
        // TODO: Add a "draw already offered" error
        self.draw_offer = Some(color);
        Ok(())
    }

    #[inline]
    // TODO: These should be specific errors that are `#[from]` in `ApplyActionError`
    pub fn retract_draw(&mut self, color: Color) -> Result<(), ApplyActionError> {
        // TODO: Add a "no offered draw to retract" error
        let Some(offered_by) = self.draw_offer else {
            return Ok(());
        };

        if offered_by != color {
            // TODO: Add a "not your draw offer" error
            return Ok(());
        }

        self.draw_offer = None;
        Ok(())
    }

    /// Tries to apply an [`Action`] from the given [`Color`].
    ///
    /// Returns an [`ApplyActionError`] if the specified move is not possible.
    pub fn apply_action(&mut self, action: Action, color: Color) -> Result<(), ApplyActionError> {
        if color != self.turn() {
            return Err(ApplyActionError::NotYourTurn);
        }
        match action {
            Action::Move(mov) => {
                if self.is_finished() {
                    return Err(ApplyActionError::GameIsFinished);
                }
                let now = Timestamp::now();
                self.moves.push((mov, now));
                self.board.apply_move(mov, color)?;

                if self.board.possible_moves(color.other()).next().is_none() {
                    if self.board.in_check(color.other()).is_some() {
                        self.result = Some(GameResult::Win {
                            winner: color,
                            reason: WinReason::Checkmate,
                        })
                    } else {
                        self.result = Some(GameResult::Draw {
                            reason: DrawReason::Stalemate,
                        })
                    }
                }
            }
            Action::Resign => self.resign(color),
            Action::OfferDraw => self.offer_draw(color)?,
            Action::RetractDraw => self.retract_draw(color)?,
            Action::AcceptDraw => self.accept_draw(color)?,
        }

        Ok(())
    }

    pub fn accept_draw(&mut self, color: Color) -> Result<(), ApplyActionError> {
        let Some(offered_by) = self.draw_offer.take() else {
            return Err(ApplyActionError::DrawNotOffered);
        };

        if offered_by == color {
            return Err(ApplyActionError::DrawNotOffered);
        }

        self.result = Some(GameResult::Draw {
            reason: DrawReason::Agreement { offered_by },
        });

        Ok(())
    }

    #[inline]
    pub fn draw_offer(&self) -> Option<Color> {
        self.draw_offer
    }
}

/// The result of a [`Game`]
#[allow(missing_docs)]
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GameResult {
    Win { winner: Color, reason: WinReason },
    Draw { reason: DrawReason },
}

/// The way the player won a game.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum WinReason {
    /// The opponent was in check and had no legal moves remaining.
    ///
    /// If the king wasn't in check, it would be a [`DrawReason::Stalemate`]
    Checkmate,
    /// The opponent [resigned](`Action::Resign`).
    Resignation,
    /// The opponent ran out of time.
    Timeout,
}

/// The way a game resulted in a draw.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DrawReason {
    /// The opponent had no legal moves remaining, but it wasn't in check.
    ///
    /// If the king were in check, it would be a [`WinReason::Checkmate`].
    Stalemate,
    /// There were fifty moves played after the last pawn move.
    FiftyMoves,
    /// Both players agreed to a draw.
    Agreement {
        /// The color of the player that offerred a draw.
        offered_by: Color,
    },
}

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum ApplyActionError {
    #[error("Move is invalid: {0}")]
    MoveError(#[from] MoveError),

    #[error("Game is finished, cannot do any more moves.")]
    GameIsFinished,

    #[error("Opponent has not offered a draw.")]
    DrawNotOffered,

    #[error("It is your opponent's turn")]
    NotYourTurn,
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Move(mov) => write!(f, "plays move {mov:?}"),
            Action::Resign => write!(f, "resigns."),
            Action::OfferDraw => write!(f, "offers draw"),
            Action::RetractDraw => write!(f, "rectracts draw"),
            Action::AcceptDraw => write!(f, "accepts the draw"),
        }
    }
}
