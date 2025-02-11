//! Knights of chessagon.
//!
//! # Movement
//!
//! Knights can move to every tile that is tied for nearest that is not a valid position for a bishop or a rook.
//!
//! This may sound very different from the definition in square chess.
//!
//! TODO: Finish docs
//!
//! The movement from the center is as follows:
//! ```txt
#![doc = include_str!("../diagrams/movement_knight.txt")]
//! ```

use crate::{Color, IVec2, board::Board, coordinate::Vec2, mov::Move, piece::movement, vec2};

use super::rook;

/// Whether the stride is a valid for a knight.
///
/// Knight strides are valid if they're one of the smallest strides that are neither a rook nor a bishop
/// stride.
pub fn valid_delta(delta: IVec2) -> Result<(), MoveError> {
    // let distance = delta.x().abs().max(delta.y().abs()) as u8;
    let distance = delta.length();
    if distance > 3 {
        return Err(MoveError::TooFarAway { distance });
    }

    if rook::valid_delta(delta) {
        return Err(MoveError::RookLikeMovement);
    }

    // Here we're not using `bishop::valid_delta` because this will result in the same thing.
    // `bishop::valid_delta` checks for same index and for correct stride direction, but the second
    // check is redundant here because all tiles not filtered out yet that are on the same index also
    // have a correct stride direction.
    if (delta.x() + delta.y()) % 3 == 0 {
        return Err(MoveError::BishopLikeMovement);
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

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("Destination is too far away (distance is {distance})")]
    TooFarAway { distance: u8 },

    #[error("Movement is bishop-like")]
    BishopLikeMovement,

    #[error("Movement is rook-like")]
    RookLikeMovement,

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),
}

/// The tiles where the knights are placed at the start of the game.
pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (vec2!(0, 2), Color::White),
        (vec2!(2, 0), Color::White),
        (vec2!(8, 10), Color::Black),
        (vec2!(10, 8), Color::Black),
    ]
    .into_iter()
}

#[cfg(test)]
mod tests {
    use crate::{
        diagrams,
        piece::{Piece, knight},
        vec2,
    };

    use super::*;

    #[test]
    fn fn_moves_from_center_match_diagram() {
        let board = Board::new_minimal(Vec2::ZERO, vec2!(0, 1)).unwrap();
        let diagram = diagrams::visualize_tile_property(
            |dest| {
                if dest == Vec2::CENTER {
                    return Piece::Knight.emoji(Color::White);
                }

                match knight::get_move(Vec2::CENTER, dest, &board, Color::White) {
                    Ok(_) => 'x',
                    Err(_) => ' ',
                }
            },
            |x| *x,
        );

        assert_eq!(diagrams::MOVEMENT_KNIGHT.trim_end(), diagram.trim_end())
    }
}
