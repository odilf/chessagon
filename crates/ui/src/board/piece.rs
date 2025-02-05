use chessagon_core::{Color, piece::Piece};
use egui::Image;

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
