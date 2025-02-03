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
pub fn valid_delta(delta: Vec2<i8>) -> Result<(), MoveError> {
    let distance = delta.x.abs().max(delta.y.abs()) as u8;
    if distance > 3 {
        return Err(MoveError::TooFarAway { distance });
    }

    // Here we're not using `bishop::valid_delta` because this is a cheaper check that makes the
    // knight movement valid. Namely, it simplifies since it removes the need to check for
    // movements like `(3, 0)`, which we would have to check separately if we were using
    // `bishop::is_delta`, since `(3, 0)` is not a bishop movement.
    //
    // That also means that the error below can be technically a bit misleading.
    // TODO: Reflect this in error
    if (delta.x + delta.y) % 3 == 0 {
        return Err(MoveError::BishopLikeMovement);
    }

    if rook::valid_delta(delta) {
        return Err(MoveError::RookLikeMovement);
    }

    Ok(())
}

/// Gets a move from `origin` to `destination` if the movement is knight-like.
///
/// See the [module-level docs](self) for more info about how a knight moves.
///
/// See [`Piece::get_move`](super::Piece::get_move) for more details about pre and postconditions.
pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    debug_assert_eq!(board.get(origin, color), Some(Piece::Knight));

    let delta = destination - origin;
    valid_delta(delta)?;

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
