use std::fmt;

use egui::{Align, Layout};
use egui_notify::Toasts;

use crate::{
    ColorScheme,
    color_scheme::ColorSchemeRgba,
    game::{GameScreen, GameScreenDisconnected, GameScreenEvent},
    main_menu::MainMenu,
};

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App<GS = GameScreen> {
    screen: Screen,
    color_scheme: ColorScheme,

    #[serde(skip)]
    main_menu_screen: MainMenu,
    game_screen: Option<GS>,

    #[serde(skip)]
    toasts: egui_notify::Toasts,
}

type AppUninit = App<GameScreenDisconnected>;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Screen {
    #[default]
    MainMenu,
    Options,
    Game,
}

impl AppUninit {
    fn init(self, cc: &eframe::CreationContext<'_>) -> App {
        // To load svgs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // TODO: Maybe this should be passed by reference.
        App::set_style(cc, self.color_scheme);

        App {
            game_screen: self.game_screen.map(|gs| gs.connect()),

            screen: self.screen,
            color_scheme: self.color_scheme,
            main_menu_screen: self.main_menu_screen,
            toasts: self.toasts,
        }
    }
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Load previous app state.
        if let Some(storage) = cc.storage {
            crate::board::gpu::prepare(cc.wgpu_render_state.as_ref().unwrap());

            let app: AppUninit = eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();

            tracing::info!("Loaded app state from storage.");
            tracing::debug!("Loaded app state is {app:?}");

            return app.init(cc);
        }

        AppUninit {
            screen: Screen::default(),
            color_scheme: ColorScheme::purple(),
            game_screen: None,
            main_menu_screen: MainMenu::default(),
            toasts: Toasts::new(),
        }
        .init(cc)
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

impl<GS: fmt::Debug> fmt::Debug for App<GS> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("App")
            .field("screen", &self.screen)
            .field("color_scheme", &self.color_scheme)
            .field("main_menu_screen", &self.main_menu_screen)
            .field("game_screen", &self.game_screen)
            .finish_non_exhaustive()
    }
}

impl<GS> Default for App<GS> {
    fn default() -> Self {
        Self {
            screen: Default::default(),
            color_scheme: Default::default(),
            main_menu_screen: Default::default(),
            game_screen: Default::default(),
            toasts: Default::default(),
        }
    }
}

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
                // NOTE: no File -> Quit on web pages
                let is_web = cfg!(target_arch = "wasm32");
                if !is_web {
                    ui.menu_button("File", |ui| {
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                    ui.add_space(16.0);
                }

                for (text, screen) in [
                    ("Main menu", Screen::MainMenu),
                    ("Game", Screen::Game),
                    ("Options", Screen::Options),
                ] {
                    let menu_button = ui.button(text);
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
                Screen::Game => {
                    if let Some(game_screen) = &mut self.game_screen {
                        let event = game_screen.draw(ui, ctx, &mut self.toasts);
                        match event {
                            None => (),
                            Some(GameScreenEvent::Reset) => self.game_screen = None,
                        }
                    } else {
                        ui.label("y no game??");

                        if ui.button("Start game").clicked() {
                            // TODO: Remove `unwrap`
                            self.game_screen = Some(GameScreen::new(frame).unwrap())
                        }
                    }
                }
            }

            ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                egui::warn_if_debug_build(ui);
            });

            self.toasts.show(ctx);
        });
    }
}
