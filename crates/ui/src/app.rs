use egui::{Align, FontData, FontDefinitions, FontFamily, Label, Layout, vec2};
use egui_notify::Toasts;

use crate::{
    ColorScheme,
    color_scheme::ColorSchemeRgba,
    game::{GameScreen, GameScreenEvent},
    main_menu::MainMenu,
};

#[derive(Default, serde::Deserialize, serde::Serialize)]
#[serde(default)]
pub struct App {
    screen: Screen,
    color_scheme: ColorScheme,

    #[serde(skip)]
    main_menu_screen: MainMenu,
    game_screen: Option<GameScreen>,

    #[serde(skip)]
    toasts: egui_notify::Toasts,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, serde::Deserialize, serde::Serialize)]
pub enum Screen {
    #[default]
    MainMenu,
    Options,
    Game,
}

impl App {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // This is also where you can customize the look and feel of egui using
        // `cc.egui_ctx.set_visuals` and `cc.egui_ctx.set_fonts`.
        let color_scheme = ColorScheme::default();
        let color_scheme_rgb = ColorSchemeRgba::from(color_scheme);

        // Colors between egui and shader are different :(
        // assert_eq!(
        //     egui::Rgba::from(color_scheme.background),
        //     ColorSchemeRgba::from(color_scheme).background,
        // );

        // assert_eq!(
        //     Color32::from(color_scheme.background),
        //     Color32::from(color_scheme_rgb.background)
        // );

        // dbg!(
        //     color_scheme_rgb.background,
        //     Color32::from(color_scheme.background)
        // );

        cc.egui_ctx.set_visuals(egui::Visuals {
            menu_corner_radius: egui::CornerRadius::same(0),
            // panel_fill: dbg!(color_scheme.background.into()),
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

        // To load svgs
        egui_extras::install_image_loaders(&cc.egui_ctx);

        // Load previous app state (if any).
        // Note that you must enable the `persistence` feature for this to work.
        if let Some(storage) = cc.storage {
            crate::board::gpu::prepare(cc.wgpu_render_state.as_ref().unwrap());
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }

        Self {
            screen: Screen::default(),
            color_scheme,
            game_screen: None,
            main_menu_screen: MainMenu::default(),
            toasts: Toasts::new(),
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
