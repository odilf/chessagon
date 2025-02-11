use crate::{
    Color,
    board::Board,
    coordinate::Vec2,
    mov::Move,
    piece::{Piece, bishop, rook},
    vec2,
};

/// Gets a move from `origin` to `destination` if the movement is queen-like.
///
/// See the [module-level docs](self) for more info about how a queen moves.
///
/// See [`Piece::get_move`](super::Piece::get_move) for more details about pre and postconditions.
pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
    debug_assert_ne!(origin, destination);
    debug_assert_eq!(board.get(origin, color), Some(Piece::Queen));

    rook::get_move(origin, destination, board, color).or_else(|rook_err| {
        bishop::get_move(origin, destination, board, color).map_err(|bishop_err| MoveError {
            rook_err,
            bishop_err,
        })
    })
}

#[derive(Debug, thiserror::Error)]
#[error("Can't do rook-like movement ({rook_err}) nor bishop-like movement ({bishop_err})")]
pub struct MoveError {
    rook_err: rook::MoveError,
    bishop_err: bishop::MoveError,
}

pub fn initial_configuration() -> impl Iterator<Item = (Vec2, Color)> {
    [(vec2!(1, 0), Color::White), (vec2!(10, 9), Color::Black)].into_iter()
}
