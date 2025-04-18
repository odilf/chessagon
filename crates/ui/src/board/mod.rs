mod piece;
mod wgpu;

// For when I add new backends
pub mod gpu {
    pub use super::wgpu::prepare;
}

use std::time::SystemTime;

use bytemuck::{Pod, Zeroable};
use chessagon_core::{Board, Color, Move, Vec2};
use eframe::egui_wgpu;
use egui::{Pos2, Rect, Ui, pos2, vec2};
use piece::GuiPiece;
use wgpu::CustomBoardCallback;

use crate::{ColorScheme, color_scheme::ColorSchemeRgba};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GuiBoard {
    /// The selected tile, and whether it's being dragged.
    selected_tile: Option<(Vec2, bool)>,
    highlighted_tiles: Vec<Vec2>,

    /// A vector of pieces, current position
    pieces: Vec<GuiPiece>,

    /// How fast do pieces move in general, when not dragging
    piece_move_speed: f32,

    /// How fast do pieces move towards cursor when dragging.
    piece_drag_speed: f32,

    /// The time the last click happened, used for passing in to uniforms, for gpu animations.
    last_click_time: SystemTime,

    // To check whether keydown is click or hold.
    #[serde(skip)]
    pointer_pressed_last_frame: bool,

    #[serde(skip)]
    uniforms: Uniforms,
}

impl Default for GuiBoard {
    fn default() -> Self {
        Self {
            selected_tile: None,
            highlighted_tiles: Vec::new(),
            pieces: Vec::new(),
            piece_move_speed: 0.2,
            piece_drag_speed: 0.8,
            last_click_time: SystemTime::now(),
            pointer_pressed_last_frame: false,
            uniforms: Uniforms::default(),
        }
    }
}

impl GuiBoard {
    pub fn new<'a>(frame: &eframe::Frame, board: &Board) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = frame.wgpu_render_state.as_ref()?;

        wgpu::prepare(wgpu_render_state);

        Some(Self {
            uniforms: Uniforms::default(),
            pieces: GuiPiece::from_board(board).collect(),
            ..Default::default()
        })
    }

    /// Updates the board. Currently, this entails:
    /// - Updating the selection.
    pub fn update(&mut self, board: &Board, color: Color, ctx: &egui::Context) {
        if let Some((selected_tile, _)) = self.selected_tile {
            self.select(&board, selected_tile, color, ctx);
        }
        ctx.request_repaint();
    }
}

/// The apothem of a hexagon in `uv` coordinates.
const APOTHEM: f32 = {
    let hexagon_height = 1.0 / Board::NUMBER_OF_RANKS as f32;
    hexagon_height / 2.0
};

const POSITION_00: egui::Vec2 = vec2(0.5, 1.0 - APOTHEM);

/// Goes from a chessagon position to a uv (0.0 - 1.0) position.
fn hex_to_uv(hex: Vec2) -> Pos2 {
    let basis_1 = vec2(-f32::sqrt(3.0), -1.0);
    let basis_2 = vec2(f32::sqrt(3.0), -1.0);

    let unscaled = hex.x() as f32 * basis_1 + hex.y() as f32 * basis_2;
    let uncentered = unscaled * APOTHEM;

    let centered = uncentered + POSITION_00;

    centered.to_pos2()
}

fn uv_to_hex(uv: Pos2) -> Option<Vec2> {
    // [x, y] denotes hex, (x, y) denotes screen
    // [0, 0] is at origin = (0.5, 1.0 - APOTHEM)
    // delta of [1, 0] is (-step_size.x, step_size.y)
    // delta of [0, 1] is (step_size.x, step_size.y)
    // So [x, y] is at origin + ((-x + y) * step_size.x, (x + y) * step_size.y);
    //
    // Then,
    // - pointer_pos.x = origin.x + (-x + y) * step_size.x
    // - pointer_pos.y = origin.y +  (x + y) * step_size.y
    //
    // Solving for x and y
    // => `2y = (pointer_pos.x - origin.x) / step_size.x + (pointer_pos.y - origin.y) / step_size.y` same for `x`
    let step_size = vec2(f32::sqrt(3.0), -1.0) * APOTHEM;
    let n_x = ((uv.x - POSITION_00.x) / step_size.x).round() as i8;
    let n_y = ((uv.y - POSITION_00.y) / step_size.y).round() as i8;

    let y = n_x.wrapping_add(n_y) / 2;
    let x = y.wrapping_sub(n_x);

    Vec2::new(x as u8, y as u8)
}

fn uv_to_screen(uv: Pos2, rect: Rect) -> Pos2 {
    pos2(
        rect.min.x + uv.x * rect.size().x,
        rect.min.y + uv.y * rect.size().y,
    )
}

fn screen_to_uv(screen_position: Pos2, rect: Rect) -> Pos2 {
    let centered = screen_position - rect.min;
    pos2(centered.x / rect.size().x, centered.y / rect.size().y)
}

impl GuiBoard {
    pub fn toggle_selection(
        &mut self,
        board: &Board,
        color: Color,
        uv: Pos2,
        ctx: &egui::Context,
    ) -> Option<Move> {
        let Some(position) = uv_to_hex(uv) else {
            self.deselect();
            return None;
        };

        if board.get(position, color).is_some() {
            self.select(board, position, color, ctx);
            return None;
        }

        let (selected_tile, _) = self.selected_tile?;

        if board.get(position, color).is_some() && position != selected_tile {
            self.select(board, position, color, ctx);
        } else {
            self.deselect();
        }

        board
            .get_move(selected_tile, position, color)
            .ok()
            .map(|(mov, _meta)| mov)
    }

    pub fn select(&mut self, board: &Board, position: Vec2, color: Color, ctx: &egui::Context) {
        self.selected_tile = Some((position, ctx.is_using_pointer()));
        self.highlighted_tiles = board
            .possible_moves(color)
            .filter(|mov| mov.origin() == position)
            .map(|mov| mov.destination())
            .collect();

        self.last_click_time = SystemTime::now();
    }

    pub fn deselect(&mut self) {
        self.selected_tile = None;
        self.highlighted_tiles = Vec::new();
    }

    pub fn draw(
        &mut self,
        ui: &mut egui::Ui,
        ctx: &egui::Context,
        board: &Board,
        color: Color,
    ) -> Option<Move> {
        let size = ui.available_width().min(ui.available_height());
        let (id, rect) = ui.allocate_space(egui::Vec2::splat(size));
        let response = ui.interact(rect, id, egui::Sense::click_and_drag());

        self.draw_board(ui, rect);
        self.draw_pieces(ui, ctx, board, size, rect);

        let mov = (|| {
            let pointer_position = ctx.pointer_latest_pos()?;
            let uv = screen_to_uv(pointer_position, rect);
            let hex = uv_to_hex(uv)?;

            let piece_there = board.get(hex, color).is_some();
            let is_move_target = self.highlighted_tiles.contains(&hex);

            if piece_there || is_move_target {
                ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            let clicking = response.is_pointer_button_down_on();
            let clicked_last_frame = self.pointer_pressed_last_frame;
            self.pointer_pressed_last_frame = clicking;

            if clicking && !clicked_last_frame {
                return self.toggle_selection(board, color, uv, ctx);
            }

            if let (Some((selected_tile, true)), Some(pointer_pos)) =
                (&mut self.selected_tile, ctx.pointer_interact_pos())
            {
                let selected_piece = self
                    .pieces
                    .iter_mut()
                    .find(|p| p.hex_tile == *selected_tile)
                    .expect("There should be a piece in the selected position");

                selected_piece.move_towards(screen_to_uv(pointer_pos, rect), self.piece_drag_speed);
            }

            if !clicking {
                if let Some((selected_tile, dragging)) = &mut self.selected_tile {
                    if *dragging {
                        *dragging = false;
                    }

                    let selected_piece = self
                        .pieces
                        .iter()
                        .find(|p| p.hex_tile == *selected_tile)
                        .expect("There should be a piece in the selected position");

                    if Some(*selected_tile) != uv_to_hex(selected_piece.position) {
                        // drop the picked up piece
                        tracing::debug!("Dropping piece at {}", selected_piece.position);
                        return self.toggle_selection(board, color, selected_piece.position, ctx);
                    }
                };
            }

            None
        })();

        mov
    }

    pub fn draw_board(&mut self, ui: &mut Ui, rect: Rect) {
        self.uniforms.tile_flags = <[TileFlags; 92]>::zeroed();
        if let Some((selected_tile, _dragging_piece)) = self.selected_tile {
            *self.uniforms.get_flag(selected_tile) |= TileFlags::SELECTED;
        }

        for &highlighted in &self.highlighted_tiles {
            *self.uniforms.get_flag(highlighted) |= TileFlags::HIGHLIGHTED;
        }

        // TODO: Get this from configuration
        self.uniforms.color_scheme = ColorScheme::purple().into_gamma_rgba();

        self.uniforms.time_since_last_click = SystemTime::now()
            .duration_since(self.last_click_time)
            .expect("`last_clicked_time` should be before now.")
            .as_secs_f32();

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomBoardCallback {
                uniforms: self.uniforms,
            },
        ));
    }

    pub fn draw_pieces(
        &mut self,
        ui: &mut Ui,
        ctx: &egui::Context,
        board: &Board,
        size: f32,
        rect: Rect,
    ) {
        // TODO: Don't update gui pieces on every frame
        piece::GuiPiece::update(&mut self.pieces, board);

        let hexagon_height = size / Board::NUMBER_OF_RANKS as f32;
        let piece_size = egui::Vec2::splat(hexagon_height * 0.85);

        for piece in &mut self.pieces {
            if Some((piece.hex_tile, true)) != self.selected_tile {
                let moved = piece.move_towards_target(self.piece_move_speed);
                if moved {
                    ctx.request_repaint();
                }
            }

            let origin = uv_to_screen(piece.position, rect);
            let piece_rect = Rect::from_center_size(origin, piece_size);

            ui.put(piece_rect, piece::icon(piece.kind, piece.color));
        }
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    // TODO: The color scheme should maybe be in different uniforms, because we don't need to update them
    // on every draw.
    color_scheme: ColorSchemeRgba,
    tile_flags: [TileFlags; 92],
    time_since_last_click: f32,
    _padding: [f32; 3],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default().into(),
            tile_flags: <[TileFlags; 92]>::zeroed(),
            time_since_last_click: 0.0,
            _padding: <[f32; 3]>::zeroed(),
        }
    }
}

impl Uniforms {
    fn get_flag(&mut self, position: Vec2) -> &mut TileFlags {
        // TODO: This could eventually be `get_unchecked`.
        &mut self.tile_flags[Board::index(position)]
    }
}

bitflags::bitflags! {
    #[repr(transparent)]
    #[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, Zeroable, Pod, serde::Serialize, serde::Deserialize)]
    pub struct TileFlags: u32 {
        const SELECTED = (1 << 0);
        const HIGHLIGHTED = (1 << 1);
    }
}
