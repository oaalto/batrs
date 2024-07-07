use crate::ansi::styled_text_block::StyledTextBlock;
use crate::ansi::{ansi_colors, AnsiCode};
use eframe::epaint::FontId;
use egui::text::LayoutJob;
use egui::{TextBuffer, TextFormat};
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Range;

pub struct StyledLine {
    pub plain_line: String,
    pub blocks: Vec<StyledTextBlock>,
    pub gag: bool,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        let plain_line = remove_ansi_codes(line);
        Self {
            blocks: Self::parse_line(line, &plain_line),
            plain_line,
            gag: false,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let mut job = LayoutJob::default();
        self.blocks.iter().for_each(|block| {
            job.append(
                self.get_string(&block.range),
                0.0,
                TextFormat {
                    font_id: FontId::monospace(16.0),
                    color: ansi_colors::get_color(block.color, block.bold),
                    ..Default::default()
                },
            );
        });
        ui.label(job);
    }

    fn get_string(&self, range: &Range<usize>) -> &str {
        self.plain_line.char_range(range.clone())
    }

    fn parse_line(line: &str, plain_line: &str) -> Vec<StyledTextBlock> {
        let matches = parse_ansi::parse_bytes(line.as_bytes());
        let mut blocks: Vec<StyledTextBlock> = Vec::new();
        let mut current_block = StyledTextBlock::new();
        let mut prev_end = 0;

        matches.for_each(|m| {
            if m.start() > 0 && prev_end < m.start() {
                let sub = &line[prev_end..m.start()];
                let start = plain_line.find(sub).unwrap_or_default();
                let end = start + sub.len();
                current_block.range = Range { start, end };

                blocks.push(current_block.clone());
                current_block.reset();
            }

            let ansi_codes = parse_ansi_code_block(m.as_bytes());
            current_block.process_ansi_codes(&ansi_codes);

            prev_end = m.end();
        });

        // No matches means no ansi control sequences found
        if blocks.is_empty() {
            current_block.range = Range {
                start: 0,
                end: plain_line.len(),
            };
            blocks.push(current_block);
        } else if prev_end < line.len() {
            let sub = &line[prev_end..];
            if !sub.trim().is_empty() {
                let start = plain_line.find(sub).unwrap_or_default();
                current_block.range = Range {
                    start,
                    end: plain_line.len(),
                };

                blocks.push(current_block);
            }
        }

        blocks
    }
}

impl Display for StyledLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let blocks = self
            .blocks
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<String>>()
            .join(", ");

        write!(f, "{}: {}", &self.plain_line, blocks)
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
