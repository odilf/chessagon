use crate::{board::Board, coordinate::Vec2, mov::Move, piece::Piece, Color};

/// Whether the delta is a valid knight stride.
///
/// Knight strides are valid if it's one of the closest tiles that are neither a rook nor a bishop
/// stride.
pub fn is_valid_knight_stride(delta: Vec2<i8>) -> Result<(), MoveError> {
    let distance = delta.x.abs().max(delta.y.abs()) as u8;
    if distance > 3 {
        return Err(MoveError::TooFarAway { distance });
    }

    if (delta.x + delta.y) % 3 == 0 {
        return Err(MoveError::BishopLikeMovement);
    }

    if delta.x == 0 || delta.y == 0 || delta.x == delta.y {
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
    debug_assert_eq!(board.get(origin, color), Some(Piece::Knight));

    let delta = destination - origin;
    // TODO: Maybe find better name. 
    is_valid_knight_stride(delta)?;

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
}
