use std::cmp::Reverse;

use chessagon_core::{Board, Color, Vec2, piece::Piece};
use egui::{Image, Pos2, vec2};

use super::hex_to_uv;

const ICONS: [[egui::ImageSource<'static>; 6]; 2] = [
    [
        egui::include_image!("../../assets/pieces/pawn-white.svg"),
        egui::include_image!("../../assets/pieces/knight-white.svg"),
        egui::include_image!("../../assets/pieces/bishop-white.svg"),
        egui::include_image!("../../assets/pieces/rook-white.svg"),
        egui::include_image!("../../assets/pieces/queen-white.svg"),
        egui::include_image!("../../assets/pieces/king-white.svg"),
    ],
    [
        egui::include_image!("../../assets/pieces/pawn-black.svg"),
        egui::include_image!("../../assets/pieces/knight-black.svg"),
        egui::include_image!("../../assets/pieces/bishop-black.svg"),
        egui::include_image!("../../assets/pieces/rook-black.svg"),
        egui::include_image!("../../assets/pieces/queen-black.svg"),
        egui::include_image!("../../assets/pieces/king-black.svg"),
    ],
];

pub fn icon(piece: Piece, color: Color) -> egui::Image<'static> {
    Image::new(ICONS[color][piece].clone())
}

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
pub struct GuiPiece {
    pub kind: Piece,
    pub color: Color,
    pub hex_tile: Vec2,
    pub position: Pos2,
}

impl GuiPiece {
    pub fn target_position(&self) -> Pos2 {
        hex_to_uv(self.hex_tile)
    }

    pub fn from_board(board: &Board) -> impl Iterator<Item = Self> {
        board
            .all_piece_positions()
            .map(move |(position, piece, color)| GuiPiece {
                kind: piece,
                color,
                hex_tile: position,
                position: hex_to_uv(position),
            })
    }

    // TODO: Maybe we should have a `dt` here, but it's unclear with exponential easing
    /// Moves towards the target.
    ///
    /// Returns whether the piece has moved.
    pub fn move_towards(&mut self, target: Pos2, move_factor: f32) -> bool {
        let delta = target - self.position;
        if delta.length_sq() < 0.0000005_f32.powi(2) {
            return false;
        }

        self.position += delta * move_factor;
        true
    }

    pub fn move_towards_target(&mut self, move_factor: f32) -> bool {
        self.move_towards(self.target_position(), move_factor)
    }

    pub fn update(pieces: &mut Vec<Self>, board: &Board) {
        let mut target_unmatched = Vec::new();
        let mut starting_unmatched = pieces.iter_mut().enumerate().collect::<Vec<_>>();
        for (position, piece, color) in board.all_piece_positions() {
            if let Some(i) = starting_unmatched
                .iter()
                .position(|(_, p)| p.kind == piece && p.color == color && p.hex_tile == position)
            {
                starting_unmatched.swap_remove(i);
            } else {
                target_unmatched.push((position, piece, color));
            }
        }

        let mut remove = Vec::new();
        for (i, piece) in starting_unmatched {
            let candidate = target_unmatched
                .iter()
                .filter(|&&(_, piece_kind, color)| piece.kind == piece_kind && piece.color == color)
                .min_by_key(|(position, _, _)| position.distance(piece.hex_tile));

            if let Some((new_hex_tile, _, _)) = candidate {
                piece.hex_tile = *new_hex_tile;
            } else {
                // TODO: Do some animation when captured.
                remove.push(i);
            }
        }

        for i in remove {
            pieces.remove(i);
        }
    }
}
