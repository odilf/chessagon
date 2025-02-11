mod piece;
mod wgpu;

// For when I add new backends
pub mod gpu {
    pub use super::wgpu::prepare;
}

use bytemuck::{Pod, Zeroable};
use chessagon_core::{Board, Color, Move, Vec2};
use eframe::egui_wgpu;
use egui::{Rect, vec2};
use egui_notify::Toasts;
use wgpu::CustomBoardCallback;

use crate::{ColorScheme, color_scheme::ColorSchemeRgba};

#[derive(Debug, Default, serde::Deserialize, serde::Serialize)]
pub struct GuiBoard {
    selected_tile: Option<Vec2>,
    highlighted_tiles: Vec<Vec2>,

    #[serde(skip)]
    uniforms: Uniforms,
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod)]
struct Uniforms {
    // TODO: The color scheme should maybe be in different uniforms, because we don't need to update them
    // on every draw.
    color_scheme: ColorSchemeRgba,
    tile_flags: [TileFlags; 92],
}

impl Default for Uniforms {
    fn default() -> Self {
        Self {
            color_scheme: ColorScheme::default().into(),
            tile_flags: <[TileFlags; 92]>::zeroed(),
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

impl GuiBoard {
    pub fn new<'a>(frame: &eframe::Frame) -> Option<Self> {
        // Get the WGPU render state from the eframe creation context. This can also be retrieved
        // from `eframe::Frame` when you don't have a `CreationContext` available.
        let wgpu_render_state = frame.wgpu_render_state.as_ref()?;

        wgpu::prepare(wgpu_render_state);

        Some(Self {
            uniforms: Uniforms {
                color_scheme: ColorScheme::default().into(),
                tile_flags: <[TileFlags; 92]>::zeroed(),
            },
            selected_tile: None,
            highlighted_tiles: Vec::new(),
        })
    }
}

fn apothem(size: f32) -> f32 {
    let hexagon_height = size / Board::NUMBER_OF_RANKS as f32;
    hexagon_height / 2.0
}

/// Goes from a chessagon position to a screen position
fn hex_to_screen(hex: Vec2, rect: Rect) -> egui::Pos2 {
    let basis_1 = vec2(-f32::sqrt(3.0), -1.0);
    let basis_2 = vec2(f32::sqrt(3.0), -1.0);

    // debug_assert_eq!(rect.size().x, rect.size().y);
    let size = rect.size().x;

    let unscaled = hex.x() as f32 * basis_1 + hex.y() as f32 * basis_2;
    let uncentered = unscaled * apothem(size);

    let position_of_00 = vec2((rect.min.x + rect.max.x) / 2.0, rect.max.y - apothem(size));
    let centered = uncentered + position_of_00;

    centered.to_pos2()
}

/// Goes from a screen position to a chessagon position
fn screen_to_hex(pointer_pos: egui::Pos2, rect: Rect) -> Option<Vec2> {
    // [x, y] denotes hex, (x, y) denotes screen
    // [0, 0] is at origin = ((rect.min.x + rect.max.x) / 2.0, rect.max.y - apothem)
    // delta of [1, 0] is (-step_size.x, step_size.y)
    // delta of [0, 1] is (step_size.x, step_size.y)
    // So [x, y] is at origin + ((-x + y) * step_size.x, (x + y) * step_size.y);
    //
    // Then,
    // - pointer_pos.x = origin.x + (-x + y) * step_size.x
    // - pointer_pos.y = origin.y +  (x + y) * step_size.y
    //
    // Solving for x and y
    // => 2y = (pointer_pos.x - origin.x) / step_size.x + (pointer_pos.y - origin.y) / step_size.y
    //
    debug_assert_eq!(rect.size().x, rect.size().y);

    let size = rect.size().x;
    let apothem = apothem(size);
    let origin = vec2((rect.min.x + rect.max.x) / 2.0, rect.max.y - apothem);

    let step_size = vec2(f32::sqrt(3.0), -1.0) * apothem;
    let n_x = ((pointer_pos.x - origin.x) / step_size.x).round() as i8;
    let n_y = ((pointer_pos.y - origin.y) / step_size.y).round() as i8;

    let y = (n_x + n_y) / 2;
    let x = y.wrapping_sub(n_x);

    Vec2::new(x as u8, y as u8)
}

impl GuiBoard {
    pub fn select(&mut self, board: &Board, position: Vec2, color: Color) {
        self.selected_tile = Some(position);
        self.highlighted_tiles = board
            .possible_moves(color)
            .filter(|mov| mov.origin() == position)
            .map(|mov| mov.destination())
            .collect();
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
        toasts: &mut Toasts,
    ) -> Option<Move> {
        let size = ui.available_width().min(ui.available_height());
        let (id, rect) = ui.allocate_space(egui::Vec2::splat(size));
        let response = ui.interact(rect, id, egui::Sense::click());

        // let (rect, response) = ui.allocate_at_least(egui::Vec2::splat(size), egui::Sense::click());

        let mut mov = None;
        // TODO: When does this return `None`?
        if let Some(pointer_position) = ctx.pointer_latest_pos() {
            if let Some(position) = screen_to_hex(pointer_position, rect) {
                // TODO: This should be an enum.
                let piece_there = board.get(position, color).is_some();
                // Is it a target for a move?
                let is_target = self.highlighted_tiles.contains(&position);

                if piece_there || is_target {
                    ctx.set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                if response.clicked() {
                    if piece_there {
                        self.select(board, position, color);
                    } else if let Some(origin) = self.selected_tile {
                        match board.get_move(origin, position, color) {
                            Ok((m, _meta)) => {
                                self.deselect();
                                mov = Some(m);
                            }
                            Err(err) => {
                                toasts.warning(err.to_string());
                                self.deselect();
                            }
                        };
                    } else {
                        self.deselect();
                    }
                }
            }
        } else {
        }
        self.uniforms.tile_flags = <[TileFlags; 92]>::zeroed();
        if let Some(selected_tile) = self.selected_tile {
            *self.uniforms.get_flag(selected_tile) |= TileFlags::SELECTED;
        }

        for &highlighted in &self.highlighted_tiles {
            *self.uniforms.get_flag(highlighted) |= TileFlags::HIGHLIGHTED;
        }

        // TODO: Get this from configuration
        self.uniforms.color_scheme = ColorScheme::purple().into_gamma_rgba();

        ui.painter().add(egui_wgpu::Callback::new_paint_callback(
            rect,
            CustomBoardCallback {
                uniforms: self.uniforms,
            },
        ));

        let hexagon_height = size / Board::NUMBER_OF_RANKS as f32;

        for (position, piece, color) in board.all_piece_positions() {
            let origin = hex_to_screen(position, rect);
            let piece_rect =
                Rect::from_center_size(origin, egui::Vec2::splat(hexagon_height * 0.8));
            ui.put(piece_rect, piece::icon(piece, color));
        }

        mov
    }
}
