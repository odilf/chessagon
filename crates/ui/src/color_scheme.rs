use bytemuck::{Pod, Zeroable};
use egui::{Rgba, ecolor, epaint::HsvaGamma};

#[derive(Debug, Clone, Copy, serde::Deserialize, serde::Serialize)]
#[serde(from = "ColorSchemeRgba", into = "ColorSchemeRgba")]
pub struct ColorScheme {
    pub tiles: [HsvaGamma; 3],
    pub background: HsvaGamma,
    pub selected_tile: HsvaGamma,
    pub highlighted_tile: HsvaGamma,
}

impl ColorScheme {
    pub fn purple() -> Self {
        Self {
            tiles: [
                HsvaGamma {
                    h: 0.7,
                    s: 0.55,
                    v: 0.50,
                    a: 1.0,
                },
                HsvaGamma {
                    h: 0.7,
                    s: 0.55,
                    v: 0.65,
                    a: 1.0,
                },
                HsvaGamma {
                    h: 0.7,
                    s: 0.55,
                    v: 0.80,
                    a: 1.0,
                },
            ],
            background: HsvaGamma {
                h: 0.6,
                s: 0.6,
                v: 0.1,
                a: 1.0,
            },
            selected_tile: HsvaGamma {
                h: 0.70,
                s: 0.9,
                v: 0.5,
                a: 0.8,
            },
            highlighted_tile: HsvaGamma {
                // h: 0.99,
                h: 0.85,
                s: 0.9,
                v: 0.4,
                // v: 0.5,
                a: 0.6,
            },
        }
    }
}

impl Default for ColorScheme {
    fn default() -> Self {
        Self::purple()
    }
}

#[repr(C)]
#[derive(Debug, Clone, Copy, Zeroable, Pod, serde::Deserialize, serde::Serialize, PartialEq)]
pub struct ColorSchemeRgba {
    pub tiles: [Rgba; 3],
    pub background: Rgba,
    pub selected_tile: Rgba,
    pub highlighted_tile: Rgba,
}

impl ColorScheme {
    // TODO: The fact this is needed is kind of dodgy.
    pub(crate) fn into_gamma_rgba(self) -> ColorSchemeRgba {
        fn from(mut hsva: HsvaGamma) -> Rgba {
            // TODO: This is extremely dodgy. I'm basically "taking away" the alpha for the conversions
            // and then putting it back in, but I'm sure this is somehow somewhat wrong, right?
            let alpha = hsva.a;
            hsva.a = 1.0;

            let rgba = Rgba::from(hsva);
            Rgba::from_rgba_premultiplied(
                ecolor::gamma_from_linear(rgba.r()),
                ecolor::gamma_from_linear(rgba.g()),
                ecolor::gamma_from_linear(rgba.b()),
                alpha,
            )
        }

        ColorSchemeRgba {
            tiles: self.tiles.map(from),
            background: from(self.background),
            selected_tile: from(self.selected_tile),
            highlighted_tile: from(self.highlighted_tile),
        }
    }
}

impl From<ColorScheme> for ColorSchemeRgba {
    fn from(value: ColorScheme) -> Self {
        Self {
            tiles: value.tiles.map(|c| c.into()),
            background: value.background.into(),
            selected_tile: value.selected_tile.into(),
            highlighted_tile: value.highlighted_tile.into(),
        }
    }
}

impl From<ColorSchemeRgba> for ColorScheme {
    fn from(value: ColorSchemeRgba) -> Self {
        Self {
            tiles: value.tiles.map(|c| c.into()),
            background: value.background.into(),
            selected_tile: value.selected_tile.into(),
            highlighted_tile: value.highlighted_tile.into(),
        }
    }
}
