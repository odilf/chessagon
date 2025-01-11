use crate::{
    coordinate::Vec2,
    mov::{FullMove, Move},
    piece::{MoveError, Piece},
    Color,
};

#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[Option<Piece>; 91]; 2],

    /// Wether or not the king of each side can castle or not.
    castling_rights: [bool; 2],

    /// The last move, if it was a pawn move. Needed for en-passant.
    last_move: Option<Move>,
}

impl Default for Board {
    fn default() -> Self {
        todo!()
    }
}

impl Board {
    /// The maximum absolute value difference that a set of coordinates can have.
    pub const SIZE: u8 = 5;

    pub const NUMBER_OF_TILES: u8 = 91;

    /// Returns the index where the position is stored in the array.
    #[inline]
    fn index(position: Vec2) -> usize {
        let rank = position.rank();
        let tiles_before_rank = (0..rank).map(Vec2::rank_width).sum::<u8>();

        // Tiles on the same rank are those where `x + y == p.x + p.y`, so each tile in a rank can be
        // characterized by `p.x` or `p.y`. `p.y` is nicer because 0->n goes left to right.
        //
        // So can we just take `p.y`? No, because 0 is not always a valid option for y. So we need
        // to find the first valid y value.
        let first_valid_y = Vec2::min_valid_rank_coordinate(rank);
        let index_on_rank = position.y - first_valid_y;

        (tiles_before_rank + index_on_rank) as usize
    }

    /// Gets the piece at the specified position, if it's white.
    #[inline]
    pub fn get_white(&self, position: Vec2) -> Option<Piece> {
        self.pieces[Color::White][Board::index(position)]
    }

    /// Gets the piece at the specified position, if it's black.
    #[inline]
    pub fn get_black(&self, position: Vec2) -> Option<Piece> {
        self.pieces[Color::Black][Board::index(position)]
    }

    /// Gets the piece at the specified position, if it's the given color.
    #[inline]
    pub fn get(&self, position: Vec2, color: Color) -> Option<Piece> {
        self.pieces[color][Board::index(position)]
    }

    /// Gets the piece with its color at the specified position.
    #[inline]
    pub fn get_either(&self, position: Vec2) -> Option<(Piece, Color)> {
        self.get_white(position)
            .map(|piece| (piece, Color::White))
            .or(self.get_black(position).map(|piece| (piece, Color::Black)))
    }

    // /// Returns a mutable reference to the piece at the specified position.
    // #[inline]
    // pub fn get_mut(&mut self, position: Vec2) -> &mut Option<Piece> {
    //     &mut self.pieces[Board::index(position)]
    // }

    /// Makes the specified move. Doesn't check for any legality
    pub fn apply_move_unchecked(&mut self, mov: Move, color: Color) -> () {
        match mov {
            Move::Regular {
                origin,
                destination,
                captures,
            } => {
                if captures {
                    let took_piece = self.pieces[color.other()][Board::index(destination)].take();
                    debug_assert!(took_piece.is_some())
                }

                self
                .pieces
                .swap(Board::index(origin), Board::index(destination))
            }

            Move::Castle { color, side } => todo!(),
            Move::EnPassant { file } => todo!(),
            Move::Promotion { file, captures } => todo!(),
        }
    }

    pub fn get_move(&self, origin: Vec2, destination: Vec2) -> Result<FullMove, MoveError> {
        let Some((piece, color)) = self.get_either(origin) else {
            return Err(MoveError::PieceNotPresent { position: origin });
        };

        piece.get_move(origin, destination, self, color)
    }

    pub fn try_move(&mut self, origin: Vec2, destination: Vec2) -> Result<(), MoveError> {
        let mov = self.get_move(origin, destination)?;
        self.apply_move_unchecked(mov.mov, mov.meta.color);
        Ok(())
    }

    /// Whether `color` still has the right to castle.
    pub fn castling_right(&self, color: Color) -> bool {
        self.castling_rights[color]
    }
}

#[cfg(test)]
mod tests {
    use crate::{board::Board, coordinate::Vec2, diagrams};
    use pretty_assertions::assert_eq;
    use std::collections::HashSet;

    #[test]
    fn number_of_tiles_const_matches_computation() {
        assert_eq!(Vec2::iter().count(), Board::NUMBER_OF_TILES as usize);
    }

    #[test]
    fn index_is_dense_and_unique() {
        let mut indices = (0..Board::NUMBER_OF_TILES as usize).collect::<HashSet<_>>();

        for position in Vec2::iter() {
            let index = Board::index(position);
            assert!(
                indices.remove(&index),
                "{index} is counted twice (by {position:?})"
            );
        }

        assert!(indices.is_empty(), "Leftover: {indices:?}");
    }

    #[test]
    fn indices_match_diagram() {
        let rendered = diagrams::visualize_tile_property(
            |position| Board::index(position),
            |width| char::from_digit(*width as u32 % 36, 36).unwrap(),
        );

        assert_eq!(rendered.trim(), diagrams::INDICES.trim());
    }

    // #[test]
    // fn fn_vec2_iter_matches_indices_order() {
    //     // for (i, position) in Vec2::iter().enumerate() {
    //     //     assert_eq!(Board::index(position), i);
    //     // }
    //     let rendered = diagrams::visualize_tile_property(
    //         |position| Vec2::iter().enumerate().find(|(_, p)| *p == position).unwrap().0,
    //         |width| char::from_digit(*width as u32 % 36, 36).unwrap(),
    //     );
    //
    //     panic!("{rendered}");
    // }
}
