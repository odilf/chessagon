use crate::{board::Board, coordinate::Vec2, mov::Move, piece::Piece, Color};

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_eq!(board.get(origin, color), Some(Piece::Rook));

    let delta = destination - origin;
    if !(delta.x == 0 || delta.y == 0 || delta.x == delta.y) {
        return Err(MoveError::InvalidDirection { delta });
    }

    let distance = if delta.x == 0 { delta.x } else { delta.y };
    let stride = delta.map(|x| x.signum());

    // TODO: Check for castling.

    // Check if it's blocked
    for i in 0..distance {
        let position = Vec2::new_unchecked(
            origin.x.wrapping_add((i * stride.x) as u8),
            origin.y.wrapping_add((i * stride.y) as u8),
        );
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
    #[error("Rooks can only move on straight lines (tried to move in direction {delta})")]
    InvalidDirection { delta: Vec2<i8> },

    #[error("Blocked by {color} {piece} on {position}{}", if *capturable { " (you can capture it)" } else { "" })]
    Blocked {
        position: Vec2,
        piece: Piece,
        color: Color,
        capturable: bool,
    },
}
