use crate::{Color, IVec2, board::Board, coordinate::Vec2, mov::Move, piece::movement, vec2};

// /// Gets the stride of a pawn given the color and the optional direction of the capture.
// pub const fn stride(color: Color, capture_direction: Option<Side>) -> IVec2 {
//     match capture_direction {
//         None => IVec2::new_unchecked(color.direction(), color.direction()),
//         Some(side) => side.step_towards(color.direction()),
//     }
// }

pub const fn is_straight_stride(stride: IVec2, color: Color) -> bool {
    stride.x() == stride.y() && stride.y() == color.direction()
}

pub const fn is_capture_stride(stride: IVec2, color: Color) -> bool {
    (stride.x() == 0 && stride.y() == color.direction())
        || (stride.x() == color.direction() && stride.y() == 0)
}

/// Gets a move from `origin` to `destination` if the movement is pawn-like.
///
/// See the [module-level docs](self) for more info about how a pawn moves.
///
/// See [`Piece::get_move`](super::Piece::get_move) for more details about pre and postconditions.
pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    let delta = destination - origin;
    let (stride, distance) = movement::get_stride(delta);

    let captures = if is_straight_stride(stride, color) {
        let max_distance = if is_intial_tile(origin, color) { 2 } else { 1 };
        if distance > max_distance {
            return Err(MoveError::TooFarAway {
                distance,
                max_distance,
            });
        }

        // TODO: If `max_distance == 1`, this should get optimized away, maybe that should be made explicit.
        movement::check_blockers(origin, stride, distance, board)?;
        movement::check_any_blocker(destination, board)?;

        false
    } else if is_capture_stride(stride, color) {
        if distance > 1 {
            return Err(MoveError::CaptureTooFarAway { distance });
        }

        let Some(_piece) = board.get(destination, color.other()) else {
            return Err(MoveError::NoPieceToCapture {
                position: destination,
            });
        };

        true
    } else {
        return Err(MoveError::InvalidMovementDirection { delta });
    };

    Ok(Move::Regular {
        origin,
        destination,
        captures,
    })
}

#[allow(missing_docs)]
#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error(
        "Target is {distance} tiles away, but can only move {max_distance} tiles from this position"
    )]
    TooFarAway { distance: u8, max_distance: u8 },

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),

    #[error("Capture target is {distance} tiles away, but you can only capture neighbors")]
    CaptureTooFarAway { distance: u8 },

    #[error("TODO")]
    NoPieceToCapture { position: Vec2 },

    #[error(
        "Pawns can only move forward or capture diagonally (it's moving in direction {delta})."
    )]
    InvalidMovementDirection { delta: IVec2 },
}

pub const fn initial_white_tiles() -> [Vec2; 9] {
    [
        vec2!(4, 0),
        vec2!(4, 1),
        vec2!(4, 2),
        vec2!(4, 3),
        vec2!(4, 4),
        vec2!(3, 4),
        vec2!(2, 4),
        vec2!(1, 4),
        vec2!(0, 4),
    ]
}

pub fn initial_black_tiles() -> [Vec2; 9] {
    initial_white_tiles().map(|position| position.flipped())
}

pub const fn max_coordinate_of_initial_position(color: Color) -> u8 {
    color.choose(4, 6)
}

pub const fn is_initial_white_tile(position: Vec2) -> bool {
    (position.x() == 4 && position.y() <= 4) || (position.x() <= 4 && position.y() == 4)
}

pub const fn is_initial_black_tile(position: Vec2) -> bool {
    (position.x() == 6 && position.y() >= 6) || (position.x() >= 6 && position.y() == 6)
}

pub fn is_intial_tile(position: Vec2, color: Color) -> bool {
    match color {
        Color::White => is_initial_white_tile(position),
        Color::Black => is_initial_black_tile(position),
    }
}

pub fn initial_position_of_file(file: u8, color: Color) -> Option<Vec2> {
    let m = max_coordinate_of_initial_position(color);
    // We have two conditions:
    // - `file == pos.y - pos.x + 5`
    // - `pos.x == m || pos.y == m` (either one of 4 or 6)
    //
    // Then, the solutions are either
    // `pox.x = m` and `pos.y = file + m - 5`
    // `pos.y = m` and `pos.x = m - file + 5`, or
    //
    // We know that positions can't be negative, so we discard them if they would result in negatives.
    //
    // Both of them could be valid, and we need to chose the one that is closer to the given color.

    let a = (file + m >= 5).then(|| Vec2::new_unchecked(m, file + m - 5));
    let b = (m + 5 >= file).then(|| Vec2::new_unchecked(m + 5 - file, m)); // Subtraction has to go at the end, otherwise it overflows

    a.into_iter()
        .chain(b)
        .max_by(|a, b| color.compare_towards(a.rank(), b.rank()).reverse())
}

pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    let white = initial_white_tiles()
        .into_iter()
        .map(|position| (position, Color::White));

    let black = initial_black_tiles()
        .into_iter()
        .map(|position| (position, Color::Black));

    white.chain(black)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_declared_initial_white_tile_is_detected_as_initial() {
        for position in Vec2::iter() {
            assert_eq!(
                is_initial_white_tile(position),
                initial_white_tiles().contains(&position)
            )
        }
    }

    #[test]
    fn every_declared_initial_black_tile_is_detected_as_initial() {
        for position in Vec2::iter() {
            assert_eq!(
                is_initial_black_tile(position),
                initial_black_tiles().contains(&position)
            )
        }
    }

    #[test]
    fn fn_initial_position_of_file_returns_position_at_given_file() {
        for file in 1..Board::NUMBER_OF_FILES - 1 {
            let white_pos = initial_position_of_file(file, Color::White).unwrap();
            assert_eq!(white_pos.file(), file,);
            assert!(is_initial_white_tile(white_pos));

            let black_pos = initial_position_of_file(file, Color::Black).unwrap();
            assert_eq!(black_pos.file(), file);
            assert!(is_initial_black_tile(black_pos));
        }
    }

    // #[test]
    // fn fn_stride_returns_correct_result_for_each_possible_value() {
    //     for (color, captures, [x, y]) in [
    //         (Color::White, None, [1, 1]),
    //         (Color::White, Some(Side::Queen), [1, 0]),
    //         (Color::White, Some(Side::King), [0, 1]),
    //         (Color::Black, None, [-1, -1]),
    //         (Color::Black, Some(Side::Queen), [0, -1]),
    //         (Color::Black, Some(Side::King), [-1, 0]),
    //     ] {
    //         assert_eq!(stride(color, captures), IVec2::new_unchecked(x, y));
    //     }
    // }
}
