use crate::{
    board::Board,
    coordinate::Vec2,
    mov::Move,
    piece::{bishop, rook, Piece},
    Color,
};

pub fn get_move(
    origin: Vec2,
    destination: Vec2,
    board: &Board,
    color: Color,
) -> Result<Move, MoveError> {
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
