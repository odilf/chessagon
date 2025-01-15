// #![warn(missing_docs)]

use std::sync::mpsc;

use chessagon_core::{
    Color,
    game::{Action, Game, TimeControl},
};

pub mod matcher;
pub mod models;
mod options;

pub trait Engine {
    fn new(color: Color, time_control: TimeControl) -> Self
    where
        Self: Sized;

    fn get_action(&mut self, game: &Game) -> Action;
    fn accept_draw_offer(&mut self, game: &Game) -> bool;
}
