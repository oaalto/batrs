use crate::ansi_codes::AnsiCode;

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

pub fn get_color(color: &AnsiCode, bold: bool) -> iced::Color {
    match color {
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
