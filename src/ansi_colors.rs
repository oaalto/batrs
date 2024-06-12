use crate::ansi_codes::AnsiCodes;

pub const RED: iced::Color = iced::Color::from_rgb(0.73, 0f32, 0f32);
pub const GREEN: iced::Color = iced::Color::from_rgb(0f32, 0.73, 0f32);
pub const YELLOW: iced::Color = iced::Color::from_rgb(0.73, 0.73, 0f32);
pub const BLUE: iced::Color = iced::Color::from_rgb(0f32, 0f32, 0.73);
pub const MAGENTA: iced::Color = iced::Color::from_rgb(0.73, 0f32, 0.73);
pub const CYAN: iced::Color = iced::Color::from_rgb(0f32, 0.73, 0.73);
pub const WHITE: iced::Color = iced::Color::from_rgb(0.73, 0.73, 0.73);

pub const BOLD_RED: iced::Color = iced::Color::from_rgb(1.0, 0.33, 0.33);
pub const BOLD_GREEN: iced::Color = iced::Color::from_rgb(0.33, 1.0, 0.33);
pub const BOLD_YELLOW: iced::Color = iced::Color::from_rgb(1.0, 1.0, 0.33);
pub const BOLD_BLUE: iced::Color = iced::Color::from_rgb(0.33, 0.33, 1.0);
pub const BOLD_MAGENTA: iced::Color = iced::Color::from_rgb(1.0, 0.33, 1.0);
pub const BOLD_CYAN: iced::Color = iced::Color::from_rgb(0.33, 1.0, 1.0);
pub const BOLD_WHITE: iced::Color = iced::Color::from_rgb(1.0, 1.0, 1.0);

#[derive(Debug, Clone)]
pub enum AnsiColors {
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
}

impl AnsiColors {
    pub fn from_ansi_code(code: &AnsiCodes) -> Self {
        match code {
            AnsiCodes::Red => AnsiColors::Red,
            AnsiCodes::Green => AnsiColors::Green,
            AnsiCodes::Yellow => AnsiColors::Yellow,
            AnsiCodes::Blue => AnsiColors::Blue,
            AnsiCodes::Magenta => AnsiColors::Magenta,
            AnsiCodes::Cyan => AnsiColors::Cyan,
            AnsiCodes::White => AnsiColors::White,
            _ => AnsiColors::White,
        }
    }
}

pub fn get_color(color: &AnsiColors, bold: bool) -> iced::Color {
    match color {
        AnsiColors::Red => {
            if bold {
                BOLD_RED
            } else {
                RED
            }
        }
        AnsiColors::Green => {
            if bold {
                BOLD_GREEN
            } else {
                GREEN
            }
        }
        AnsiColors::Yellow => {
            if bold {
                BOLD_YELLOW
            } else {
                YELLOW
            }
        }
        AnsiColors::Blue => {
            if bold {
                BOLD_BLUE
            } else {
                BLUE
            }
        }
        AnsiColors::Magenta => {
            if bold {
                BOLD_MAGENTA
            } else {
                MAGENTA
            }
        }
        AnsiColors::Cyan => {
            if bold {
                BOLD_CYAN
            } else {
                CYAN
            }
        }
        AnsiColors::White => {
            if bold {
                BOLD_WHITE
            } else {
                WHITE
            }
        }
        _ => WHITE,
    }
}
