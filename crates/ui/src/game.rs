use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use crate::{GuiBoard, components};
use chessagon_core::{
    Color, Game,
    game::{Action, TimeControl},
};
use chessagon_engine::{Engine as _, models::Anthony};
use egui::{Align, Button, Context, Layout, Margin, RichText, Spacing, Ui, Vec2, vec2};
use egui_notify::Toasts;

mod timer;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GameScreen<S = Sender<Action>, R = Receiver<Action>> {
    /// The color of the player
    pub color: Color,
    pub game: Game,
    pub gui_board: GuiBoard,

    /// Send your actions.
    #[serde(skip)]
    pub action_sender: S,

    /// Receive opponent's actions.
    #[serde(skip)]
    pub opponent_action_receiver: R,
}

// TODO: Maybe it's just easier to have two distinct types...
pub type GameScreenDisconnected = GameScreen<(), ()>;

pub enum GameScreenEvent {
    Reset,
}

impl GameScreenDisconnected {
    fn connect_to_channel(
        self,
        opponent_action_receiver: Receiver<Action>,
    ) -> (GameScreen, Receiver<Action>) {
        let (sender, receiver) = mpsc::channel();

        (
            GameScreen {
                action_sender: sender,
                opponent_action_receiver,

                color: self.color,
                game: self.game,
                gui_board: self.gui_board,
            },
            receiver,
        )
    }

    pub fn connect(self) -> GameScreen {
        let mut opponent = Anthony::new(self.color.other(), self.game.time_control());

        let (opponent_sender, opponent_receiver) = mpsc::channel();
        let (output, player_receiver) = self.connect_to_channel(opponent_receiver);

        {
            let mut game = output.game.clone();
            let player_color = output.color;
            std::thread::spawn(move || {
                // TODO: This has clearly too many unwraps
                let span = tracing::info_span!("Opponent engine");
                let _guard = span.enter();

                tracing::info!("Starting engine in other thread");
                loop {
                    if game.is_finished() {
                        return;
                    }

                    if game.turn() == player_color {
                        tracing::debug!("Waiting for player action");
                        let player_action = player_receiver.recv().unwrap();

                        tracing::debug!("Got {player_action:?} from player");
                        game.apply_action(player_action, player_color).unwrap();
                    } else {
                        tracing::debug!("getting engine action");
                        let action = opponent.get_action(&game);

                        tracing::debug!(?action);
                        opponent_sender.send(action).unwrap();
                        game.apply_action(action, player_color.other()).unwrap();
                    }
                }
            });
        }

        output
    }

    pub fn new(frame: &mut eframe::Frame) -> Option<GameScreen> {
        let game = Game::new(TimeControl::rapid());
        let gui_board = GuiBoard::new(frame, game.board())?;

        let disconnected = GameScreenDisconnected {
            color: Color::White,
            game,
            gui_board,
            action_sender: (),
            opponent_action_receiver: (),
        };

        Some(disconnected.connect())
    }
}

impl GameScreen {
    pub fn draw(
        &mut self,
        ui: &mut Ui,
        ctx: &Context,
        _toasts: &mut Toasts,
    ) -> Option<GameScreenEvent> {
        match self.opponent_action_receiver.try_recv() {
            Ok(action) => {
                tracing::debug!("got action {action:?} from opponent");
                // TODO: Should we somehow handle invalid actions?
                self.game
                    .apply_action(action, self.color.other())
                    .expect("Action received from opponent should be valid.");

                self.gui_board.update(self.game.board(), self.color, ctx);
            }
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {}
        }

        let mut event = None;
        ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
            let panel_size = 200.0;
            let max_board_width = 600.0;
            let width = ui.available_width();
            let padding = (width - (panel_size + max_board_width)).max(0.0) / 2.0;
            let board_width = width - panel_size - 2.0 * padding;

            ui.allocate_space(vec2(padding, ui.available_height()));

            let mov = ui
                .allocate_ui_with_layout(
                    vec2(board_width, ui.available_height()),
                    Layout::left_to_right(Align::Center),
                    |ui| self.gui_board.draw(ui, ctx, self.game.board(), self.color),
                )
                .inner;

            ui.allocate_ui_with_layout(
                vec2(panel_size, ui.available_height()),
                Layout::top_down(Align::Center),
                |ui| {
                    event = self.draw_sidebar(ui, ctx);
                },
            );

            if let Some(mov) = mov {
                self.apply_action(Action::Move(mov));
            }

            ui.allocate_space(vec2(padding, ui.available_height()));
        });

        event
    }

    pub fn draw_sidebar(&mut self, ui: &mut Ui, ctx: &Context) -> Option<GameScreenEvent> {
        let mut event = None;

        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), ui.available_height() / 2.0),
            Layout::bottom_up(Align::Center),
            |ui| {
                timer::draw(ui, ctx, self.game.time_remaining(self.color.other()));
                if self.game.winner() == Some(Some(self.color.other())) {
                    ui.label("Opponent (winner)");
                } else if self.game.winner() == Some(Some(self.color)) {
                    ui.label("Opponent (loser)");
                } else if self.game.winner() == Some(None) {
                    ui.label("Opponent (draw)");
                } else {
                    ui.label("Opponent");
                }
            },
        );

        ui.style_mut().spacing = Spacing {
            button_padding: Vec2::splat(12.0),
            menu_margin: Margin::ZERO,
            ..Default::default()
        };

        ui.horizontal(|ui| {
            let in_progress = !self.game.is_finished();
            let mut button = |text, always_enabled| {
                ui.add_enabled(
                    in_progress || always_enabled,
                    components::button(RichText::new(text).size(12.0)),
                )
            };

            if button("Resign", false).clicked() {
                self.apply_action(Action::Resign);
            }

            match self.game.draw_offer() {
                None => {
                    if button("Offer draw", false).clicked() {
                        self.apply_action(Action::OfferDraw);
                    }
                }
                Some(color) if color == self.color => {
                    if button("Retract draw", false).clicked() {
                        self.apply_action(Action::RetractDraw);
                    }
                }
                Some(_) => {
                    if button("Accept draw", false).clicked() {
                        self.apply_action(Action::AcceptDraw);
                    }
                }
            }

            if self.game.is_finished() {
                if button("New game", true).clicked() {
                    event = Some(GameScreenEvent::Reset);
                }
            }
        });

        ui.allocate_ui_with_layout(
            vec2(ui.available_width(), ui.available_height() / 2.0),
            Layout::top_down(Align::Center),
            |ui| {
                timer::draw(ui, ctx, self.game.time_remaining(self.color));
                if self.game.winner() == Some(Some(self.color)) {
                    ui.label("You (winner)");
                } else if self.game.winner() == Some(Some(self.color.other())) {
                    ui.label("You (loser)");
                } else if self.game.winner() == Some(None) {
                    ui.label("You (draw)");
                } else {
                    ui.label("You");
                }
            },
        );

        event
    }

    /// Applies a valid action from the player while sending it to the sender.
    ///
    /// # Panics
    ///
    /// If the action is invalid.
    pub fn apply_action(&mut self, action: Action) {
        // TODO: Handle sending error more gracefully
        self.action_sender
            .send(action)
            .map_err(|err| tracing::error!(?err))
            .expect("TODO: Handle sending errors");

        self.game
            .apply_action(action, self.color)
            .expect("Given action should have been generated validly");
    }
}
