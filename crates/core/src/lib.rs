//! Core functionality of chessagon (hexagonal chess).

mod board;
pub(crate) mod diagrams;
mod mov;
mod sides;

pub mod coordinate;
pub mod game;
pub mod piece;

pub use board::Board;
pub use coordinate::{IVec2, Vec2};
pub use game::Game;
pub use mov::Move;
pub use sides::{Color, Side};
