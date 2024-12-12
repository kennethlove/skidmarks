use catppuccin::{self, Flavor};
use ratatui::prelude::*;
static PALETTE: Flavor = catppuccin::PALETTE.mocha;

const fn rgb(color: &catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}

#[derive(Clone, Debug)]
pub struct GuiStyles {
    pub header_bg: Color,
    pub header_fg: Color,
    pub background: String,
    pub foreground: String,
}

impl GuiStyles {
    pub fn new() -> Self {
        GuiStyles {
            header_bg: rgb(&PALETTE.colors.peach),
            header_fg: rgb(&PALETTE.colors.surface0),
            background: PALETTE.colors.base.hex.to_string(),
            foreground: PALETTE.colors.text.hex.to_string(),
        }
    }
}
