use crate::ansi::AnsiCode;
use std::fmt::{Display, Formatter};
use std::ops::Range;

#[derive(Debug, Clone)]
pub struct StyledTextBlock {
    pub bold: bool,
    pub color: AnsiCode,
    pub range: Range<usize>,
}

impl StyledTextBlock {
    pub fn new() -> Self {
        Self {
            bold: false,
            color: AnsiCode::White,
            range: Range::default(),
        }
    }

    pub fn reset(&mut self) {
        *self = StyledTextBlock::new();
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

impl Display for StyledTextBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "({}..{}), color: {:?}, bold: {}",
            self.range.start, self.range.end, self.color, self.bold
        )
    }
}
