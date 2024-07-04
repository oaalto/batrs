use crate::ansi::AnsiCode;
use egui::Color32;

pub const RED: Color32 = Color32::from_rgb(187, 0, 0);
pub const GREEN: Color32 = Color32::from_rgb(0, 187, 0);
pub const YELLOW: Color32 = Color32::from_rgb(187, 187, 0);
pub const BLUE: Color32 = Color32::from_rgb(0, 0, 187);
pub const MAGENTA: Color32 = Color32::from_rgb(187, 0, 187);
pub const CYAN: Color32 = Color32::from_rgb(0, 187, 187);
pub const WHITE: Color32 = Color32::from_rgb(187, 187, 187);

pub const BOLD_RED: Color32 = Color32::from_rgb(255, 85, 85);
pub const BOLD_GREEN: Color32 = Color32::from_rgb(85, 255, 85);
pub const BOLD_YELLOW: Color32 = Color32::from_rgb(255, 255, 85);
pub const BOLD_BLUE: Color32 = Color32::from_rgb(85, 85, 255);
pub const BOLD_MAGENTA: Color32 = Color32::from_rgb(255, 85, 255);
pub const BOLD_CYAN: Color32 = Color32::from_rgb(85, 255, 255);
pub const BOLD_WHITE: Color32 = Color32::from_rgb(255, 255, 255);

pub fn get_color(color: AnsiCode, bold: bool) -> Color32 {
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
