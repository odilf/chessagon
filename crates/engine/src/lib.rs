pub mod board;
pub mod coordinate;
pub mod diagrams;
pub mod mov;
pub mod piece;

mod color;

pub use color::Color;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Side {
    King,
    Queen,
}
