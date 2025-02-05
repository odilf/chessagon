use crate::GuiBoard;
use chessagon_core::{
    Color, Game,
    game::{Action, TimeControl},
};
use chessagon_engine::{Engine as _, models::Anthony};
use egui::{Align, Context, Layout, Ui, vec2};
use egui_notify::Toasts;

mod timer;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GameScreen {
    /// The color of the player
    pub color: Color,
    pub game: Game,
    pub gui_board: GuiBoard,
}

pub enum GameScreenEvent {
    Reset,
}

impl GameScreen {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        toasts: &mut Toasts,
    ) -> Option<GameScreenEvent> {
        let mut event = None;
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let panel_size = 200.0;
            let width = ui.available_width();

            ui.allocate_ui_with_layout(
                vec2(panel_size, ui.available_height()),
                Layout::top_down(Align::Center),
                |ui| {
                    ui.label("Playing against Anthony (bot)");
                },
            );

            let mov = ui
                .allocate_ui_with_layout(
                    vec2(width - 2.0 * panel_size, ui.available_height()),
                    Layout::left_to_right(Align::Center),
                    |ui| {
                        self.gui_board
                            .draw(ui, ctx, self.game.board(), self.color, toasts)
                    },
                )
                .inner;

            ui.allocate_ui_with_layout(
                vec2(panel_size, ui.available_height()),
                Layout::top_down(Align::Center),
                |ui| {
                    if let Some(result) = self.game.result() {
                        ui.label(format!("Game finished: {result:?}"));
                        if ui.button("New game").clicked() {
                            event = Some(GameScreenEvent::Reset);
                        }
                    }

                    ui.allocate_ui_with_layout(
                        vec2(ui.available_width(), ui.available_height() / 2.0),
                        Layout::bottom_up(Align::Center),
                        |ui| {
                            timer::draw(ui, ctx, self.game.time_remaining(self.color.other()));
                        },
                    );

                    if ui.button("resign").clicked() {
                        self.game.resign(self.color);
                    }

                    match self.game.draw_offer() {
                        None => ui.button("Offer draw"),
                        Some(c) if c == self.color.other() => ui.button("Accept draw"),
                        Some(_) => {
                            ui.add_enabled_ui(false, |ui| ui.button("Can't offer draw"))
                                .response
                        }
                    };

                    // if self.game.can_accept_draw(self.color) {
                    //     if ui.button("Accept draw").clicked() {
                    //         self.game
                    //             .accept_draw(self.color)
                    //             .expect("We checked we can accept draws");
                    //     }
                    // } else {
                    //     let offer_draw_button = ui.button("Offer draw");
                    //     offer_draw_button.on_disabled_hover_text("You have already offered a draw");

                    //     if ui
                    //         .add_enabled(self.game.draw_offer().is_some(), add_contents)
                    //         .button("Offer draw")
                    //         .clicked()
                    //     {
                    //         self.game.offer_draw(self.color).unwrap();
                    //     }
                    // }

                    timer::draw(ui, ctx, self.game.time_remaining(self.color));
                },
            );

            if let Some(mov) = mov {
                if let Err(err) = self.game.apply_action(Action::Move(mov), self.color) {
                    toasts.warning(err.to_string());
                }
            }

            if self.game.turn() == self.color.other() {
                // TODO: Move this off the main thread
                // TODO: Support more than just playing against the engine
                let mut engine = Anthony::new(self.color.other(), TimeControl::rapid());
                engine.play(&mut self.game).unwrap();
            }
        });

        event
    }

    pub fn new(frame: &mut eframe::Frame) -> Option<Self> {
        Some(Self {
            color: Color::White,
            game: Game::new(TimeControl::rapid()),
            gui_board: GuiBoard::new(frame)?,
        })
    }
}
