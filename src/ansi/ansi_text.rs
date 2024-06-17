use crate::ansi::{ansi_colors, AnsiCode};
use crate::Message;
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct StyledTextBlock {
    pub bold: bool,
    pub color: AnsiCode,
    pub text: String,
}

impl StyledTextBlock {
    pub fn new() -> Self {
        Self {
            bold: false,
            color: AnsiCode::White,
            text: "".to_string(),
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

    pub fn to_text_element(&self) -> iced::Element<'_, Message> {
        let color = ansi_colors::get_color(&self.color, self.bold);
        iced::Element::from(iced::widget::text(&self.text).color(color))
    }
}

pub struct StyledLine {
    pub plain_line: String,
    pub blocks: Vec<StyledTextBlock>,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        Self {
            plain_line: remove_ansi_codes(line),
            blocks: Self::parse_line(line),
        }
    }

    pub fn to_row(&self) -> iced::widget::Row<'_, Message> {
        iced::widget::row(self.blocks.iter().map(StyledTextBlock::to_text_element))
            .width(iced::Length::Fill)
    }

    fn parse_line(line: &str) -> Vec<StyledTextBlock> {
        let matches = parse_ansi::parse_bytes(line.as_bytes());
        let mut blocks: Vec<StyledTextBlock> = Vec::new();
        let mut current_block = StyledTextBlock::new();
        let mut prev_end = 0;

        matches.for_each(|m| {
            if m.start() > 0 {
                current_block.text = line[prev_end..m.start()].to_string();
                blocks.push(current_block.clone());
                current_block = StyledTextBlock::new();

                let ansi_codes = parse_ansi_code_block(m.as_bytes());
                current_block.process_ansi_codes(&ansi_codes);
                prev_end = m.end();
            } else {
                let ansi_codes = parse_ansi_code_block(m.as_bytes());
                current_block.process_ansi_codes(&ansi_codes);
                prev_end = m.end();
            }
        });

        // No matches means no ansi control sequences found
        if blocks.is_empty() {
            current_block.text = line.to_string();
            blocks.push(current_block);
        } else if prev_end < line.len() {
            current_block.text = line[prev_end..].to_string();
            blocks.push(current_block);
        }

        blocks
    }
}

lazy_static! {
    pub static ref ANSI_REGEX: Regex = Regex::new(r"\u{1b}\[(.*)m").unwrap();
}

fn parse_ansi_code_block(block: &[u8]) -> Vec<AnsiCode> {
    match std::str::from_utf8(block) {
        Ok(s) => {
            if let Some(captures) = ANSI_REGEX.captures(s) {
                let (_, groups): (&str, [&str; 1]) = captures.extract();

                return groups[0]
                    .split(';')
                    .filter_map(|c| c.parse::<u8>().ok())
                    .filter_map(AnsiCode::from_u8)
                    .collect();
            }
        }
        Err(e) => {
            eprintln!("parsing ansi code: {}", e)
        }
    }

    Vec::new()
}

fn remove_ansi_codes(line: &str) -> String {
    ANSI_REGEX.replace_all(line, "").to_string()
}