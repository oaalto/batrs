use crate::ansi::AnsiCode;

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct TextStyle {
    pub color: AnsiCode,
    pub bold: bool,
}

impl TextStyle {
    pub const DEFAULT: Self = Self::new(AnsiCode::DefaultColor, false);
    pub const RED: Self = Self::new(AnsiCode::Red, false);
    pub const GREEN: Self = Self::new(AnsiCode::Green, false);
    pub const YELLOW: Self = Self::new(AnsiCode::Yellow, false);
    pub const BLUE: Self = Self::new(AnsiCode::Blue, false);
    pub const MAGENTA: Self = Self::new(AnsiCode::Magenta, false);
    pub const CYAN: Self = Self::new(AnsiCode::Cyan, false);
    pub const WHITE: Self = Self::new(AnsiCode::White, false);

    pub const BRIGHT_RED: Self = Self::new(AnsiCode::Red, true);
    pub const BRIGHT_GREEN: Self = Self::new(AnsiCode::Green, true);
    pub const BRIGHT_YELLOW: Self = Self::new(AnsiCode::Yellow, true);
    pub const BRIGHT_BLUE: Self = Self::new(AnsiCode::Blue, true);
    pub const BRIGHT_MAGENTA: Self = Self::new(AnsiCode::Magenta, true);
    pub const BRIGHT_CYAN: Self = Self::new(AnsiCode::Cyan, true);
    pub const BRIGHT_WHITE: Self = Self::new(AnsiCode::White, true);

    pub const fn new(color: AnsiCode, bold: bool) -> Self {
        Self { color, bold }
    }
}
