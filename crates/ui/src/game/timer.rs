use std::time::Duration;

use egui::{Align, Context, FontFamily, Layout, RichText, Ui};

pub fn draw(ui: &mut Ui, ctx: &Context, duration: Duration) {
    let millis = duration.subsec_millis();
    let secs = duration.as_secs() % 60;
    let mins = duration.as_secs() / 60;

    let main = RichText::new(format!("{mins:0>2}:{secs:0>2}"))
        .size(32.0)
        .family(FontFamily::Monospace);

    let extra = RichText::new(format!(":{millis:0>3}"))
        .size(16.0)
        .family(FontFamily::Monospace);

    ui.horizontal(|ui| {
        ui.label(main);
        ui.label(extra);
    });

    // TODO: Request repaints only after refresh rate of timer.
    if !duration.is_zero() {
        ctx.request_repaint();
    }
}
