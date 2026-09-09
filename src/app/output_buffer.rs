use crate::ansi::StyledLine;
use ratatui::text::{Line, Span};

#[derive(Default)]
pub struct OutputBuffer {
    lines: Vec<StyledLine>,
    generation: u64,
    wrapped_cache: Option<WrappedLinesCache>,
}

#[derive(Default)]
struct WrappedLinesCache {
    width: u16,
    generation: u64,
    lines: Vec<ratatui::text::Line<'static>>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Return wrapped output lines cached by terminal width and output generation.
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
        if cache.width != width || cache.generation != self.generation {
            cache.width = width;
            cache.generation = self.generation;
            cache.lines = self
                .lines
                .iter()
                .flat_map(|line| line.to_wrapped_lines(width))
                .map(owned_line)
                .collect();
        }

        cache.lines.as_slice()
    }

    pub fn append_lines(&mut self, mut lines: Vec<StyledLine>) {
        remove_gagged_lines(&mut lines);
        if lines.is_empty() {
            return;
        }
        self.lines.append(&mut lines);
        self.generation = self.generation.wrapping_add(1);
    }

    pub fn clear(&mut self) {
        if self.lines.is_empty() {
            return;
        }
        self.lines.clear();
        self.generation = self.generation.wrapping_add(1);
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
