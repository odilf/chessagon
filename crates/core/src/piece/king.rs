use crate::{board::Board, coordinate::Vec2, mov::Move, piece::Piece, Color};

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_eq!(board.get(origin, color), Some(Piece::King));
    
    let delta = destination - origin;
    if delta.x.abs() + delta.y.abs() > 2 {
        return Err(MoveError::TooFarAway)
    }

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
    TooFarAway
}
