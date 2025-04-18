use crate::{
    Color, IVec2, board::Board, coordinate::Vec2, ivec2, mov::Move, piece::movement, vec2,
};

pub const fn strides() -> [IVec2; 6] {
    [
        ivec2!(0, 1),
        ivec2!(1, 1),
        ivec2!(1, 0),
        ivec2!(0, -1),
        ivec2!(-1, -1),
        ivec2!(-1, 0),
    ]
}

pub const fn valid_stride(stride: IVec2) -> bool {
    (stride.x() == 0 && stride.y().abs() == 1)
        || (stride.x().abs() == 1 && stride.y() == 0)
        || (stride.x().abs() == 1 && stride.x() == stride.y())
}

pub const fn valid_delta(delta: IVec2) -> bool {
    delta.x() == 0 || delta.y() == 0 || delta.x() == delta.y()
}

/// Gets a move from `origin` to `destination` if the movement is rook-like.
///
/// See the [module-level docs](self) for more info about how a rook moves.
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

    if !valid_stride(stride) {
        return Err(MoveError::InvalidDirection { stride });
    }

    movement::check_blockers(origin, stride, distance, board)?;
    movement::check_color_blocker(destination, board, color)?;

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
    #[error("Rooks can only move on straight lines (tried to move with stride {stride})")]
    InvalidDirection { stride: IVec2 },

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),
}

pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [
        (vec2!(0, 3), Color::White),
        (vec2!(3, 0), Color::White),
        (vec2!(7, 10), Color::Black),
        (vec2!(10, 7), Color::Black),
    ]
    .into_iter()
}
