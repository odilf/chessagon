pub mod bishop;
pub mod king;
pub mod knight;
pub mod movement;
pub mod pawn;
pub mod queen;
pub mod rook;

use core::fmt;

use strum::EnumString;

use crate::{
    Color,
    board::Board,
    coordinate::Vec2,
    mov::{Move, MoveMeta},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString, Hash)]
pub enum Piece {
    Pawn,
    Knight,
    Bishop,
    Rook,
    Queen,
    King,
}

impl Piece {
    /// TODO: Docs
    ///
    /// # Panics
    ///
    /// - If the piece at the `origin` position is not `self` with the given color.
    pub fn get_move(
        &self,
        origin: Vec2,
        destination: Vec2,
        board: &Board,
        color: Color,
    ) -> Result<(Move, MoveMeta), MoveError> {
        assert_eq!(
            board.get(origin, color).as_ref(),
            Some(self),
            "The board need to contain `self` in `origin`"
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

        // TODO: Implement checking for checks
        let checks = None;
        let meta = MoveMeta { color, checks };

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

    #[error("Can't move pawn")]
    Pawn(#[from] pawn::MoveError),

    #[error("Can't move bishop")]
    Bishop(#[from] bishop::MoveError),

    #[error("Can't move knight")]
    Knight(#[from] knight::MoveError),

    #[error("Can't move rook")]
    Rook(#[from] rook::MoveError),

    #[error("Can't move queen")]
    Queen(#[from] queen::MoveError),

    #[error("Can't move king")]
    King(#[from] king::MoveError),
}

impl Piece {
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
    pub const fn value(self) -> Option<u8> {
        Some(match self {
            Piece::Pawn => 1,
            Piece::Knight => 3,
            Piece::Bishop => 3,
            Piece::Rook => 5,
            Piece::Queen => 9,
            Piece::King => return None,
        })
    }

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
