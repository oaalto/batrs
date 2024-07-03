use crate::ansi::AnsiCode;
use egui::Color32;

pub fn get_color(color: AnsiCode, bold: bool) -> Color32 {
    match color {
        AnsiCode::Red => {
            if bold {
                Color32::RED
            } else {
                Color32::LIGHT_RED
            }
        }
        AnsiCode::Green => {
            if bold {
                Color32::GREEN
            } else {
                Color32::LIGHT_GREEN
            }
        }
        AnsiCode::Yellow => {
            if bold {
                Color32::YELLOW
            } else {
                Color32::LIGHT_YELLOW
            }
        }
        AnsiCode::Blue => {
            if bold {
                Color32::BLUE
            } else {
                Color32::DARK_BLUE
            }
        }
        AnsiCode::Magenta => {
            if bold {
                Color32::from_rgb(187, 0, 187)
            } else {
                Color32::from_rgb(255, 0, 255)
            }
        }
        AnsiCode::Cyan => {
            if bold {
                Color32::from_rgb(85, 255, 255)
            } else {
                Color32::from_rgb(0, 187, 187)
            }
        }
        AnsiCode::White => {
            if bold {
                Color32::WHITE
            } else {
                Color32::from_rgb(187, 187, 187)
            }
        }
        _ => Color32::from_rgb(187, 187, 187),
    }
}
