#[derive(Debug, PartialEq, PartialOrd, Eq, Clone, Copy, num_derive::FromPrimitive)]
pub enum AnsiCodes {
    Reset = 0,
    Bold = 1,
    BoldOff = 22,
    Red = 31,
    Green = 32,
    Yellow = 33,
    Blue = 34,
    Magenta = 35,
    Cyan = 36,
    White = 37,
    DefaultColor = 39,
}
