use crate::ansi::{AnsiCode, TextColor, TextStyle};
use ratatui::style::Color;

pub const BLACK: Color = Color::Black;
pub const DARK_GRAY: Color = Color::DarkGray;

pub const DARK_RED: Color = Color::Rgb(128, 0, 0);
pub const DARK_GREEN: Color = Color::Rgb(0, 128, 0);
pub const DARK_BLUE: Color = Color::Rgb(64, 64, 220);

pub const RED: Color = Color::Rgb(187, 0, 0);
pub const GREEN: Color = Color::Rgb(0, 187, 0);
pub const YELLOW: Color = Color::Rgb(187, 187, 0);
pub const BLUE: Color = Color::Rgb(96, 96, 255);
pub const MAGENTA: Color = Color::Rgb(187, 0, 187);
pub const CYAN: Color = Color::Rgb(0, 187, 187);
pub const WHITE: Color = Color::Rgb(187, 187, 187);

pub const BOLD_RED: Color = Color::Rgb(255, 85, 85);
pub const BOLD_GREEN: Color = Color::Rgb(85, 255, 85);
pub const BOLD_YELLOW: Color = Color::Rgb(255, 255, 85);
pub const BOLD_BLUE: Color = Color::Rgb(135, 135, 255);
pub const BOLD_MAGENTA: Color = Color::Rgb(255, 85, 255);
pub const BOLD_CYAN: Color = Color::Rgb(85, 255, 255);
pub const BOLD_WHITE: Color = Color::Rgb(255, 255, 255);

pub const SURFACE: Color = BLACK;
pub const TEXT: Color = WHITE;
pub const SELECTION: Color = DARK_GRAY;

pub fn get_color(style: TextStyle) -> Color {
    match style.color {
        TextColor::Default => {
            if style.bold {
                BOLD_WHITE
            } else {
                TEXT
            }
        }
        TextColor::Indexed(index) => Color::Indexed(index),
        TextColor::Rgb(red, green, blue) => Color::Rgb(red, green, blue),
        TextColor::Ansi(color) => ansi_color(color, style.bold),
    }
}

fn ansi_color(color: AnsiCode, bold: bool) -> Color {
    match color {
        AnsiCode::Black => {
            if bold {
                DARK_GRAY
            } else {
                BLACK
            }
        }
        AnsiCode::Red => {
            if bold {
                BOLD_RED
            } else {
                RED
            }
        }
        AnsiCode::Green => {
            if bold {
                BOLD_GREEN
            } else {
                GREEN
            }
        }
        AnsiCode::Yellow => {
            if bold {
                BOLD_YELLOW
            } else {
                YELLOW
            }
        }
        AnsiCode::Blue => {
            if bold {
                BOLD_BLUE
            } else {
                BLUE
            }
        }
        AnsiCode::Magenta => {
            if bold {
                BOLD_MAGENTA
            } else {
                MAGENTA
            }
        }
        AnsiCode::Cyan => {
            if bold {
                BOLD_CYAN
            } else {
                CYAN
            }
        }
        AnsiCode::White => {
            if bold {
                BOLD_WHITE
            } else {
                WHITE
            }
        }
        _ => WHITE,
    }
}
