#![allow(missing_docs)]

use chessagon_core::{
    Color,
    board::Board,
    game::{Action, Game, TimeControl},
};

pub mod matcher;
pub mod models;

pub trait Engine {
    fn new(color: Color, time_control: TimeControl) -> Self
    where
        Self: Sized;

    fn get_action(&mut self, game: &Game) -> Action;
    fn accept_draw_offer(&mut self, game: &Game) -> bool;

    fn eval(&mut self, board: &Board) -> f64;

    fn eval_for(&mut self, board: &Board, color: Color) -> f64 {
        let eval = self.eval(board);
        color.choose(eval, -eval)
    }
}
