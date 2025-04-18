use eframe::wgpu::CompilationInfo;
use egui::{Align, FontFamily, Label, Layout, RichText, Vec2, vec2};

use crate::{app::Screen, components};

#[derive(Debug, Default)]
pub struct MainMenu {
    // decoration_tile_angle: f32,
}

impl MainMenu {
    pub fn draw(&self, ui: &mut egui::Ui) -> Option<Screen> {
        let label = Label::new(
            RichText::new("Chessagon")
                .font(egui::FontId {
                    size: 92.0,
                    family: FontFamily::Name("bold".into()),
                })
                .strong(),
        );

        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), ui.available_height() / 2.0),
            Layout::bottom_up(Align::Center),
            |ui| ui.add(label),
        );

        ui.spacing_mut().button_padding = Vec2::splat(12.0);
        if ui
            .add(components::button(
                RichText::new("Play against computer").size(16.0).strong(),
            ))
            .clicked()
        {
            return Some(Screen::Game);
        }

        ui.spacing_mut().button_padding = Vec2::splat(8.0);
        ui.add(components::button("Options"));
        ui.add(components::button("How to play"));

        None
    }
}
