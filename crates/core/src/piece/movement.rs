//! General piece movement logic.

use crate::{Color, IVec2, board::Board, coordinate::Vec2};
use gcd::Gcd;

use super::Piece;

/// The number of times `stride` happens in `delta`. Returns `Err` if `delta` is not a multiple of `stride`.
///
/// In other words, `stride * stride_length(stride, delta) == delta`
pub fn stride_length(stride: IVec2, delta: IVec2) -> Result<i8, StrideLengthError> {
    let (sx, sy) = (stride.x(), stride.y());
    let (dx, dy) = (delta.x(), delta.y());

    let divides_evenly = (dx % sx == 0) && (dy % sy == 0) && (dx / sx == dy / sy);
    if !divides_evenly {
        return Err(StrideLengthError { stride, delta });
    }

    Ok(dx / sx)
}

#[derive(Debug, thiserror::Error)]
#[error("The stride {stride} does not evenly divide into the delta {delta}")]
pub struct StrideLengthError {
    stride: IVec2,
    delta: IVec2,
}

/// Gets the stride of a movement, along with the number of strides that fit in `delta`.
///
/// In other words, it finds the smallest vector that can divide `delta`.
///
/// # Panics
///
/// If `delta` is [`IVec2::ZERO`].
///
/// # Examples
///
/// ```rust
/// use chessagon_core::{ivec2, piece::movement::get_stride};
///
/// let delta = ivec2!(-6, 3);
/// let (stride, length) = get_stride(delta);
///
/// assert_eq!(stride, ivec2!(-2, 1));
/// assert_eq!(length, 3);
///
/// // True for all arguments
/// assert_eq!(stride * length as i8, delta);
/// ```
///
/// ```rust
/// use chessagon_core::{ivec2, piece::movement::get_stride};
///
/// let delta = ivec2!(-6, 4);
/// let (stride, length) = get_stride(delta);
///
/// assert_eq!(stride, ivec2!(-3, 2));
/// assert_eq!(length, 2);
///
/// // True for all arguments
/// assert_eq!(stride * length as i8, delta);
/// ```
pub fn get_stride(delta: IVec2) -> (IVec2, u8) {
    let gcd = (delta.x().abs() as u8).gcd(delta.y().abs() as u8);
    (delta / gcd as i8, gcd)
}

/// Check whether there's a blocker of a specific color at that specific coordinate.
///
/// This is used mostly to check if the tile the piece is moving towards has a piece of the
/// same color, which would block it, but if it's a different color it could capture it.
///
/// See also [`check_any_blocker`].
pub fn check_color_blocker(
    position: Vec2,
    board: &Board,
    color: Color,
) -> Result<(), BlockerError> {
    if let Some(piece) = board.get(position, color) {
        return Err(BlockerError {
            position,
            piece,
            color,
        });
    }

    Ok(())
}

/// Checks if there is a blocker of either color in that specific coordinate.
///
/// This is used to check if there's any piece blocking movement in between two tiles
/// through [`check_blockers`].
///
/// See also [`check_color_blocker`].
pub fn check_any_blocker(position: Vec2, board: &Board) -> Result<(), BlockerError> {
    if let Some((piece, color)) = board.get_either(position) {
        return Err(BlockerError {
            position,
            piece,
            color,
        });
    }

    Ok(())
}

/// Checks if there are any blockers at any of the strides given by the distance.
///
/// Doesn't check for blockers at the final tile (i.e., `origin + distance * stride`).
///
/// See also [`check_color_blocker`] and [`check_any_blocker`].
pub fn check_blockers(
    origin: Vec2,
    stride: IVec2,
    distance: u8,
    board: &Board,
) -> Result<(), BlockerError> {
    for i in 1..distance {
        let position = origin + stride * i as i8;
        check_any_blocker(position, board)?;
    }

    Ok(())
}

#[derive(Debug, thiserror::Error)]
#[error("Blocked by {color} {piece} on {position}")]
pub struct BlockerError {
    pub position: Vec2,
    pub piece: Piece,
    pub color: Color,
}
