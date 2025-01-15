use crate::{
    Color, Side,
    coordinate::Vec2,
    piece::{Piece, pawn},
};

#[derive(Debug, Copy, Clone)]
pub enum CheckStatus {
    Check,
    Checkmate,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub enum Move {
    Regular {
        /// Where the piece was at the start.
        origin: Vec2,

        /// Where the piece is at the end.
        destination: Vec2,

        /// Whether the move captures a piece.
        captures: bool,
    },

    /// on peasent
    ///
    /// Invariants
    /// - `Self::file` has to be a valid file (between 1 and 9, inclusive).
    EnPassant {
        /// The file the pawn was original from.
        file: u8,

        /// The side it captured towards.
        direction: Side,
    },

    Promotion {
        /// The file the pawn was originally on.
        file: u8,

        /// If the promotion captured a piece, to which side it captured it towards.
        captures: Option<Side>,

        /// Piece that pawn gets promoted to.
        promoting_to: Piece,
    },
}

impl Move {
    pub fn origin(self, color: Color) -> Vec2 {
        match self {
            Move::Regular { origin, .. } => origin,
            Move::EnPassant { file, .. } => pawn::initial_position_of_file(file, color)
                .expect("Move::EnPassant::file should always be between 1 and 9"),
            Move::Promotion { .. } => todo!(),
        }
    }

    pub fn destination(self, color: Color) -> Vec2 {
        match self {
            Move::Regular { destination, .. } => destination,
            Move::EnPassant { .. } => todo!(),
            Move::Promotion { .. } => todo!(),
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct MoveMeta {
    pub color: Color,
    pub checks: Option<CheckStatus>,
}
