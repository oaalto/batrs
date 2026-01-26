use crate::ansi::StyledLine;

#[derive(Default)]
pub struct OutputBuffer {
    lines: Vec<StyledLine>,
}

impl OutputBuffer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn lines(&self) -> &[StyledLine] {
        &self.lines
    }

    pub fn append_lines(&mut self, mut lines: Vec<StyledLine>) {
        remove_gagged_lines(&mut lines);
        self.lines.append(&mut lines);
    }
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
