use egui::WidgetText;

pub fn button<'a>(text: impl Into<WidgetText>) -> egui::Button<'a> {
    egui::Button::new(text).corner_radius(0)
}
