use crate::ansi::StyledLine;
use ratatui::text::{Line, Span};

#[derive(Default)]
pub struct OutputBuffer {
    lines: Vec<StyledLine>,
    wrapped_cache: Option<WrappedLinesCache>,
}

#[derive(Default)]
struct WrappedLinesCache {
    width: u16,
    source_count: usize,
    lines: Vec<ratatui::text::Line<'static>>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return cached rows, wrapping only pending source lines at an unchanged width.
    /// Width changes rebuild all rows; zero width leaves pending output untouched.
    ///
    /// The returned slice is borrowed from internal cache storage and stays valid
    /// until the next mutable `OutputBuffer` operation.
    pub fn wrapped_lines(&mut self, width: u16) -> &[Line<'static>] {
        if width == 0 {
            return &[];
        }

        let cache = self
            .wrapped_cache
            .get_or_insert_with(WrappedLinesCache::default);
        // ponytail: growing history and full width-change wrapping remain; revisit storage if measured limits warrant it.
        if cache.width != width {
            cache.width = width;
            cache.source_count = 0;
            cache.lines.clear();
        }
        cache.lines.extend(
            self.lines[cache.source_count..]
                .iter()
                .flat_map(|line| line.to_wrapped_lines(width))
                .map(owned_line),
        );
        cache.source_count = self.lines.len();

        cache.lines.as_slice()
    }

    /// Retain non-gagged source lines in order; existing lines remain immutable.
    pub fn append_lines(&mut self, mut lines: Vec<StyledLine>) {
        remove_gagged_lines(&mut lines);
        if lines.is_empty() {
            return;
        }
        self.lines.append(&mut lines);
    }

    /// Discard source and cached output, including progress from before a refill.
    pub fn clear(&mut self) {
        self.lines.clear();
        self.wrapped_cache = None;
    }

    #[cfg(test)]
    pub(crate) fn plain_lines(&self) -> Vec<&str> {
        self.lines
            .iter()
            .map(|line| line.plain_line.as_str())
            .collect()
    }
}

fn owned_line(line: Line<'_>) -> Line<'static> {
    let spans: Vec<_> = line.spans.into_iter().map(owned_span).collect();
    Line::from(spans)
}

fn owned_span(span: Span<'_>) -> Span<'static> {
    Span::styled(span.content.as_ref().to_string(), span.style)
}

fn remove_gagged_lines(lines: &mut Vec<StyledLine>) {
    let num_lines = lines.len();
    let mut indices: Vec<usize> = lines
        .iter()
        .enumerate()
        .map(|(index, line)| if line.gag { index } else { num_lines + 1 })
        .filter(|index| *index < num_lines + 1)
        .collect();

    indices.sort_by(|a, b| b.cmp(a));

    indices.iter().for_each(|index| {
        lines.remove(*index);
    });
}
