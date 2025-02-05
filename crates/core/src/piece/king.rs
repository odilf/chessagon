//! Kings of chessagon.
//!
//! # Movement
//!
//! Kings can move one stride which is either [rook](super::rook)-like or [bishop](super::bishop)-like.
//!
//! This is a generalization of square chess, where the one rook stride corresponds to adjecent squares and the one
//! bishop stride corresponds to diagonal squares.
//!
// TODO: Add docs for numerical shortcut

use crate::{Color, board::Board, coordinate::Vec2, mov::Move, piece::movement};

/// Gets a move from `origin` to `destination` if the movement is king-like.
///
/// See the [module-level docs](self) for more info about how a king moves.
///
/// See [`Piece::get_move`](super::Piece::get_move) for more details about pre and postconditions.
pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);

    let delta = destination - origin;
    let distance = delta.x().abs() + delta.y().abs();
    if distance > 2 {
        return Err(MoveError::TooFarAway { distance });
    }

    movement::check_any_blocker(destination, board)?;
    let captures = board.get(destination, color.other()).is_some();

    Ok(Move::Regular {
        origin,
        destination,
        captures,
    })
}

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("The move destination is too far away ({distance} tiles away)")]
    TooFarAway { distance: i8 },

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),
}

/// The tiles where the kings are placed at the start of the game.
pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (Vec2::new_unchecked(0, 1), Color::White),
        (Vec2::new_unchecked(9, 10), Color::Black),
    ]
    .into_iter()
}
