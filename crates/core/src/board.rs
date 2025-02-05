use crate::{
    Color,
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
///     - There is exactly one king of each color.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Board {
    #[cfg_attr(feature = "serde", serde(with = "serde_piece_nested_array"))]
    pieces: [[Option<Piece>; 91]; 2],
    last_move: Option<Move>,
}

impl Default for Board {
    fn default() -> Self {
        let mut output = Board {
            pieces: [[None; 91]; 2],
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

    #[allow(missing_docs)]
    pub const NUMBER_OF_TILES: u8 = 91;

    /// The total number of [files](`Vec2::file`).
    pub const NUMBER_OF_FILES: u8 = 11;

    /// The total number of [ranks](`Vec2::rank`).
    pub const NUMBER_OF_RANKS: u8 = 11;

    /// Creates a new board with the minimal number of pieces (i.e, two kings).
    // TODO: This should maybe return an error because some positions could be impossible to reach normally.
    pub fn new_minimal(white_king_position: Vec2, black_king_position: Vec2) -> Self {
        let mut output = Self {
            pieces: [[None; Self::NUMBER_OF_TILES as usize]; 2],
            last_move: None,
        };

        output.pieces[Color::White][Board::index(white_king_position)] = Some(Piece::King);
        output.pieces[Color::Black][Board::index(black_king_position)] = Some(Piece::King);

        output
    }

    /// Returns the index where the position is stored in the array.
    ///
    /// See also [`Self::index_to_vec`]
    pub fn index(position: Vec2) -> usize {
        let rank = position.rank();
        let tiles_before_rank = (0..rank).map(Vec2::rank_width).sum::<u8>();

        // Tiles on the same rank are those where `x + y == p.x + p.y`, so each tile in a rank can be
        // characterized by `p.x` or `p.y`. `p.y` is nicer because 0->n goes left to right.
        //
        // So can we just take `p.y`? No, because 0 is not always a valid option for y. So we need
        // to find the first valid y value.
        let first_valid_y = Vec2::min_valid_rank_coordinate(rank);
        let index_on_rank = position.y() - first_valid_y;

        (tiles_before_rank + index_on_rank) as usize
    }

    /// Returns the position that would result into the given index.
    ///
    /// See also [`Self::index`]
    pub fn index_to_vec(index: usize) -> Vec2 {
        // Find the rank, using the fact that `tiles_before_rank` should be less than `index`
        let mut rank = 0;
        let mut tiles_before_rank = 0;
        while tiles_before_rank + Vec2::rank_width(rank) <= index as u8 {
            tiles_before_rank += Vec2::rank_width(rank);
            rank += 1;
        }

        // Find the index of the position in the rank, add the min valid coordinate so that index 0 goes to the min
        let rank_position = index as u8 - tiles_before_rank;
        let y = rank_position + Vec2::min_valid_rank_coordinate(rank);

        // `rank == x + y`, so:
        let x = rank - y;
        Vec2::new_unchecked(x, y)
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

    /// Enumerates all pieces of the given color, without their positions.
    ///
    /// See also [`Self::piece_positions`].
    pub fn pieces(&self, color: Color) -> impl Iterator<Item = Piece> {
        self.pieces[color].iter().copied().flatten()
    }

    /// Enumerates all `(position, piece)` pairs of the given color.
    ///
    /// See also [`Self::all_piece_positions`] and [`Self::pieces`].
    pub fn piece_positions(&self, color: Color) -> impl Iterator<Item = (Vec2, Piece)> {
        // TODO: This could be `get_unchecked`
        self.pieces[color]
            .iter()
            .enumerate()
            .filter_map(|(i, &piece)| Some((Board::index_to_vec(i), piece?)))
    }

    /// Enumerates the positions of all pieces as a `(position, piece, color)` triplet.
    ///
    /// See also [`Self::piece_positions`].
    pub fn all_piece_positions(&self) -> impl Iterator<Item = (Vec2, Piece, Color)> {
        self.piece_positions(Color::White)
            .map(|(pos, piece)| (pos, piece, Color::White))
            .chain(
                self.piece_positions(Color::Black)
                    .map(|(pos, piece)| (pos, piece, Color::Black)),
            )
    }

    /// Verifies whether the given move is legal or not.
    pub fn check_move(&self, mov: Move, color: Color) -> Result<(), MoveError> {
        // TODO: Make this use direct logic instead of reusing `get_move`
        self.get_move(mov.origin(), mov.destination(), color)?;
        Ok(())
    }

    /// Applies the given move.
    ///
    /// See also [`Self::apply_move_unchecked`] to skip verifying if the move is legal if it is known to
    /// be from construction (i.e., from [`Self::possible_moves`]).
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

            Move::EnPassant { .. } => todo!(),
            Move::Promotion { .. } => todo!(),
        };

        self.last_move = Some(mov);
        capture
    }

    /// Gets the move from `origin` to `destination`, if the it is legal.
    ///
    /// Most of it is delegated to [`Piece::get_move`].
    pub fn get_move(
        &self,
        origin: Vec2,
        destination: Vec2,
        color: Color,
    ) -> Result<(Move, MoveMeta), MoveError> {
        let Some((piece, board_piece_color)) = self.get_either(origin) else {
            return Err(MoveError::PieceNotPresent { position: origin });
        };

        if board_piece_color != color {
            return Err(MoveError::NotYourPiece {
                position: origin,
                color: board_piece_color,
            });
        }

        piece.get_move(origin, destination, self, color)
    }

    /// Tries to apply a move from `origin` to `destination`.
    pub fn try_move(
        &mut self,
        origin: Vec2,
        destination: Vec2,
        color: Color,
    ) -> Result<(), MoveError> {
        let (mov, meta) = self.get_move(origin, destination, color)?;
        self.apply_move_unchecked(mov, meta.color);
        Ok(())
    }

    // pub fn undo_move_unchecked(&mut self, _mov: Move) -> Result<(), ()> {
    //     todo!()
    // }

    /// An iterator over all legal moves in the current for position that the player of the given color can do.
    pub fn possible_moves(&self, color: Color) -> impl Iterator<Item = Move> {
        Vec2::iter()
            .map(move |origin| {
                Vec2::iter().filter_map(move |destination| {
                    self.get_move(origin, destination, color)
                        .ok()
                        .map(|(mov, _)| mov)
                })
            })
            .flatten()
    }

    /// The sum of the [`Piece::value`]s of the pieces of the given color.
    pub fn total_piece_value(&self, color: Color) -> u16 {
        self.pieces(color)
            .map(|piece| piece.value().unwrap_or(0) as u16)
            .sum()
    }

    /// Returns the position of the king of the given color.
    pub fn find_king(&self, color: Color) -> Vec2 {
        for (index, &piece) in self.pieces[color].iter().enumerate() {
            if piece == Some(Piece::King) {
                return Self::index_to_vec(index);
            }
        }

        unreachable!("Boards should always have at least one king of each color");
    }

    /// Verifies whether the king of the given color could be attacked next move.
    ///
    /// If it is, returns a move that would capture the king.
    pub fn in_check(&self, color: Color) -> Option<Move> {
        let king_position = self.find_king(color);
        Vec2::iter()
            .filter_map(|origin| {
                self.get(origin, color.other()).and_then(|piece| {
                    piece
                        .get_move_no_checks(origin, king_position, self, color.other())
                        .ok()
                })
            })
            .map(|(mov, _)| mov)
            .next()
    }
}

impl std::fmt::Display for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        use hext_boards::HexagonalBoard;

        let hex_board: HexagonalBoard<_> = Vec2::iter()
            .map(|position| {
                let vec = [position.x() as i32, position.y() as i32];
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
    fn vec_to_index_to_vec_is_identity() {
        for position in Vec2::iter() {
            let index = Board::index(position);
            let new_position = Board::index_to_vec(index);

            assert_eq!(position, new_position);
        }
    }

    #[test]
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

// Huge workaround for lack of const generics in serde...
// Can't easily even use `serde_arrays` or `serde_with` since the array is nested and that turns
// out to be a huge pain in the ass.
#[cfg(feature = "serde")]
mod serde_piece_nested_array {
    use super::Piece;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    #[derive(Serialize, Deserialize)]
    struct SerializableArray<T, const N: usize>
    where
        T: Serialize,
        T: for<'d> Deserialize<'d>,
    {
        #[serde(with = "serde_arrays")]
        inner: [T; N],
    }

    type Arr = SerializableArray<SerializableArray<Option<Piece>, 91>, 2>;

    pub fn serialize<S>(data: &[[Option<Piece>; 91]; 2], ser: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let data: Arr = SerializableArray {
            inner: [
                SerializableArray { inner: data[0] },
                SerializableArray { inner: data[1] },
            ],
        };

        data.serialize(ser)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[[Option<Piece>; 91]; 2], D::Error>
    where
        D: Deserializer<'de>,
    {
        let data = Arr::deserialize(deserializer)?;

        let white = data.inner[0].inner;
        let black = data.inner[1].inner;

        Ok([white, black])
    }
}
