use crate::ansi::ansi_colors::get_color;
use crate::ansi::AnsiCode;
use eframe::epaint::FontId;
use egui::text::LayoutJob;
use egui::TextFormat;
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
}

pub struct StyledLine {
    pub plain_line: String,
    pub blocks: Vec<StyledTextBlock>,
    pub gag: bool,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        Self {
            blocks: Self::parse_line(line),
            plain_line: remove_ansi_codes(line),
            gag: false,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let mut job = LayoutJob::default();
        self.blocks.iter().for_each(|block| {
            job.append(
                &block.text,
                0.0,
                TextFormat {
                    font_id: FontId::monospace(16.0),
                    color: get_color(block.color, block.bold),
                    ..Default::default()
                },
            );
        });
        ui.label(job);
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
    pub static ref ANSI_REGEX: Regex =
        Regex::new(r"\x1b\[([\x30-\x3f]*[\x20-\x2f]*[\x40-\x7e])").unwrap();
    pub static ref ANSI_CODE_REGEX: Regex = Regex::new(r"\u{1b}\[(.*)m").unwrap();
}

fn parse_ansi_code_block(block: &[u8]) -> Vec<AnsiCode> {
    match std::str::from_utf8(block) {
        Ok(s) => {
            if let Some(captures) = ANSI_CODE_REGEX.captures(s) {
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
