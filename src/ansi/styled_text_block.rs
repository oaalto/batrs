use crate::ansi::{AnsiCode, TextColor};
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct StyledChar {
    pub bold: bool,
    pub color: TextColor,
    pub character: String,
}

impl StyledChar {
    pub fn new(c: String) -> Self {
        Self {
            bold: false,
            color: TextColor::Default,
            character: c,
        }
    }

    pub fn reset(&mut self) {
        self.bold = false;
        self.color = TextColor::Default;
    }

    pub fn process_sgr_codes(&mut self, codes: &[u8]) {
        if codes.is_empty() {
            self.reset();
            return;
        }

        let mut index = 0;
        while index < codes.len() {
            match codes[index] {
                0 => self.reset(),
                1 => self.bold = true,
                22 => self.bold = false,
                30..=37 => {
                    if let Some(color) = ansi_color_from_offset(codes[index] - 30) {
                        self.color = TextColor::Ansi(color);
                    }
                }
                38 => {
                    if let Some((color, consumed)) = extended_color(&codes[index..]) {
                        self.color = color;
                        index += consumed - 1;
                    }
                }
                39 => self.color = TextColor::Default,
                90..=97 => {
                    if let Some(color) = ansi_color_from_offset(codes[index] - 90) {
                        self.color = TextColor::Ansi(color);
                        self.bold = true;
                    }
                }
                _ => {}
            }
            index += 1;
        }
    }
}

fn ansi_color_from_offset(offset: u8) -> Option<AnsiCode> {
    match offset {
        0 => Some(AnsiCode::Black),
        1 => Some(AnsiCode::Red),
        2 => Some(AnsiCode::Green),
        3 => Some(AnsiCode::Yellow),
        4 => Some(AnsiCode::Blue),
        5 => Some(AnsiCode::Magenta),
        6 => Some(AnsiCode::Cyan),
        7 => Some(AnsiCode::White),
        _ => None,
    }
}

fn extended_color(codes: &[u8]) -> Option<(TextColor, usize)> {
    match codes {
        [38, 5, index, ..] => Some((TextColor::Indexed(*index), 3)),
        [38, 2, red, green, blue, ..] => Some((TextColor::Rgb(*red, *green, *blue), 5)),
        _ => None,
    }
}

impl Display for StyledChar {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "char: {}, color: {:?}, bold: {}",
            self.character, self.color, self.bold
        )
    }
}
