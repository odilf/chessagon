use crate::{
    Color,
    board::Board,
    coordinate::Vec2,
    mov::Move,
    piece::{Piece, bishop, rook},
};

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
    [
        (Vec2::new_unchecked(1, 0), Color::White),
        (Vec2::new_unchecked(10, 9), Color::Black),
    ]
    .into_iter()
}
