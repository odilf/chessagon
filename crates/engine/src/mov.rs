use crate::{coordinate::Vec2, Color, Side};

#[derive(Debug, Clone)]
pub enum CheckStatus {
    Check,
    Checkmate,
}

#[derive(Debug, Clone)]
pub enum Move {
    Regular {
        origin: Vec2,
        destination: Vec2,

        captures: bool,
    },

    /// Interchange of the rook and the king.
    Castle {
        color: Color,
        side: Side,
    },

    /// on peasent
    EnPassant {
        file: u8,
    },

    Promotion {
        file: u8,
        captures: bool,
    },
}

pub struct MoveMeta {
    pub color: Color,
    pub checks: Option<CheckStatus>,
}

pub struct FullMove {
    pub mov: Move,
    pub meta: MoveMeta,
}

