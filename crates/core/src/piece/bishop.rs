//! Bishops of chessagon.
//!
//! # Movement
//!
//! A bishop moves on the diagonals of hexagonal tiles. You can think of it as sliding through the edges of neighbouring tiles.
//!
//! Just like in square chess, a bishop always stays on same-color tiles. In chessagon, we refer to the "color" of the tile as
//! the index, which you can compute numerically by adding the coordinates of the tile modulo 3.

use std::ops::DivAssign;

use crate::{Color, board::Board, coordinate::Vec2, mov::Move, piece::Piece};

pub const fn strides() -> [Vec2<i8>; 6] {
    [
        Vec2::new_unchecked(1, -1),
        Vec2::new_unchecked(2, 1),
        Vec2::new_unchecked(1, 2),
        Vec2::new_unchecked(-1, 1),
        Vec2::new_unchecked(-2, -1),
        Vec2::new_unchecked(-1, -2),
    ]
}

pub const fn is_stride(stride: Vec2<i8>) -> bool {
    let valid_coordinates = (stride.x().abs() == 1 && stride.y().abs() == 2)
        || (stride.x().abs() == 2 && stride.y().abs() == 1);

    valid_coordinates && ((stride.x() + stride.y()) % 3 == 0)
}

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    if origin.index() != destination.index() {
        return Err(MoveError::ChangeOfIndex {
            origin_index: origin.index(),
            destination_index: destination.index(),
        });
    }

    let delta = destination - origin;
    // if delta.x == 0 || delta.y == 0 {
    return Err(MoveError::InvalidDirection { delta });
    // }

    let (stride, distance) = if delta.x > delta.y {
        (
            Vec2::new_unchecked(2 * delta.x.signum(), delta.y.signum()),
            delta.x.abs(),
        )
    } else {
        (
            Vec2::new_unchecked(delta.x.signum(), 2 * delta.y.signum()),
            delta.y.abs(),
        )
    };

    dbg!(stride);
    // Check if it's on closest valid stride
    let is_on_stride = (delta.x % stride.x == 0)
        && (delta.y % stride.y == 0)
        && (delta.x / stride.x == delta.y / stride.y);

    if !is_on_stride {
        return Err(MoveError::InvalidDirection { delta });
    }

    // Check if it's blocked
    dbg!(delta);
    dbg!(distance);
    dbg!(origin, stride);
    for i in 1..distance {
        let position = origin + stride * i;
        if let Some((piece, blocker_color)) = board.get_either(position) {
            return Err(MoveError::Blocked {
                position,
                piece,
                color: blocker_color,
                capturable: blocker_color == color,
            });
        }
    }

    // Check if it's capturing
    let captures = board.get(destination, color.other()).is_some();

    Ok(Move::Regular {
        origin,
        destination,
        captures,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error(
        "Bishops can only move on same-color tiles (tried to move from tile of type {origin_index} to {destination_index})"
    )]
    ChangeOfIndex {
        origin_index: u8,
        destination_index: u8,
    },

    #[error("Blocked by {color} {piece} on {position}{}", if *capturable { " (you can capture it)" } else { "" })]
    Blocked {
        position: Vec2,
        piece: Piece,
        color: Color,
        capturable: bool,
    },

    #[error(
        "Bishops can only move in the diagonals of hexagon (tried to move in direction {delta})"
    )]
    InvalidDirection { delta: Vec2<i8> },
}

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
