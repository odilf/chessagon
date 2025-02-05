use std::fmt;

use crate::{Color, Side, coordinate::Vec2, piece::Piece};

/// Translations of pieces with optional captures.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Move {
    /// Usual kind of move, where a piece moves from a tile to another tile, optionally
    /// capturing the piece in the destination tile. There are no other side-effects.
    Regular {
        /// Where the piece was at the start.
        origin: Vec2,

        /// Where the piece is at the end.
        destination: Vec2,

        /// Whether the move captures a piece.
        captures: bool,
    },

    /// A capture of a pawn _in passing_.
    ///
    /// This can occur when an opponent's pawn has moved two tiles on the previous pawn,
    /// and it now stands next to a pawn of yours. Said pawn can capture the pawn that just moved.
    ///
    /// The opponent's pawn is captured and your pawn moves as a regular pawn capture.
    ///
    /// See [`crate::piece::pawn`] for how pawns generally move.
    ///
    /// Invariants
    /// - `Self::file` has to be a valid file (between 1 and 9, inclusive).
    EnPassant {
        /// The file the pawn was original from.
        file: u8,

        /// The side it captured towards.
        direction: Side,
    },

    /// A pawn has reached the opposite end of the board, and is converted to a new piece.
    ///
    /// Invariants
    /// - `Self::promoting_to` can't be a pawn nor the king.
    Promotion {
        /// The file the pawn was originally on.
        file: u8,

        /// If the promotion captured a piece, to which side it captured it towards.
        captures: Option<Side>,

        /// Piece that pawn gets promoted to.
        promoting_to: Piece,
    },
}

impl fmt::Display for Move {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} -> {}", self.origin(), self.destination())
    }
}

impl Move {
    /// The tile where the piece was at before the move.
    pub fn origin(self) -> Vec2 {
        match self {
            Move::Regular { origin, .. } => origin,
            Move::EnPassant { .. } => todo!(),
            // Move::EnPassant { file, .. } => pawn::initial_position_of_file(file, color)
            //     .expect("Move::EnPassant::file should always be between 1 and 9"),
            Move::Promotion { .. } => todo!(),
        }
    }

    /// The tile where the piece went to after the move.
    pub fn destination(self) -> Vec2 {
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
}
