use crate::{board::Board, coordinate::Vec2, mov::Move, piece::Piece, Color};

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_eq!(board.get(origin, color), Some(Piece::Bishop));

    if origin.index() != destination.index() {
        return Err(MoveError::ChangeOfIndex {
            origin_index: origin.index(),
            destination_index: destination.index(),
        });
    }

    let delta = destination - origin;
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
    #[error("Rooks can only move on same-color tiles (tried to move from tile of type {origin_index} to {destination_index})")]
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
}
