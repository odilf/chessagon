//! Kings of chessagon.
//!
//! # Movement
//!
//! Kings can move one stride which is either [rook](super::rook)-like or [bishop](super::bishop)-like.
//!
//! This is a generalization of square chess, where the one rook stride corresponds to adjecent squares and the one
//! bishop stride corresponds to diagonal squares.
//!
// TODO: Add docs for numerical shortcut

use crate::{
    Color, IVec2,
    board::Board,
    coordinate::Vec2,
    ivec2,
    mov::Move,
    piece::{bishop, movement, rook},
    vec2,
};

/// Gets a move from `origin` to `destination` if the movement is king-like.
///
/// See the [module-level docs](self) for more info about how a king moves.
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

    if distance > 1 {
        return Err(MoveError::TooFarAway { distance });
    }

    if !(bishop::valid_stride(stride) || rook::valid_stride(stride)) {
        return Err(MoveError::IncorrectStride);
    }

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
    #[error("The move destination is too far away ({distance} tiles away)")]
    TooFarAway { distance: u8 },

    #[error("The stride is neither bishop-like nor rook-like")]
    IncorrectStride,

    #[error("{0}")]
    Blocked(#[from] movement::BlockerError),
}

/// The tiles where the kings are placed at the start of the game.
pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [(vec2!(0, 1), Color::White), (vec2!(9, 10), Color::Black)].into_iter()
}

pub const VALID_DELTAS: [IVec2; 12] = [
    // Rook-like
    ivec2!(1, 0),
    ivec2!(1, 1),
    ivec2!(0, 1),
    ivec2!(0, -1),
    ivec2!(-1, -1),
    ivec2!(-1, 0),
    // Bishop-like
    ivec2!(2, 1),
    ivec2!(1, 2),
    ivec2!(-1, 1),
    ivec2!(-2, -1),
    ivec2!(-1, -2),
    ivec2!(1, -1),
];
