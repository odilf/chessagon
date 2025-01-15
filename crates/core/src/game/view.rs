use crate::board::Board;

use super::Game;

pub struct GameView<'a> {
    original: &'a Game,
    current_board: Board,
}
