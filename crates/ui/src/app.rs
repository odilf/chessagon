use chessagon_core::game::TimeControl;
use egui::{Align, FontFamily, Layout, RichText, Vec2};

use crate::{
    ColorScheme,
    color_scheme::ColorSchemeRgba,
    components,
    game::{self, GameOrInitGameScreen, GameScreen, GameScreenEvent},
    main_menu::MainMenu,
};

#[derive(Default, Debug, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    screen: Screen,
    color_scheme: ColorScheme,

    #[serde(skip)]
    main_menu_screen: MainMenu,
    game_screen: GameOrInitGameScreen,
    // #[serde(skip)]
    // toasts: egui_notify::Toasts,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Screen {
    #[default]
    MainMenu,
    Options,
    Game,
}

impl App {
    fn init(&mut self, cc: &eframe::CreationContext<'_>) {
        // To load svgs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // TODO: Maybe this should be passed by reference.
        App::set_style(cc, self.color_scheme);

        self.game_screen.map_game(|game| game.connect());
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state.
        if let Some(storage) = cc.storage {
            crate::board::gpu::prepare(cc.wgpu_render_state.as_ref().unwrap());

            let mut app: App = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            tracing::info!("Loaded app state from storage.");
            tracing::debug!("Loaded app state is {app:?}");

            app.init(cc);
            return app;
        }

        App::default()
    }

    /// Sets the style for the app.
    fn set_style(cc: &eframe::CreationContext<'_>, color_scheme: ColorScheme) {
        use egui::{CornerRadius, FontData, FontDefinitions, FontFamily, Visuals};

        let color_scheme_rgb = ColorSchemeRgba::from(color_scheme);

        cc.egui_ctx.set_visuals(Visuals {
            menu_corner_radius: CornerRadius::same(0),
            panel_fill: color_scheme_rgb.background.into(),
            ..Default::default()
        });

        cc.egui_ctx.set_fonts({
            let mut fonts = FontDefinitions::default();
            fonts.font_data.insert(
                "AtkinsonHyperlegible".to_string(),
                std::sync::Arc::new(FontData::from_static(include_bytes!(
                    "../assets/font/Atkinson_Hyperlegible/AtkinsonHyperlegible-Regular.ttf"
                ))),
            );

            fonts.font_data.insert(
                "AtkinsonHyperlegible-Bold".to_string(),
                std::sync::Arc::new(FontData::from_static(include_bytes!(
                    "../assets/font/Atkinson_Hyperlegible/AtkinsonHyperlegible-Bold.ttf"
                ))),
            );

            fonts
                .families
                .get_mut(&FontFamily::Proportional)
                .unwrap()
                .insert(0, "AtkinsonHyperlegible".to_string());

            fonts.families.insert(
                FontFamily::Name("bold".into()),
                vec!["AtkinsonHyperlegible-Bold".to_string()],
            );

            fonts
        });
    }
}

// impl<GS: fmt::Debug> fmt::Debug for App<GS> {
//     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
//         f.debug_struct("App")
//             .field("screen", &self.screen)
//             .field("color_scheme", &self.color_scheme)
//             .field("main_menu_screen", &self.main_menu_screen)
//             .field("game_screen", &self.game_screen)
//             .finish_non_exhaustive()
//     }
// }

// impl<GS> Default for App<GS> {
//     fn default() -> Self {
//         Self {
//             screen: Default::default(),
//             color_scheme: Default::default(),
//             main_menu_screen: Default::default(),
//             game_screen: Default::default(),
//             toasts: Default::default(),
//         }
//     }
// }

impl eframe::App for App {
    /// Called by the frame work to save state before shutdown.
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    /// Called each time the UI needs repainting, which may be many times per second.
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopBottomPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::TopBottomPanel::top("top_panel").show(ctx, |ui| {
            // The top panel is often a good place for a menu bar:
            egui::menu::bar(ui, |ui| {
                ui.spacing_mut().button_padding = Vec2::new(12.0, 4.0);
                ui.spacing_mut().item_spacing = Vec2::splat(3.0);

                for (text, screen) in [
                    ("Main menu", Screen::MainMenu),
                    ("Game", Screen::Game),
                    ("Options", Screen::Options),
                ] {
                    let menu_button = ui.add(components::button(
                        RichText::new(text)
                            .font(egui::FontId {
                                size: 14.0,
                                family: FontFamily::Proportional,
                            })
                            .strong(),
                    ));
                    if self.screen == screen {
                        menu_button.highlight();
                    } else if menu_button.clicked() {
                        self.screen = screen
                    }
                }
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.screen {
                Screen::MainMenu => {
                    ui.with_layout(
                        Layout::top_down(Align::Center).with_cross_align(Align::Center),
                        |ui| {
                            if let Some(switch_to) = self.main_menu_screen.draw(ui) {
                                self.screen = switch_to;
                            }
                        },
                    );
                }

                Screen::Options => {
                    ui.label("Work in progress.");
                }
                Screen::Game => match &mut self.game_screen {
                    GameOrInitGameScreen::Game(game_screen) => {
                        let event = game_screen.draw(ui, ctx);
                        match event {
                            None => (),
                            Some(GameScreenEvent::Reset) => {
                                self.game_screen = GameOrInitGameScreen::default()
                            }
                        }
                    }
                    GameOrInitGameScreen::InitGame { time_control } => {
                        if game::draw_init_game_screen(ui, time_control) {
                            // TODO: Maybe we shouldn't unwrap here.
                            self.game_screen = GameOrInitGameScreen::Game(
                                GameScreen::new(frame, *time_control).unwrap(),
                            )
                        }
                    }
                },
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });

            // self.toasts.show(ctx);
        });
    }
}
