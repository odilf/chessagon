#![cfg(test)]

use crate::coordinate::Vec2;
use hext_boards::HexagonalBoard;

/// Returns a string of a diagram where each tile has a character that is based on some property of a board.
pub fn visualize_tile_property<T>(
    property: impl Fn(Vec2) -> T,
    to_char: impl Fn(&T) -> char,
) -> String {
    let hext_board = Vec2::iter()
        .map(|position| {
            (
                [position.x() as i32, position.y() as i32],
                property(position),
            )
        })
        .collect::<HexagonalBoard<_>>();

    hext_board.render_with(to_char)
}

#[allow(missing_docs)]
pub const FILES: &str = include_str!("./files.txt");
#[allow(missing_docs)]
pub const RANKS: &str = include_str!("./ranks.txt");
#[allow(missing_docs)]
pub const RANK_WIDTHS: &str = include_str!("./rank_widths.txt");
#[allow(missing_docs)]
pub const MIN_VALID_RANK_COORDINATES: &str = include_str!("./min_valid_rank_coordinates.txt");
#[allow(missing_docs)]
pub const INDICES: &str = include_str!("./indices.txt");
#[allow(missing_docs)]
pub const INITIAL_BOARD: &str = include_str!("./initial_board.txt");
#[allow(missing_docs)]
pub const MOVEMENT_KNIGHT: &str = include_str!("./movement_knight.txt");
