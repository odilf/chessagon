pub mod bishop;
pub mod king;
pub mod knight;
pub mod pawn;
pub mod queen;
pub mod rook;

use core::fmt;

use strum::EnumString;

use crate::{
    board::Board,
    coordinate::Vec2,
    mov::{FullMove, MoveMeta},
    Color,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
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
    ) -> Result<FullMove, MoveError> {
        assert_eq!(
            board.get(origin, color).as_ref(),
            Some(self),
            "The board need to contain `Self` in `origin`"
        );

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

        Ok(FullMove {
            mov,
            meta: MoveMeta { color, checks },
        })
    }

    pub fn enumerate_legal_moves<'b>(
        &self,
        origin: Vec2,
        board: &'b Board,
        color: Color,
    ) -> impl Iterator<Item = FullMove> + use<'_, 'b> {
        // TODO: Implement this more optimally.
        Vec2::iter()
            .filter_map(move |destination| self.get_move(origin, destination, board, color).ok())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum MoveError {
    #[error("There is no piece at {position}")]
    PieceNotPresent { position: Vec2 },

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
    pub fn name(self) -> &'static str {
        match self {
            Piece::Pawn => "pawn",
            Piece::Knight => "knight",
            Piece::Bishop => "bishop",
            Piece::Rook => "rook",
            Piece::Queen => "queen",
            Piece::King => "king",
        }
    }

    pub fn representing_letter(self) -> char {
        match self {
            Piece::Pawn => 'P',
            Piece::Knight => 'N',
            Piece::Bishop => 'B',
            Piece::Rook => 'R',
            Piece::Queen => 'Q',
            Piece::King => 'K',
        }
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
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.name())
    }
}
