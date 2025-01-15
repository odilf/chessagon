use crate::{
    Color,
    board::Board,
    coordinate::Vec2,
    mov::Move,
    piece::{Piece, movement},
};

use super::rook;

/// Whether the stride is a valid for a knight.
///
/// Knight strides are valid if they're one of the smallest strides that are neither a rook nor a bishop
/// stride.
pub fn is_stride(stride: Vec2<i8>) -> Result<(), MoveError> {
    let distance = stride.x.abs().max(stride.y.abs()) as u8;
    if distance > 3 {
        return Err(MoveError::TooFarAway { distance });
    }

    // Here we're not using `bishop::is_stride` because this is a cheaper check that makes the
    // knight movement valid. Namely, it simplifies since it removes the need to check for
    // movements like `(3, 0)`, which we would have to check separately if we were using
    // `bishop::is_stride`, since `(3, 0)` is not a bishop movement.
    //
    // That also means that the error below can be technically a bit misleading.
    // TODO: Reflect this in error
    if (stride.x + stride.y) % 3 == 0 {
        return Err(MoveError::BishopLikeMovement);
    }

    if rook::is_stride(stride) {
        return Err(MoveError::RookLikeMovement);
    }

    Ok(())
}

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    debug_assert_eq!(board.get(origin, color), Some(Piece::Knight));

    let delta = destination - origin;
    is_stride(delta)?;

    movement::check_color_blocker(destination, board, color)?;
    let captures = board.get(destination, color.other()).is_some();

    Ok(Move::Regular {
        origin,
        destination,
        captures,
    })
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("TODO")]
    TooFarAway { distance: u8 },

    #[error("TODO")]
    BishopLikeMovement,

    #[error("TODO")]
    RookLikeMovement,

    #[error("Blocked")]
    Blocked(#[from] movement::BlockerError),
}

pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (Vec2::new_unchecked(0, 2), Color::White),
        (Vec2::new_unchecked(2, 0), Color::White),
        (Vec2::new_unchecked(8, 10), Color::Black),
        (Vec2::new_unchecked(10, 8), Color::Black),
    ]
    .into_iter()
}
