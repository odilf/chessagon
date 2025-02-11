use egui::{Align, FontFamily, Label, Layout, RichText, vec2};

use crate::app::Screen;

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

        if ui.button("Play against computer").clicked() {
            return Some(Screen::Game);
        }

        None
    }
}
