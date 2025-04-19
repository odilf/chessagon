use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

use crate::{GuiBoard, components};
use chessagon_core::{
    Color, Game,
    game::{Action, TimeControl},
};
use chessagon_engine::{Engine as _, models::Anthony};
use egui::{Align, Context, Layout, Margin, RichText, Spacing, Ui, Vec2, vec2};

mod timer;

// TODO: Fix this god awful name.
#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub enum GameOrInitGameScreen {
    InitGame { time_control: TimeControl },
    Game(GameScreen),
}
impl GameOrInitGameScreen {
    /// Applies a function if the variant is `Self::Game`.
    pub fn map_game(&mut self, f: impl FnOnce(&mut GameScreen)) {
        match self {
            Self::InitGame { .. } => (),
            Self::Game(game) => f(game),
        }
    }
}

impl Default for GameOrInitGameScreen {
    fn default() -> Self {
        Self::InitGame {
            time_control: TimeControl::blitz(),
        }
    }
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GameScreen {
    /// The color of the player
    pub color: Color,
    pub game: Game,
    pub gui_board: GuiBoard,
    #[serde(skip)]
    pub connection: Option<GameConnection>,
}

#[derive(Debug)]
pub struct GameConnection {
    pub action_sender: Sender<Action>,
    pub opponent_action_receiver: Receiver<Action>,
}

pub enum GameScreenEvent {
    Reset,
}

impl GameScreen {
    fn connect_to_channel(
        &mut self,
        opponent_action_receiver: Receiver<Action>,
    ) -> Receiver<Action> {
        let (sender, receiver) = mpsc::channel();
        self.connection = Some(GameConnection {
            action_sender: sender,
            opponent_action_receiver,
        });

        receiver
    }

    pub fn connect(&mut self) {
        let mut opponent = Anthony::new(self.color.other(), self.game.time_control());

        let (opponent_sender, opponent_receiver) = mpsc::channel();
        let player_receiver = self.connect_to_channel(opponent_receiver);

        {
            let mut game = self.game.clone();
            let player_color = self.color;
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
    }

    /// Creates a new game screen.
    ///
    /// Returns `None` when [`GuiBoard::new`] does (no wgpu render state available).
    pub fn new(frame: &mut eframe::Frame, time_control: TimeControl) -> Option<GameScreen> {
        let game = Game::new(time_control);
        let gui_board = GuiBoard::new(frame, game.board())?;

        let mut output = GameScreen {
            color: Color::White,
            game,
            gui_board,
            connection: None,
        };

        output.connect();

        Some(output)
    }
}

impl GameScreen {
    pub fn draw(&mut self, ui: &mut Ui, ctx: &Context) -> Option<GameScreenEvent> {
        match self
            .connection
            .as_ref()?
            .opponent_action_receiver
            .try_recv()
        {
            Ok(action) => {
                tracing::debug!("got action {action:?} from opponent");
                // TODO: Should we somehow handle invalid actions?
                self.game
                    .apply_action(action, self.color.other())
                    .expect("Action received from opponent should be valid.");

                self.gui_board.update(self.game.board(), self.color, ctx);
            }
            // Opponent hasn't moved yet.
            Err(TryRecvError::Empty) => (),
            Err(TryRecvError::Disconnected) => {
                tracing::warn!("Opponent action receiver disconnected!");
            }
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

            if self.game.is_finished() && button("New game", true).clicked() {
                event = Some(GameScreenEvent::Reset);
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
        self.connection
            .as_ref()
            .unwrap()
            .action_sender
            .send(action)
            .map_err(|err| tracing::error!(?err))
            .expect("TODO: Handle sending errors");

        self.game
            .apply_action(action, self.color)
            .expect("Given action should have been generated validly");
    }
}

/// Draw the game initialization screen where you select the time control.
///
/// Returns whether to start the game.
pub fn draw_init_game_screen(ui: &mut Ui, time_control: &mut TimeControl) -> bool {
    const MAX_WIDTH: f32 = 300.0;
    let margin = ((ui.available_width() - MAX_WIDTH) / 2.0).max(0.0);
    // effective width, in case the screen is smaller than 360.
    let width = ui.available_width() - 2.0 * margin;

    ui.horizontal(|ui| {
        ui.add_space(margin);
        ui.vertical(|ui| {
            ui.add_space(24.0);
            ui.label(RichText::new("New game").strong().size(16.0));
            ui.add_space(8.0);

            ui.label("Select a time control:");

            const SPACING: f32 = 4.0;
            let button_size = width / 3.0 - 2.0 * SPACING;

            let mut tc_button = |ui: &mut egui::Ui, tc| {
                ui.add_enabled_ui(*time_control != tc, |ui| {
                    let button = ui.add_sized(
                        Vec2::splat(button_size),
                        components::button(
                            RichText::new(format!("{}\n({})", tc.formatted(), tc.category()))
                                .size(16.0),
                        ),
                    );

                    if button.clicked() {
                        *time_control = tc;
                    }
                })
            };

            ui.spacing_mut().item_spacing = Vec2::splat(SPACING);
            ui.horizontal(|ui| {
                tc_button(ui, TimeControl::mps(1, 0));
                tc_button(ui, TimeControl::mps(2, 1));
                tc_button(ui, TimeControl::mps(3, 0));
            });
            ui.horizontal(|ui| {
                tc_button(ui, TimeControl::mps(3, 2));
                tc_button(ui, TimeControl::mps(5, 0));
                tc_button(ui, TimeControl::mps(5, 3));
            });
            ui.horizontal(|ui| {
                tc_button(ui, TimeControl::mps(10, 0));
                tc_button(ui, TimeControl::mps(10, 5));
                tc_button(ui, TimeControl::mps(15, 10));
            });
            ui.horizontal(|ui| {
                tc_button(ui, TimeControl::mps(30, 0));
                tc_button(ui, TimeControl::mps(30, 20));
            });

            ui.add_sized(
                Vec2::new(width, 64.0),
                components::button(RichText::new("Start game").strong().size(16.0)),
            )
            .clicked()
        })
        .inner
    })
    .inner
}
