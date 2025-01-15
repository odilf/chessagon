//! General piece movement logic.

use crate::{Color, board::Board, coordinate::Vec2};
use gcd::Gcd;

use super::Piece;

/// The number of times `stride` happens in `delta`. Returns `Err` if `delta` is not a multiple of `stride`.
///
/// In other words, `stride * stride_length(stride, delta) == delta`
pub fn stride_length(stride: Vec2<i8>, delta: Vec2<i8>) -> Result<i8, StrideLengthError> {
    let (sx, sy) = (stride.x, stride.y);
    let (dx, dy) = (delta.x, delta.y);

    let divides_evenly = (dx % sx == 0) && (dy % sy == 0) && (dx / sx == dy / sy);
    if !divides_evenly {
        return Err(StrideLengthError { stride, delta });
    }

    Ok(dx / sx)
}

#[derive(Debug, thiserror::Error)]
#[error("The stride {stride} does not evenly divide into the delta {delta}")]
pub struct StrideLengthError {
    stride: Vec2<i8>,
    delta: Vec2<i8>,
}

/// Gets the stride of a movement, along with the number of strides that fit in `delta`.
///
/// In other words, it finds the smallest vector that can divide `delta`.
///
/// # Examples
///
/// ```rust
/// use chessagon_core::{Vec2, piece::movement::get_stride};
///
/// let delta = Vec2::new_unchecked(15, 5);
/// let (stride, length) = get_stride(delta);
///
/// assert_eq!(stride, Vec2::new_unchecked(3, 1));
/// assert_eq!(length, 5);
///
/// // True for all arguments
/// assert_eq!(stride * length as i8, delta);
/// ```
///
/// ```rust
/// use chessagon_core::{Vec2, piece::movement::get_stride};
///
/// let delta = Vec2::new_unchecked(-6, 4);
/// let (stride, length) = get_stride(delta);
///
/// assert_eq!(stride, Vec2::new_unchecked(-3, 2));
/// assert_eq!(length, 2);
///
/// // True for all arguments
/// assert_eq!(stride * length as i8, delta);
/// ```
pub fn get_stride(delta: Vec2<i8>) -> (Vec2<i8>, u8) {
    let gcd = (delta.x.abs() as u8).gcd(delta.y.abs() as u8);
    (delta / gcd as i8, gcd)
}

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

pub fn check_blockers(
    origin: Vec2,
    stride: Vec2<i8>,
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
