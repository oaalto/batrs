use crate::ansi::AnsiCode;
use std::fmt::{Display, Formatter};

#[derive(Debug, Clone)]
pub struct StyledChar {
    pub bold: bool,
    pub color: AnsiCode,
    pub character: String,
}

impl StyledChar {
    pub fn reset(&mut self) {
        self.bold = false;
        self.color = AnsiCode::White;
    }

    pub fn process_ansi_codes(&mut self, ansi_codes: &[AnsiCode]) {
        ansi_codes.iter().for_each(|code| match code {
            AnsiCode::Reset => self.reset(),
            AnsiCode::Bold => self.bold = true,
            AnsiCode::BoldOff => self.bold = false,
            AnsiCode::DefaultColor => self.color = AnsiCode::White,
            color => self.color = *color,
        });
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
