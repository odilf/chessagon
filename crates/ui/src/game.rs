use std::sync::mpsc::{self, Receiver, Sender, TryRecvError};

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
                tracing::info!("Starting engine in other thread");
                loop {
                    if game.turn() == player_color {
                        tracing::debug!("Waiting for player action");
                        let player_action = player_receiver.recv().unwrap();
                        tracing::debug!("Got {player_action:?} from player");
                        game.apply_action(player_action, player_color).unwrap();
                    }

                    tracing::debug!("getting engine action");
                    let action = opponent.get_action(&game);

                    tracing::debug!("Got action {action:?} from engine");
                    opponent_sender.send(action).unwrap();
                    game.apply_action(action, player_color.other()).unwrap();
                }
            });
        }

        output
    }

    pub fn new(frame: &mut eframe::Frame) -> Option<GameScreen> {
        let disconnected = GameScreenDisconnected {
            color: Color::White,
            game: Game::new(TimeControl::rapid()),
            gui_board: GuiBoard::new(frame)?,
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
        toasts: &mut Toasts,
    ) -> Option<GameScreenEvent> {
        match self.opponent_action_receiver.try_recv() {
            Ok(action) => {
                tracing::info!("got action {action:?} from opponent");
                self.game.apply_action(action, self.color.other()).unwrap()
            }
            Err(TryRecvError::Empty) => (),
            Err(err) => panic!("{err}"),
        }

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

                    timer::draw(ui, ctx, self.game.time_remaining(self.color));
                },
            );

            if let Some(mov) = mov {
                self.action_sender.send(Action::Move(mov)).unwrap();

                if let Err(err) = self.game.apply_action(Action::Move(mov), self.color) {
                    toasts.warning(err.to_string());
                }
            }

            // if self.game.turn() == self.color.other() {}
        });

        event
    }
}
