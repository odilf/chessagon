use crate::{
    Color, Side,
    coordinate::Vec2,
    mov::{Move, MoveMeta},
    piece::{MoveError, Piece},
};

/// A hexagonal chess board.
///
/// Could also call it a chessagonal board.
///
/// # Invariants
/// - The board is always in a valid state. This implies:
///     - If [`Self::has_moved_rook`] is false, the rook must be at it's original position.
#[derive(Debug, Clone)]
pub struct Board {
    pieces: [[Option<Piece>; 91]; 2],

    /// Wether or not the king of each side can castle or not.
    has_moved_rook: [[bool; 2]; 2],

    /// The last move, if it was a pawn move. Needed for en-passant.
    last_move: Option<Move>,
}

impl Default for Board {
    fn default() -> Self {
        let mut output = Board {
            pieces: [[None; 91]; 2],
            has_moved_rook: [[false; 2]; 2],
            last_move: None,
        };

        for (piece, position, color) in Piece::initial_configuration() {
            output.get_mut(position, color).replace(piece);
        }

        output
    }
}

impl Board {
    /// The maximum absolute value difference that a set of coordinates can have.
    pub const SIZE: u8 = 5;

    pub const NUMBER_OF_TILES: u8 = 91;
    pub const NUMBER_OF_FILES: u8 = 11;

    pub fn new_minimal(white_king_position: Vec2, black_king_position: Vec2) -> Self {
        let mut output = Self {
            pieces: [[None; Self::NUMBER_OF_TILES as usize]; 2],
            has_moved_rook: [[false; 2]; 2],
            last_move: None,
        };

        output.pieces[Color::White][Board::index(white_king_position)] = Some(Piece::King);
        output.pieces[Color::Black][Board::index(black_king_position)] = Some(Piece::King);

        output
    }

    /// Returns the index where the position is stored in the array.
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

    /// Returns the position that would result into the given index.
    fn index_to_vec(index: usize) -> Vec2 {
        todo!("index to vec. This might not be necessary?")
    }

    /// Gets the piece at the specified position, if it's white.
    #[inline]
    pub fn get_white(&self, position: Vec2) -> Option<Piece> {
        // TODO: This could be `get_unchecked`
        self.pieces[Color::White][Board::index(position)]
    }

    /// Gets the piece at the specified position, if it's black.
    #[inline]
    pub fn get_black(&self, position: Vec2) -> Option<Piece> {
        // TODO: This could be `get_unchecked`
        self.pieces[Color::Black][Board::index(position)]
    }

    /// Gets the piece at the specified position, if it's of the given color.
    #[inline]
    pub fn get(&self, position: Vec2, color: Color) -> Option<Piece> {
        // TODO: This could be `get_unchecked`
        self.pieces[color][Board::index(position)]
    }

    /// Gets a mutable reference to the piece at the specified position, if it's of the given color.
    #[inline]
    pub fn get_mut(&mut self, position: Vec2, color: Color) -> &mut Option<Piece> {
        // TODO: This could be `get_unchecked`
        &mut self.pieces[color][Board::index(position)]
    }

    /// Gets the piece with its color at the specified position.
    #[inline]
    pub fn get_either(&self, position: Vec2) -> Option<(Piece, Color)> {
        self.get_white(position)
            .map(|piece| (piece, Color::White))
            .or(self.get_black(position).map(|piece| (piece, Color::Black)))
    }

    /// Whether `color` still has the right to castle.
    ///
    /// Note: Moving the rook back to it's initial position does not make the rook count as not
    /// having moved. If it moves at any point, it is consider to have been moved.
    ///
    /// See [`Self::reset_moved_rook`] to reset the moved rook status.
    #[inline]
    pub fn has_moved_rook(&self, color: Color, side: Side) -> bool {
        self.has_moved_rook[color][side]
    }

    /// Resets the rook position and the status of having been moved.
    ///
    /// In other words, it makes [`Self::has_moved_rook`] return false.
    ///
    /// Returns `true` if the value was modified, `false` otherwise
    pub fn reset_moved_rook(&mut self, color: Color, side: Side) -> bool {
        let modified = self.has_moved_rook[color][side] == true;
        self.has_moved_rook[color][side] = false;

        todo!("Reset rook position");

        modified
    }

    pub fn pieces(&self, color: Color) -> impl Iterator<Item = Piece> {
        self.pieces[color].iter().copied().filter_map(|piece| piece)
    }

    pub fn piece_positions(&self, color: Color) -> impl Iterator<Item = (Vec2, Piece)> {
        Vec2::iter().zip(self.pieces(color))
    }

    pub fn check_move(&self, mov: Move, color: Color) -> Result<(), MoveError> {
        // TODO: Make this use direct logic instead of reusing `get_move`
        self.get_move_from_color(mov.origin(color), mov.destination(color), color)?;
        Ok(())
    }

    pub fn apply_move(&mut self, mov: Move, color: Color) -> Result<Option<Piece>, MoveError> {
        self.check_move(mov, color)?;
        Ok(self.apply_move_unchecked(mov, color))
    }

    /// Makes the specified move. Doesn't check for any legality. May leave the board in an inconsistent state.
    ///
    /// Returns the captured piece, if any.
    ///
    /// This method can only be used if `mov` is obtained from an enumeration of moves, or if [`Self::check_move`] has been called with the given moves. To apply checked moves, see [`Self::apply_move`]
    pub fn apply_move_unchecked(&mut self, mov: Move, color: Color) -> Option<Piece> {
        let capture = match mov {
            Move::Regular {
                origin,
                destination,
                captures,
            } => {
                let capture = captures.then(|| {
                    self.get_mut(destination, color.other())
                        .take()
                        // TODO: Maybe `captures isn't necessary then?`
                        .expect(
                            "There should be a piece in the destination if the move is a capture",
                        )
                });

                self.pieces[color].swap(Board::index(origin), Board::index(destination));

                capture
            }

            Move::EnPassant { file, direction } => todo!(),
            Move::Promotion {
                file,
                captures,
                promoting_to,
            } => todo!(),
        };

        self.last_move = Some(mov);
        capture
    }

    pub fn get_move(&self, origin: Vec2, destination: Vec2) -> Result<(Move, MoveMeta), MoveError> {
        let Some((piece, color)) = self.get_either(origin) else {
            return Err(MoveError::PieceNotPresent { position: origin });
        };

        piece.get_move(origin, destination, self, color)
    }

    pub fn get_move_from_color(
        &self,
        origin: Vec2,
        destination: Vec2,
        color: Color,
    ) -> Result<(Move, MoveMeta), MoveError> {
        let Some((piece, color_of_target)) = self.get_either(origin) else {
            return Err(MoveError::PieceNotPresent { position: origin });
        };

        if color != color_of_target {
            return Err(MoveError::NotYourPiece {
                position: origin,
                color: color_of_target,
            });
        }

        piece.get_move(origin, destination, self, color_of_target)
    }

    pub fn try_move(&mut self, origin: Vec2, destination: Vec2) -> Result<(), MoveError> {
        let (mov, meta) = self.get_move(origin, destination)?;
        self.apply_move_unchecked(mov, meta.color);
        Ok(())
    }

    pub fn try_undo_move(&mut self, mov: Move) -> Result<(), ()> {
        todo!()
    }

    pub fn enumerate_moves(&self, color: Color) -> impl Iterator<Item = Move> {
        Vec2::iter()
            .map(move |origin| {
                Vec2::iter().filter_map(move |destination| {
                    self.get_move_from_color(origin, destination, color)
                        .ok()
                        .map(|(mov, _)| mov)
                })
            })
            .flatten()
    }

    pub fn total_piece_value(&self, color: Color) -> u16 {
        self.pieces(color)
            .map(|piece| piece.value().unwrap_or(0) as u16)
            .sum()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use hext_boards::HexagonalBoard;

        // TODO: `hext_boards` should provide traits to implement rendering directly. I.e., [`HexagonalBoard`] should be a trait
        let hex_board: HexagonalBoard<_> = self
            .piece_positions(Color::White)
            .map(|(p, piece)| (p, piece, Color::White))
            .chain(
                self.piece_positions(Color::Black)
                    .map(|(p, piece)| (p, piece, Color::Black)),
            )
            .map(|(p, piece, color)| ([p.x as i32, p.y as i32], (piece, color)))
            .collect();

        let hex_board: HexagonalBoard<_> = Vec2::iter()
            .map(|position| {
                let vec = [position.x as i32, position.y as i32];
                let val = self.get_either(position);

                (vec, val)
            })
            .collect();

        f.write_str(
            &hex_board
                .render_with(|val| val.map(|(piece, color)| piece.emoji(color)).unwrap_or(' ')),
        )
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
    fn number_of_files_const_matches_computation() {
        assert_eq!(
            Vec2::iter()
                .map(|pos| pos.file())
                .collect::<HashSet<_>>()
                .len(),
            Board::NUMBER_OF_FILES as usize
        );
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

    #[test]
    #[ignore = "might not implement this"]
    fn vec_to_index_to_vec_is_identity() {
        for position in Vec2::iter() {
            let index = Board::index(position);
            let new_position = Board::index_to_vec(index);

            assert_eq!(position, new_position);
        }
    }

    #[test]
    #[ignore = "might not implement this"]
    fn index_to_vec_to_index_is_identity() {
        for index in 0..Board::NUMBER_OF_TILES as usize {
            let vec = Board::index_to_vec(index);
            let new_index = Board::index(vec);

            assert_eq!(index, new_index);
        }
    }

    #[test]
    fn intial_board_matches_diagram() {
        let rendered = Board::default().to_string();
        assert_eq!(rendered.trim(), diagrams::INITIAL_BOARD.trim());
    }
}
