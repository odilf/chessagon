#![cfg(test)]

use crate::coordinate::Vec2;
use hext_boards::HexagonalBoard;

pub fn visualize_tile_property<T>(
    property: impl Fn(Vec2) -> T,
    to_char: impl Fn(&T) -> char,
) -> String {
    let hext_board = Vec2::iter()
        .map(|position| ([position.x as i32, position.y as i32], property(position)))
        .collect::<HexagonalBoard<_>>();

    hext_board.render_with(to_char)
}

pub const FILES: &str = include_str!("./files.txt");
pub const RANKS: &str = include_str!("./ranks.txt");
pub const RANK_WIDTHS: &str = include_str!("./rank_widths.txt");
pub const MIN_VALID_RANK_COORDINATES: &str = include_str!("./min_valid_rank_coordinates.txt");
pub const INDICES: &str = include_str!("./indices.txt");
pub const INITIAL_BOARD: &str = include_str!("./initial_board.txt");
