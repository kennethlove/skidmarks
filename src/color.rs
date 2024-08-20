use catppuccin::{self, Flavor};
use ratatui::prelude::*;
static PALETTE: Flavor = catppuccin::PALETTE.mocha;

const fn ansi(color: &catppuccin::Color) -> ansi_term::Color {
    ansi_term::Colour::RGB(color.rgb.r, color.rgb.g, color.rgb.b)
}

const fn rgb(color: &catppuccin::Color) -> Color {
    Color::Rgb(color.rgb.r, color.rgb.g, color.rgb.b)
}

#[derive(Clone, Debug)]
pub struct AppStyles {
    pub background: Color,
    pub foreground: Color,
    pub danger: Color,
    pub row_bg: Color,
    pub alt_row_bg: Color,
    pub row_fg: Color,
    pub alt_row_fg: Color,
    pub highlight_bg: Color,
    pub highlight_fg: Color,
    pub tab_fg: Color,
    pub selected_tab_fg: Color,
    pub response_fg: ansi_term::Color,
    pub response_error_fg: ansi_term::Color,
    pub table_header_fg: ansi_term::Color,
}

impl AppStyles {
    pub fn new() -> Self {
        let peach = rgb(&PALETTE.colors.peach);
        let text = rgb(&PALETTE.colors.text);
        let base = rgb(&PALETTE.colors.base);

        AppStyles {
            background: base,
            foreground: text,
            danger: Color::Rgb(
                PALETTE.colors.red.rgb.r,
                PALETTE.colors.red.rgb.g,
                PALETTE.colors.red.rgb.b,
            ),
            row_bg: Color::Rgb(
                PALETTE.colors.surface0.rgb.r,
                PALETTE.colors.surface0.rgb.g,
                PALETTE.colors.surface0.rgb.b,
            ),
            alt_row_bg: Color::Rgb(
                PALETTE.colors.surface1.rgb.r,
                PALETTE.colors.surface1.rgb.g,
                PALETTE.colors.surface1.rgb.b,
            ),
            row_fg: text,
            alt_row_fg: text,
            highlight_bg: peach,
            highlight_fg: base,
            tab_fg: text,
            selected_tab_fg: peach,
            response_fg: ansi(&PALETTE.colors.peach),
            response_error_fg: ansi(&PALETTE.colors.red),
            table_header_fg: ansi(&PALETTE.colors.peach),
        }
    }
}
