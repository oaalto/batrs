use crate::ansi::styled_text_block::StyledChar;
use crate::ansi::{AnsiCode, ansi_colors};
use eframe::epaint::FontId;
use egui::TextFormat;
use egui::text::LayoutJob;
use lazy_static::lazy_static;
use num_traits::FromPrimitive;
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;

pub struct StyledLine {
    pub plain_line: String,
    pub styled_chars: Vec<StyledChar>,
    pub gag: bool,
}

struct MatchedAnsiBlock {
    start: usize,
    end: usize,
    codes: Vec<AnsiCode>,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        Self {
            styled_chars: Self::parse_line(line),
            plain_line: remove_ansi_codes(line),
            gag: false,
        }
    }

    pub fn show(&self, ui: &mut egui::Ui) {
        let mut job = LayoutJob::default();
        self.styled_chars.iter().for_each(|c| {
            job.append(
                &c.character.to_string(),
                0.0,
                TextFormat {
                    font_id: FontId::monospace(16.0),
                    color: ansi_colors::get_color(c.color, c.bold),
                    ..Default::default()
                },
            );
        });
        ui.label(job);
    }

    pub fn set_block_color(&mut self, part: &str, color: AnsiCode, bold: bool) {
        if let Some(range) = self.get_range_for(part) {
            range.into_iter().for_each(|index| {
                self.styled_chars[index].color = color;
                self.styled_chars[index].bold = bold;
            });
        }
    }

    fn get_range_for(&self, part: &str) -> Option<Range<usize>> {
        self.plain_line.find(part).map(|start| Range {
            start,
            end: start + part.len(),
        })
    }

    pub fn set_line_color(&mut self, color: AnsiCode, bold: bool) {
        self.styled_chars = self
            .plain_line
            .graphemes(true)
            .map(|c| StyledChar {
                bold,
                color,
                character: c.to_string(),
            })
            .collect();
    }

    fn parse_line(line: &str) -> Vec<StyledChar> {
        let matches = parse_ansi::parse_bytes(line.as_bytes());

        let matched_ansi_blocks: Vec<MatchedAnsiBlock> = matches
            .map(|m| MatchedAnsiBlock {
                start: m.start(),
                end: m.end(),
                codes: parse_ansi_code_block(m.as_bytes()),
            })
            .collect();

        let find_block = |i| matched_ansi_blocks.iter().rfind(|b| b.end <= i);

        let is_inside_ansi_block = |i| {
            matched_ansi_blocks
                .iter()
                .any(|b| b.start <= i && b.end > i)
        };

        line.graphemes(true)
            .map(|c| StyledChar::new(c.to_string()))
            .enumerate()
            .filter_map(|(i, mut styled_char)| {
                if is_inside_ansi_block(i) {
                    return None;
                }

                if let Some(block) = find_block(i) {
                    styled_char.process_ansi_codes(&block.codes);
                }

                Some(styled_char)
            })
            .collect()
    }
}

impl Display for StyledLine {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let blocks = self
            .styled_chars
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
            eprintln!("parsing ansi code: {e}")
        }
    }

    Vec::new()
}

fn remove_ansi_codes(line: &str) -> String {
    ANSI_REGEX.replace_all(line, "").to_string()
}
