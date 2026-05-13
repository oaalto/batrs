use crate::ansi::AnsiCode;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TextColor {
    Default,
    Ansi(AnsiCode),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl TextColor {
    pub const fn from_ansi(color: AnsiCode) -> Self {
        match color {
            AnsiCode::DefaultColor | AnsiCode::Reset | AnsiCode::Bold | AnsiCode::BoldOff => {
                Self::Default
            }
            color => Self::Ansi(color),
        }
    }
}

impl From<AnsiCode> for TextColor {
    fn from(value: AnsiCode) -> Self {
        Self::from_ansi(value)
    }
}

impl PartialEq<AnsiCode> for TextColor {
    fn eq(&self, other: &AnsiCode) -> bool {
        match (self, other) {
            (Self::Default, AnsiCode::DefaultColor) => true,
            (Self::Ansi(color), other) => color == other,
            _ => false,
        }
    }
}

impl PartialEq<TextColor> for AnsiCode {
    fn eq(&self, other: &TextColor) -> bool {
        other == self
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TextStyle {
    pub color: TextColor,
    pub bold: bool,
}

impl TextStyle {
    pub const DEFAULT: Self = Self::ansi(AnsiCode::DefaultColor, false);
    pub const RED: Self = Self::ansi(AnsiCode::Red, false);
    pub const GREEN: Self = Self::ansi(AnsiCode::Green, false);
    pub const YELLOW: Self = Self::ansi(AnsiCode::Yellow, false);
    pub const BLUE: Self = Self::ansi(AnsiCode::Blue, false);
    pub const MAGENTA: Self = Self::ansi(AnsiCode::Magenta, false);
    pub const CYAN: Self = Self::ansi(AnsiCode::Cyan, false);
    pub const WHITE: Self = Self::ansi(AnsiCode::White, false);

    pub const BRIGHT_RED: Self = Self::ansi(AnsiCode::Red, true);
    pub const BRIGHT_GREEN: Self = Self::ansi(AnsiCode::Green, true);
    pub const BRIGHT_YELLOW: Self = Self::ansi(AnsiCode::Yellow, true);
    pub const BRIGHT_BLUE: Self = Self::ansi(AnsiCode::Blue, true);
    pub const BRIGHT_MAGENTA: Self = Self::ansi(AnsiCode::Magenta, true);
    pub const BRIGHT_CYAN: Self = Self::ansi(AnsiCode::Cyan, true);
    pub const BRIGHT_WHITE: Self = Self::ansi(AnsiCode::White, true);

    pub fn new(color: impl Into<TextColor>, bold: bool) -> Self {
        Self {
            color: color.into(),
            bold,
        }
    }

    pub const fn ansi(color: AnsiCode, bold: bool) -> Self {
        Self {
            color: TextColor::from_ansi(color),
            bold,
        }
    }
}
