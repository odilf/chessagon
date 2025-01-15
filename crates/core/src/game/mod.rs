mod action;
mod time_control;
mod view;

use crate::{Color, board::Board, mov::Move, piece::MoveError};
use std::{
    fmt,
    time::{Duration, Instant},
};
pub use time_control::TimeControl;

#[derive(Debug, Clone)]
pub struct Game {
    board: Board,
    time_control: TimeControl,
    moves: Vec<(Move, Instant)>,
    result: Option<GameResult>,
    draw_offered: Option<Color>,
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
    pub fn new(time_control: TimeControl) -> Self {
        Self::from_position(Board::default(), time_control)
    }

    pub fn from_position(board: Board, time_control: TimeControl) -> Self {
        Self {
            board,
            time_control,
            moves: Vec::new(),
            result: None,
            draw_offered: None,
        }
    }

    pub fn turn(&self) -> Color {
        if self.moves.len() % 2 == 0 {
            Color::White
        } else {
            Color::Black
        }
    }

    pub fn board(&self) -> &Board {
        &self.board
    }

    pub fn result(&self) -> Option<GameResult> {
        self.result
    }

    pub fn moves_from(&self, color: Color) -> impl Iterator<Item = &(Move, Instant)> {
        // TODO: No chance in hell this can't be more efficient.
        self.moves
            .iter()
            .enumerate()
            .filter(move |(i, _)| i % 2 == color as usize)
            .map(|(_, item)| item)
    }

    pub fn time_remaining(&self, color: Color) -> Duration {
        todo!()
    }

    pub fn winner(&self) -> Option<Option<Color>> {
        self.result.map(|result| match result {
            GameResult::Win { winner, .. } => Some(winner),
            GameResult::Draw { .. } => None,
        })
    }

    pub fn is_finished(&self) -> bool {
        self.winner().is_some()
    }

    /// Tries to apply an [`Action`] from the given [`Color`].
    ///
    /// Returns an [`ApplyActionError`] if the specified move is not possible.
    pub fn apply_action(&mut self, action: Action, color: Color) -> Result<(), ApplyActionError> {
        match action {
            Action::Move(mov) => {
                if self.is_finished() {
                    return Err(ApplyActionError::GameIsFinished);
                }
                let now = Instant::now();
                self.moves.push((mov, now));
                self.board.apply_move(mov, color)?;
            }
            Action::Resign => {
                self.result = Some(GameResult::Win {
                    winner: color.other(),
                    reason: WinReason::Resignation,
                })
            }
            // TODO: Add a "draw already offered" error
            Action::OfferDraw => self.draw_offered = Some(color),
            // TODO: Add a "no offered draw to retract" error
            Action::RetractDraw => {
                self.draw_offered.take();
            }

            Action::AcceptDraw => {
                let Some(offered_by) = self.draw_offered.take() else {
                    return Err(ApplyActionError::DrawNotOffered);
                };
                self.result = Some(GameResult::Draw {
                    reason: DrawReason::Agreement { offered_by },
                })
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone, Copy)]
pub enum GameResult {
    Win { winner: Color, reason: WinReason },
    Draw { reason: DrawReason },
}

#[derive(Debug, Clone, Copy)]
pub enum WinReason {
    Checkmate,
    Resignation,
    Timeout,
}

#[derive(Debug, Clone, Copy)]
pub enum DrawReason {
    Stalemate,
    FiftyMoves,
    Agreement { offered_by: Color },
}

#[derive(Debug, thiserror::Error)]
pub enum ApplyActionError {
    #[error("Move is invalid")]
    MoveError(#[from] MoveError),

    #[error("Game is finished, cannot do any more moves.")]
    GameIsFinished,

    #[error("Opponent has not offered a draw.")]
    DrawNotOffered,
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
