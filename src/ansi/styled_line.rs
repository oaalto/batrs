use crate::ansi::styled_text_block::StyledChar;
use crate::ansi::{TextStyle, palette};
use lazy_static::lazy_static;
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use regex::Regex;
use std::fmt::{Display, Formatter};
use std::ops::Range;
use unicode_segmentation::UnicodeSegmentation;
use unicode_width::UnicodeWidthStr;

const TAB_STOP_WIDTH: usize = 8;

#[derive(Debug)]
pub struct StyledLine {
    pub plain_line: String,
    pub styled_chars: Vec<StyledChar>,
    pub gag: bool,
}

struct MatchedAnsiBlock {
    start: usize,
    end: usize,
    codes: Vec<u8>,
}

impl StyledLine {
    pub fn new(line: &str) -> Self {
        let styled_chars = Self::parse_line(line);
        let plain_line = plain_line_from_chars(&styled_chars);

        Self {
            styled_chars,
            plain_line,
            gag: false,
        }
    }

    pub fn to_wrapped_lines(&self, width: u16) -> Vec<Line<'_>> {
        if width == 0 {
            return Vec::new();
        }

        if self.styled_chars.is_empty() {
            return vec![Line::from("")];
        }

        let width = width as usize;
        let mut lines = Vec::new();
        let mut start = 0;

        while start < self.styled_chars.len() {
            let end = (start + width).min(self.styled_chars.len());
            lines.push(Self::line_from_chars(&self.styled_chars[start..end]));
            start = end;
        }

        lines
    }

    pub fn set_block_style(&mut self, part: &str, style: TextStyle) {
        if let Some(range) = self.get_range_for(part) {
            range.into_iter().for_each(|index| {
                self.styled_chars[index].color = style.color;
                self.styled_chars[index].bold = style.bold;
            });
        }
    }

    /// Style graphemes whose `plain_line` UTF-8 byte span lies in `[range.start, range.end)`.
    pub fn set_plain_byte_range_style(&mut self, range: Range<usize>, style: TextStyle) {
        let len = self.plain_line.len();
        let start = range.start.min(len);
        let end = range.end.min(len).max(start);
        let grapheme_start = plain_prefix_grapheme_count(&self.plain_line, start);
        let grapheme_end = plain_prefix_grapheme_count(&self.plain_line, end);
        for styled in self
            .styled_chars
            .iter_mut()
            .take(grapheme_end)
            .skip(grapheme_start)
        {
            styled.color = style.color;
            styled.bold = style.bold;
        }
    }

    fn get_range_for(&self, part: &str) -> Option<Range<usize>> {
        self.plain_line.find(part).map(|start| Range {
            start,
            end: start + part.len(),
        })
    }

    pub fn set_line_style(&mut self, style: TextStyle) {
        self.styled_chars = self
            .plain_line
            .graphemes(true)
            .map(|c| StyledChar {
                bold: style.bold,
                color: style.color,
                character: c.to_string(),
            })
            .collect();
    }

    /// Inserts `suffix` into `plain_line` at `byte_idx` (UTF-8 boundary), and splices matching
    /// graphemes into `styled_chars`. New graphemes copy style from the grapheme before the
    /// insertion point (or the first grapheme if inserting at the front).
    pub fn insert_plain_after_plain_byte_idx(&mut self, byte_idx: usize, suffix: &str) {
        if suffix.is_empty() {
            return;
        }

        let byte_idx = byte_idx.min(self.plain_line.len());
        let grapheme_insert = plain_prefix_grapheme_count(&self.plain_line, byte_idx);

        let style_template = if grapheme_insert > 0 {
            self.styled_chars.get(grapheme_insert - 1)
        } else {
            self.styled_chars.first()
        };
        let style = style_template
            .map(|styled| TextStyle::new(styled.color, styled.bold))
            .unwrap_or(TextStyle::DEFAULT);

        let new_styled: Vec<StyledChar> = suffix
            .graphemes(true)
            .map(|grapheme| {
                let mut next = StyledChar::new(grapheme.to_string());
                next.bold = style.bold;
                next.color = style.color;
                next
            })
            .collect();

        self.plain_line.insert_str(byte_idx, suffix);
        self.styled_chars
            .splice(grapheme_insert..grapheme_insert, new_styled);
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

        let mut block_index = 0;
        let mut current_style = StyledChar::new(String::new());
        let mut styled_chars = Vec::new();
        let mut column = 0;

        for (byte_index, grapheme) in line.grapheme_indices(true) {
            while let Some(block) = matched_ansi_blocks.get(block_index)
                && block.end <= byte_index
            {
                current_style.process_sgr_codes(&block.codes);
                block_index += 1;
            }

            if matched_ansi_blocks
                .get(block_index)
                .is_some_and(|block| block.start <= byte_index && byte_index < block.end)
            {
                continue;
            }

            if grapheme == "\t" {
                let spaces = TAB_STOP_WIDTH - (column % TAB_STOP_WIDTH);
                styled_chars.extend((0..spaces).map(|_| {
                    let mut styled_char = StyledChar::new(" ".to_string());
                    styled_char.color = current_style.color;
                    styled_char.bold = current_style.bold;
                    styled_char
                }));
                column += spaces;
                continue;
            }

            let mut styled_char = StyledChar::new(grapheme.to_string());
            styled_char.color = current_style.color;
            styled_char.bold = current_style.bold;
            column += UnicodeWidthStr::width(grapheme);
            styled_chars.push(styled_char);
        }

        styled_chars
    }

    fn line_from_chars(chars: &[StyledChar]) -> Line<'_> {
        let spans: Vec<Span<'_>> = chars
            .iter()
            .map(|c| {
                let mut style =
                    Style::default().fg(palette::get_color(TextStyle::new(c.color, c.bold)));
                if c.bold {
                    style = style.add_modifier(Modifier::BOLD);
                }
                Span::styled(c.character.clone(), style)
            })
            .collect();
        Line::from(spans)
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
    pub static ref ANSI_CODE_REGEX: Regex = Regex::new(r"\u{1b}\[(.*)m").unwrap();
}

fn parse_ansi_code_block(block: &[u8]) -> Vec<u8> {
    match std::str::from_utf8(block) {
        Ok(s) => {
            if let Some(captures) = ANSI_CODE_REGEX.captures(s) {
                let (_, groups): (&str, [&str; 1]) = captures.extract();

                return groups[0]
                    .split(';')
                    .filter_map(|c| c.parse::<u8>().ok())
                    .collect();
            }
        }
        Err(e) => {
            eprintln!("parsing ansi code: {e}")
        }
    }

    Vec::new()
}

fn plain_line_from_chars(chars: &[StyledChar]) -> String {
    chars
        .iter()
        .map(|styled_char| styled_char.character.as_str())
        .collect()
}

fn plain_prefix_grapheme_count(plain_line: &str, byte_end: usize) -> usize {
    let end = byte_end.min(plain_line.len());
    plain_line[..end].graphemes(true).count()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::{AnsiCode, TextColor, TextStyle, palette};
    use ratatui::style::{Color, Modifier};

    #[test]
    fn plain_byte_range_style_marks_substring() {
        let mut line = StyledLine::new("alpha β gamma");
        let beta_start = line.plain_line.find('β').unwrap();
        let beta_end = beta_start + "β".len();
        line.set_plain_byte_range_style(beta_start..beta_end, TextStyle::BRIGHT_RED);
        let beta_grapheme = line.plain_line[..beta_start].graphemes(true).count();
        assert_eq!(
            line.styled_chars[beta_grapheme].color,
            TextStyle::BRIGHT_RED.color
        );
        assert!(line.styled_chars[beta_grapheme].bold);
        assert_eq!(line.styled_chars[0].color, TextColor::Default);
    }

    #[test]
    fn leading_tab_expands_to_spaces() {
        let line = StyledLine::new("\tHellish bandolier contains:");

        assert_eq!(line.plain_line, "        Hellish bandolier contains:");
        assert_eq!(line.styled_chars.len(), line.plain_line.chars().count());
        assert!(line.styled_chars.iter().all(|c| c.character != "\t"));

        let wrapped = line.to_wrapped_lines(80);
        assert!(
            wrapped[0]
                .spans
                .iter()
                .all(|span| !span.content.contains('\t'))
        );
    }

    #[test]
    fn styled_tab_expands_with_active_style() {
        let line = StyledLine::new("\x1b[32mA\tB");

        assert_eq!(line.plain_line, "A       B");
        assert_eq!(line.styled_chars.len(), 9);
        assert!(
            line.styled_chars[0..8]
                .iter()
                .all(|c| c.color == AnsiCode::Green)
        );
    }

    #[test]
    fn parses_and_renders_256_color_foreground() {
        let line = StyledLine::new("\x1b[38;5;219mpink\x1b[0m plain");

        assert_eq!(line.plain_line, "pink plain");
        assert_eq!(line.styled_chars[0].color, TextColor::Indexed(219));
        assert_eq!(line.styled_chars[4].color, TextColor::Default);

        let rendered = line.to_wrapped_lines(80);
        assert_eq!(rendered[0].spans[0].style.fg, Some(Color::Indexed(219)));
        assert_eq!(rendered[0].spans[4].style.fg, Some(palette::TEXT));
    }

    #[test]
    fn parses_and_renders_truecolor_foreground() {
        let line = StyledLine::new("\x1b[38;2;255;128;64morange\x1b[0m");

        assert_eq!(line.plain_line, "orange");
        assert_eq!(line.styled_chars[0].color, TextColor::Rgb(255, 128, 64));

        let rendered = line.to_wrapped_lines(80);
        assert_eq!(
            rendered[0].spans[0].style.fg,
            Some(Color::Rgb(255, 128, 64))
        );
    }

    #[test]
    fn bright_white_stays_bold_and_uses_palette_white() {
        let line = StyledLine::new("\x1b[1;37mwhite\x1b[0m");

        assert_eq!(line.styled_chars[0].color, AnsiCode::White);
        assert!(line.styled_chars[0].bold);

        let rendered = line.to_wrapped_lines(80);
        assert_eq!(rendered[0].spans[0].style.fg, Some(palette::BOLD_WHITE));
        assert!(
            rendered[0].spans[0]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }

    #[test]
    fn bold_default_text_renders_as_bright_white() {
        let line = StyledLine::new("\x1b[1mYou start chanting.\x1b[0m");

        assert_eq!(line.styled_chars[0].color, TextColor::Default);
        assert!(line.styled_chars[0].bold);

        let rendered = line.to_wrapped_lines(80);
        assert_eq!(rendered[0].spans[0].style.fg, Some(palette::BOLD_WHITE));
        assert!(
            rendered[0].spans[0]
                .style
                .add_modifier
                .contains(Modifier::BOLD)
        );
    }

    #[test]
    fn reset_returns_following_text_to_default_style() {
        let line = StyledLine::new("\x1b[31mred\x1b[0mplain");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert_eq!(line.styled_chars[3].color, TextColor::Default);
        assert!(!line.styled_chars[3].bold);

        let rendered = line.to_wrapped_lines(80);
        assert_eq!(rendered[0].spans[0].style.fg, Some(palette::RED));
        assert_eq!(rendered[0].spans[3].style.fg, Some(palette::TEXT));
    }
}
