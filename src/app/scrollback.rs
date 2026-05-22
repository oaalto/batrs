#[derive(Default, Debug)]
pub struct Scrollback {
    top_line: Option<usize>,
    total_lines: usize,
    visible_height: usize,
}

impl Scrollback {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn update_viewport(&mut self, total_lines: usize, visible_height: usize) {
        self.total_lines = total_lines;
        self.visible_height = visible_height;

        if self.max_offset() == 0 {
            self.top_line = None;
        }
    }

    pub fn page_up(&mut self) {
        self.scroll_up(self.page_size());
    }

    pub fn page_down(&mut self) {
        self.scroll_down(self.page_size());
    }

    pub fn scroll_up(&mut self, line_count: usize) {
        let max_offset = self.max_offset();
        if max_offset == 0 {
            self.top_line = None;
            return;
        }

        let current_top_line = self.top_line.unwrap_or(max_offset).min(max_offset);
        self.top_line = Some(current_top_line.saturating_sub(line_count.max(1)));
    }

    pub fn scroll_down(&mut self, line_count: usize) {
        let max_offset = self.max_offset();
        if max_offset == 0 {
            self.top_line = None;
            return;
        }

        let current_top_line = self.top_line.unwrap_or(max_offset).min(max_offset);
        let next_top_line = current_top_line.saturating_add(line_count.max(1));
        if next_top_line >= max_offset {
            self.follow_latest();
        } else {
            self.top_line = Some(next_top_line);
        }
    }

    pub fn follow_latest(&mut self) {
        self.top_line = None;
    }

    pub fn offset(&self) -> u16 {
        let offset = self.top_line.unwrap_or_else(|| self.max_offset());
        offset.min(self.max_offset()).min(u16::MAX as usize) as u16
    }

    fn max_offset(&self) -> usize {
        self.total_lines.saturating_sub(self.visible_height)
    }

    fn page_size(&self) -> usize {
        self.visible_height.max(1)
    }
}

#[cfg(test)]
mod tests {
    use super::Scrollback;

    #[test]
    fn follows_latest_output_by_default() {
        let mut scrollback = Scrollback::new();

        scrollback.update_viewport(100, 20);

        assert_eq!(scrollback.offset(), 80);
    }

    #[test]
    fn page_up_moves_one_viewport_and_clamps_at_top() {
        let mut scrollback = Scrollback::new();
        scrollback.update_viewport(100, 20);

        scrollback.page_up();
        assert_eq!(scrollback.offset(), 60);

        scrollback.page_up();
        scrollback.page_up();
        scrollback.page_up();
        scrollback.page_up();
        assert_eq!(scrollback.offset(), 0);
    }

    #[test]
    fn page_down_returns_to_follow_mode_at_bottom() {
        let mut scrollback = Scrollback::new();
        scrollback.update_viewport(100, 20);

        scrollback.page_up();
        scrollback.page_down();
        assert_eq!(scrollback.offset(), 80);

        scrollback.update_viewport(120, 20);
        assert_eq!(scrollback.offset(), 100);
    }

    #[test]
    fn line_scroll_moves_by_requested_amount_and_clamps() {
        let mut scrollback = Scrollback::new();
        scrollback.update_viewport(100, 20);

        scrollback.scroll_up(3);
        assert_eq!(scrollback.offset(), 77);

        scrollback.scroll_down(2);
        assert_eq!(scrollback.offset(), 79);

        scrollback.scroll_down(2);
        assert_eq!(scrollback.offset(), 80);
    }

    #[test]
    fn new_output_keeps_scrolled_back_top_line_stable() {
        let mut scrollback = Scrollback::new();
        scrollback.update_viewport(100, 20);

        scrollback.page_up();
        scrollback.update_viewport(120, 20);

        assert_eq!(scrollback.offset(), 60);
    }

    #[test]
    fn paging_is_noop_when_output_fits_viewport() {
        let mut scrollback = Scrollback::new();
        scrollback.update_viewport(10, 20);

        scrollback.page_up();
        assert_eq!(scrollback.offset(), 0);

        scrollback.page_down();
        assert_eq!(scrollback.offset(), 0);
    }
}
