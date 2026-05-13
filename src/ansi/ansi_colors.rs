use crate::ansi::{AnsiCode, TextStyle};
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
        AnsiCode::Red => {
            if style.bold {
                BOLD_RED
            } else {
                RED
            }
        }
        AnsiCode::Green => {
            if style.bold {
                BOLD_GREEN
            } else {
                GREEN
            }
        }
        AnsiCode::Yellow => {
            if style.bold {
                BOLD_YELLOW
            } else {
                YELLOW
            }
        }
        AnsiCode::Blue => {
            if style.bold {
                BOLD_BLUE
            } else {
                BLUE
            }
        }
        AnsiCode::Magenta => {
            if style.bold {
                BOLD_MAGENTA
            } else {
                MAGENTA
            }
        }
        AnsiCode::Cyan => {
            if style.bold {
                BOLD_CYAN
            } else {
                CYAN
            }
        }
        AnsiCode::White => {
            if style.bold {
                BOLD_WHITE
            } else {
                WHITE
            }
        }
        _ => WHITE,
    }
}
