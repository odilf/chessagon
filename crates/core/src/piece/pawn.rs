use crate::{
    board::Board,
    coordinate::Vec2,
    mov::{FullMove, Move, MoveMeta},
    Color,
};

use super::Piece;

pub const fn initial_white_tiles() -> [Vec2; 9] {
    [
        Vec2::new_unchecked(4, 0),
        Vec2::new_unchecked(4, 1),
        Vec2::new_unchecked(4, 2),
        Vec2::new_unchecked(4, 3),
        Vec2::new_unchecked(4, 4),
        Vec2::new_unchecked(3, 4),
        Vec2::new_unchecked(2, 4),
        Vec2::new_unchecked(1, 4),
        Vec2::new_unchecked(0, 4),
    ]
}

pub fn initial_black_tiles() -> [Vec2; 9] {
    initial_white_tiles().map(|position| position.flipped())
}

pub fn is_initial_white_tile(position: Vec2) -> bool {
    (position.x == 4 && position.y <= 4) || (position.x <= 4 && position.y == 4)
}

pub fn is_initial_black_tile(position: Vec2) -> bool {
    (position.x == 6 && position.y >= 6) || (position.x >= 6 && position.y == 6)
}

pub fn is_intial_tile(position: Vec2, color: Color) -> bool {
    color.choose::<fn(Vec2) -> bool>(is_initial_white_tile, is_initial_black_tile)(position)
}

pub fn can_move_unobstructed_no_capture(origin: Vec2, destination: Vec2, color: Color) -> bool {
    let delta = destination - origin;
    if delta.x != delta.y {
        return false;
    }

    let delta = delta.x;
    if delta.signum() != color.direction() {
        return false;
    }

    let is_initial = is_intial_tile(origin, color);
    let max_distance = if is_initial { 2 } else { 1 };

    delta.abs() <= max_distance
}

pub fn can_capture(origin: Vec2, destination: Vec2, board: Board, color: Color) -> bool {
    let delta = destination - origin;
    if !(delta.x == color.direction() && delta.y == 0)
        && !(delta.x == 0 && delta.y == color.direction())
    {
        return false;
    }

    board.get(destination, color.other()).is_some()
}

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_eq!(board.get(origin, color), Some(Piece::Pawn));

    let delta = destination - origin;
    match delta.x.abs_diff(delta.y) {
        // Straight, advancing
        0 => {
            let moved = delta.x; // Or delta.y, since delta.x - delta.y == 0
            if moved.signum() != color.direction() {
                return Err(MoveError::MovingBackwards);
            }

            match moved.abs() {
                1 => {
                    // Moving 1, only have to check if it's blocked where the piece is moving to is
                    // blocked
                    if let Some((piece, color)) = board.get_either(destination) {
                        return Err(MoveError::Blocked {
                            piece,
                            color,
                            position: destination,
                        });
                    }

                    // TODO: Check for promotions

                    Ok(Move::Regular {
                        origin,
                        destination,
                        captures: false,
                    })
                }

                2 => {
                    // Moving 2, have to check if it's in an initial position...
                    if !is_intial_tile(origin, color) {
                        return Err(MoveError::MovingTwoFromNonInitialTile { origin });
                    }

                    // ...and whether either of the two is blocked.
                    let middle = (origin + destination).map(|x| x / 2);
                    for position in [middle, destination] {
                        // TODO: Maybe factor this out
                        if let Some((piece, color)) = board.get_either(position) {
                            return Err(MoveError::Blocked {
                                piece,
                                color,
                                position,
                            });
                        }
                    }

                    Ok(Move::Regular {
                        origin,
                        destination,
                        captures: false,
                    })
                }

                distance => {
                    return Err(MoveError::TooFarAway {
                        distance: distance as u8,
                    })
                }
            }
        }

        // Diagonal capture
        1 => {
            // TODO: Prettier/more efficient way to do this?
            let moved = if delta.x == 0 { delta.y } else { delta.x };
            if moved.signum() != color.direction() {
                return Err(MoveError::MovingBackwards);
            }

            if moved > 1 {
                return Err(MoveError::CaptureTooFarAway { destination });
            }

            // TODO: Implement on passant
            let Some(_piece) = board.get(destination, color.other()) else {
                return Err(MoveError::NoPieceToCapture {
                    position: destination,
                });
            };

            Ok(Move::Regular {
                origin,
                destination,
                captures: true,
            })
        }

        _ => return Err(MoveError::InvalidMovementDirection { delta }),
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error(
        "Pawns can only move forward or capture diagonally (it's moving in direction {delta})."
    )]
    InvalidMovementDirection { delta: Vec2<i8> },

    #[error("Pawns can only move towards the oppponent.")]
    MovingBackwards,

    #[error("TODO")]
    TooFarAway { distance: u8 },

    #[error("TODO")]
    Blocked {
        position: Vec2,
        piece: Piece,
        color: Color,
    },

    #[error("TODO")]
    MovingTwoFromNonInitialTile { origin: Vec2 },

    #[error("TODO")]
    CaptureTooFarAway { destination: Vec2 },

    #[error("TODO")]
    NoPieceToCapture { position: Vec2 },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn every_declared_initial_white_tile_is_detected_as_initial() {
        for position in Vec2::iter() {
            assert_eq!(
                is_initial_white_tile(position),
                initial_white_tiles().contains(&position)
            )
        }
    }

    #[test]
    pub fn every_declared_initial_black_tile_is_detected_as_initial() {
        for position in Vec2::iter() {
            assert_eq!(
                is_initial_black_tile(position),
                initial_black_tiles().contains(&position)
            )
        }
    }
}
