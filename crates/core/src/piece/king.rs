use crate::{Color, board::Board, coordinate::Vec2, mov::Move, piece::Piece};

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    debug_assert_eq!(board.get(origin, color), Some(Piece::King));

    let delta = destination - origin;
    if delta.x.abs() + delta.y.abs() > 2 {
        return Err(MoveError::TooFarAway);
    }

    if let Some(piece) = board.get(destination, color) {
        return Err(MoveError::Blocked {
            position: destination,
            piece,
            color: color.other(),
        });
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
    TooFarAway,

    #[error("Blocked by {color} {piece} on {position}")]
    Blocked {
        position: Vec2,
        piece: Piece,
        color: Color,
    },
}

pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (Vec2::new_unchecked(0, 1), Color::White),
        (Vec2::new_unchecked(9, 10), Color::Black),
    ]
    .into_iter()
}
