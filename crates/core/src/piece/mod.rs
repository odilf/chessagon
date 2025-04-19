//! Actors of chessagon.
//!
//! Includes utilities for each specific kind of bishop.

pub mod bishop;
pub mod king;
pub mod knight;
pub mod movement;
pub mod pawn;
pub mod queen;
pub mod rook;

use core::fmt;
use std::ops::Index;

use strum::EnumString;

use crate::{
    Color,
    board::Board,
    coordinate::Vec2,
    mov::{Move, MoveMeta},
};

/// A piece in chessagon.
#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Piece {
    /// A [`pawn`]
    Pawn = 0,
    /// A [`knight`]
    Knight = 1,
    /// A [`bishop`]
    Bishop = 2,
    /// A [`rook`]
    Rook = 3,
    /// A [`queen`]
    Queen = 4,
    /// A [`king`]
    King = 5,
}

impl Piece {
    /// Gets the move from `origin` to `destination`, if it is legal, assuming the given piece is at `origin`.
    ///
    /// # Panics
    /// - If the piece at the `origin` position is not `self` with the given color.
    ///
    /// # Preconditions and delegation
    ///
    /// This method delegates most of its logic to piece-specific `get_move` methods:
    /// - [`pawn::get_move`]
    /// - [`knight::get_move`]
    /// - [`bishop::get_move`]
    /// - [`rook::get_move`]
    /// - [`queen::get_move`]
    /// - [`king::get_move`]
    ///
    /// These methods only check if the type of movement is correct. They assume:
    /// - that the piece at `origin` is the given type
    /// - that the movement is not null (i.e., not (0, 0)).
    /// - that they don't leave the king in check.
    pub fn get_move(
        &self,
        origin: Vec2,
        destination: Vec2,
        board: &Board,
        color: Color,
    ) -> Result<(Move, MoveMeta), MoveError> {
        let (mov, meta) = self.get_move_no_checks(origin, destination, board, color)?;

        assert!(
            board.get(mov.destination(), color).is_none(),
            "pieces should not capture pieces of their own color",
        );

        assert_ne!(
            board.get(mov.destination(), color.other()),
            Some(Piece::King),
            "Should not be able to capture the king. ({mov}) {origin} -> {destination}. {:?} {board}",
            board.get_either(origin),
        );

        let mut test_board = board.clone();
        test_board.apply_move_unchecked(mov, color);

        if let Some(capturing_move) = test_board.in_check(color) {
            return Err(MoveError::KingIsUnprotected { capturing_move });
        }

        Ok((mov, meta))
    }

    /// Gets the move from `origin` to `destination` is legal, except verifying whether it leaves the king in a check.
    pub(crate) fn get_move_no_checks(
        self,
        origin: Vec2,
        destination: Vec2,
        board: &Board,
        color: Color,
    ) -> Result<(Move, MoveMeta), MoveError> {
        assert_eq!(
            board.get(origin, color),
            Some(self),
            "The board needs to have a {self} in {origin}"
        );

        if origin == destination {
            return Err(MoveError::NullMovement);
        }

        let mov = match self {
            Self::Pawn => pawn::get_move(origin, destination, board, color)?,
            Self::Bishop => bishop::get_move(origin, destination, board, color)?,
            Self::Knight => knight::get_move(origin, destination, board, color)?,
            Self::Rook => rook::get_move(origin, destination, board, color)?,
            Self::Queen => queen::get_move(origin, destination, board, color)?,
            Self::King => king::get_move(origin, destination, board, color)?,
        };

        let meta = MoveMeta { color };

        Ok((mov, meta))
    }

    pub fn initial_configuration() -> impl Iterator<Item = (Piece, Vec2, Color)> {
        pawn::initial_configuration()
            .map(|(p, c)| (Piece::Pawn, p, c))
            .chain(knight::initial_configuration().map(|(p, c)| (Piece::Knight, p, c)))
            .chain(bishop::initial_configuration().map(|(p, c)| (Piece::Bishop, p, c)))
            .chain(rook::initial_configuration().map(|(p, c)| (Piece::Rook, p, c)))
            .chain(queen::initial_configuration().map(|(p, c)| (Piece::Queen, p, c)))
            .chain(king::initial_configuration().map(|(p, c)| (Piece::King, p, c)))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("There is no piece to move at {position}")]
    PieceNotPresent { position: Vec2 },

    #[error("The piece at {position} is {color}, you're not allowed to move it")]
    NotYourPiece { position: Vec2, color: Color },

    #[error("The origin and the destination can't be the same tile")]
    NullMovement,

    #[error("{0}")]
    Pawn(#[from] pawn::MoveError),

    #[error("{0}")]
    Bishop(#[from] bishop::MoveError),

    #[error("{0}")]
    Knight(#[from] knight::MoveError),

    #[error("{0}")]
    Rook(#[from] rook::MoveError),

    #[error("{0}")]
    Queen(#[from] queen::MoveError),

    #[error("{0}")]
    King(#[from] king::MoveError),

    #[error("Move leaves king unprotected (could by captured by {capturing_move})")]
    KingIsUnprotected { capturing_move: Move },
}

impl Piece {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Piece::Pawn => "pawn",
            Piece::Knight => "knight",
            Piece::Bishop => "bishop",
            Piece::Rook => "rook",
            Piece::Queen => "queen",
            Piece::King => "king",
        }
    }

    #[must_use]
    pub const fn representing_letter(self) -> char {
        match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        }
    }

    /// The numeric value of the piece.
    ///
    /// Returns [`None`] for [`Piece::King`].
    ///
    /// - Pawns are 1
    /// - Knights and bishops are 3
    /// - Rooks are 5
    /// - Queens is 9
    /// - King is invaluable
    #[must_use]
    pub const fn value(self) -> Option<u8> {
        Some(match self {
            Piece::Pawn => 1,
            Piece::Knight | Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            Piece::King => return None,
        })
    }

    #[must_use]
    pub fn emoji(self, color: Color) -> char {
        color.choose(
            match self {
                Piece::Pawn => '♟',
                Piece::Knight => '♞',
                Piece::Bishop => '♝',
                Piece::Rook => '♜',
                Piece::Queen => '♛',
                Piece::King => '♚',
            },
            match self {
                Piece::Pawn => '♙',
                Piece::Knight => '♘',
                Piece::Bishop => '♗',
                Piece::Rook => '♖',
                Piece::Queen => '♕',
                Piece::King => '♔',
            },
        )
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

impl<T> Index<Piece> for [T; 6] {
    type Output = T;
    fn index(&self, index: Piece) -> &Self::Output {
        // TODO: This could be `get_unchecked`
        &self[index as usize]
    }
}
