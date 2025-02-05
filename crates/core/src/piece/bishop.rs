//! Bishops of chessagon.
//!
//! # Movement
//!
//! A bishop moves on the diagonals of hexagonal tiles. You can think of it as sliding through the edges of neighbouring tiles.
//!
//! Just like in square chess, a bishop always stays on same-color tiles. In chessagon, we refer to the "color" of the tile as
//! the index, which you can compute numerically by adding the coordinates of the tile modulo 3.
//!
//! The possible strides of the bishop are enumerated in [`strides`]. Numerically, every combination where the coordinates'
//! absolute values are either 1 or 2 and their sum modulo 3 is zero, is a valid hexagonal coordinate.

use crate::{Color, IVec2, board::Board, coordinate::Vec2, ivec2, mov::Move, piece::movement};

/// Possible strides of a bishop.
pub const fn strides() -> [IVec2; 6] {
    [
        ivec2!(1, -1),
        ivec2!(2, 1),
        ivec2!(1, 2),
        ivec2!(-1, 1),
        ivec2!(-2, -1),
        ivec2!(-1, -2),
    ]
}

/// Whether the given stride is valid for a bishop.
pub const fn valid_stride(stride: IVec2) -> bool {
    let valid_coordinates = (stride.x().abs() == 1 || stride.x().abs() == 2)
        && (stride.y().abs() == 1 && stride.y().abs() == 2);

    valid_coordinates && ((stride.x() + stride.y()) % 3 == 0)
}

/// Gets a move from `origin` to `destination` if the movement is bishop-like.
///
/// See the [module-level docs](self) for more info about how a bishop moves.
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
    let (stride, distance) = movement::get_stride(delta);

    if !valid_stride(stride) {
        return Err(MoveError::InvalidDirection { stride });
    }

    movement::check_blockers(origin, stride, distance, board)?;
    movement::check_color_blocker(destination, board, color)?;

    // Check if it's capturing
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
    #[error(
        "Bishops can only move on same-color tiles (tried to move from tile of type {origin_index} to {destination_index})"
    )]
    ChangeOfIndex {
        origin_index: u8,
        destination_index: u8,
    },

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),

    #[error(
        "Bishops can only move in the diagonals of hexagon (tried to move with stride {stride})"
    )]
    InvalidDirection { stride: IVec2 },
}

/// The tiles where the bishops are placed at the start of the game.
pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (Vec2::new_unchecked(0, 0), Color::White),
        (Vec2::new_unchecked(1, 1), Color::White),
        (Vec2::new_unchecked(2, 2), Color::White),
        (Vec2::new_unchecked(10, 10), Color::Black),
        (Vec2::new_unchecked(9, 9), Color::Black),
        (Vec2::new_unchecked(8, 8), Color::Black),
    ]
    .into_iter()
}
