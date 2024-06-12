use crate::ansi_codes::AnsiCodes;
use crate::ansi_colors::AnsiColors;
use crate::{ansi_colors, Message};
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use regex::Regex;

#[derive(Debug, Clone)]
pub struct StyledTextBlock {
    pub bold: bool,
    pub color: AnsiColors,
    pub text: String,
}

impl StyledTextBlock {
    pub fn new() -> Self {
        Self {
            bold: false,
            color: AnsiColors::White,
            text: "".to_string(),
        }
    }

    pub fn reset(&mut self) {
        *self = StyledTextBlock::new();
    }

    pub fn process_ansi_codes(&mut self, ansi_codes: &[AnsiCodes]) {
        ansi_codes.iter().for_each(|code| match code {
            AnsiCodes::Reset => self.reset(),
            AnsiCodes::Bold => self.bold = true,
            AnsiCodes::BoldOff => self.bold = false,
            AnsiCodes::DefaultColor => self.color = AnsiColors::White,
            color => self.color = AnsiColors::from_ansi_code(color),
        });
    }

    pub fn to_text_element(&self) -> iced::Element<'_, Message> {
        let color = ansi_colors::get_color(&self.color, self.bold);
        iced::Element::from(iced::widget::text(&self.text).color(color))
    }
}

pub struct StyledLine {
    original: String,
    blocks: Vec<StyledTextBlock>,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        Self {
            original: line.to_owned(),
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

        /*        println!();
                println!("{}", line);
                println!("{:?}", &blocks);
        */
        blocks
    }
}

lazy_static! {
    pub static ref ANSI_REGEX: Regex = Regex::new(r"\u{1b}\[(.*)m").unwrap();
}

fn parse_ansi_code_block(block: &[u8]) -> Vec<AnsiCodes> {
    match std::str::from_utf8(block) {
        Ok(s) => {
            if let Some(captures) = ANSI_REGEX.captures(s) {
                let (_, groups): (&str, [&str; 1]) = captures.extract();

                return groups[0]
                    .split(';')
                    .filter_map(|c| c.parse::<u8>().ok())
                    .filter_map(AnsiCodes::from_u8)
                    .collect();
            }
        }
        Err(e) => {
            eprintln!("parsing ansi code: {}", e)
        }
    }

    Vec::new()
}
