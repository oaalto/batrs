use std::mem;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug)]
pub struct InputState {
    current_typed_input: String,
    displayed_input: String,
    history: Vec<String>,
    cur_history_pos: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_char(&mut self, c: char) {
        self.displayed_input.push(c);
        self.current_typed_input.clone_from(&self.displayed_input);
        self.cur_history_pos = self.history.len();
    }

    pub fn backspace(&mut self) {
        if let Some((index, _)) = self.displayed_input.grapheme_indices(true).next_back() {
            self.displayed_input.truncate(index);
            self.current_typed_input.clone_from(&self.displayed_input);
            self.cur_history_pos = self.history.len();
        }
    }

    pub fn move_history(&mut self, direction: i32) {
        let prev_pos = self.cur_history_pos;

        if direction < 0 {
            if self.cur_history_pos > 0 {
                self.cur_history_pos = self.cur_history_pos.saturating_sub(1);
            }
        } else if self.cur_history_pos < self.history.len() {
            self.cur_history_pos = self.cur_history_pos.saturating_add(1);
        }

        if prev_pos != self.cur_history_pos {
            if self.cur_history_pos < self.history.len() {
                self.displayed_input
                    .clone_from(&self.history[self.cur_history_pos]);
            } else if self.cur_history_pos == self.history.len() {
                self.displayed_input.clone_from(&self.current_typed_input);
            }
        }
    }

    pub fn displayed_text(&self, hide_input: bool) -> String {
        if hide_input {
            String::new()
        } else {
            self.displayed_input.clone()
        }
    }

    pub fn cursor_offset(&self, hide_input: bool) -> u16 {
        if hide_input {
            1
        } else {
            self.displayed_input.graphemes(true).count() as u16 + 1
        }
    }

    pub fn displayed_input(&self) -> &str {
        &self.displayed_input
    }

    pub fn take_displayed_input(&mut self) -> String {
        mem::take(&mut self.displayed_input)
    }

    pub fn push_history(&mut self, input: String) {
        self.history.push(input);
        self.cur_history_pos = self.history.len();
    }

    pub fn clear_all(&mut self) {
        self.displayed_input.clear();
        self.current_typed_input.clear();
    }

    pub fn clear_current_typed_input(&mut self) {
        self.current_typed_input.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::InputState;

    #[test]
    fn history_moves_and_restores_typed_input() {
        let mut state = InputState::new();
        state.insert_char('h');
        state.insert_char('i');
        let history_entry = state.take_displayed_input();
        state.push_history(history_entry);

        state.insert_char('b');
        state.insert_char('y');
        state.insert_char('e');
        state.move_history(-1);

        assert_eq!(state.displayed_input(), "hi");

        state.move_history(1);
        assert_eq!(state.displayed_input(), "bye");
    }

    #[test]
    fn backspace_removes_last_grapheme() {
        let mut state = InputState::new();
        state.insert_char('h');
        state.insert_char('i');
        state.backspace();

        assert_eq!(state.displayed_input(), "h");
    }

    #[test]
    fn cursor_offset_starts_after_prompt() {
        let mut state = InputState::new();

        assert_eq!(state.cursor_offset(false), 1);

        state.insert_char('h');
        state.insert_char('i');

        assert_eq!(state.cursor_offset(false), 3);
    }

    #[test]
    fn hidden_input_cursor_offset_starts_after_prompt() {
        let state = InputState::new();

        assert_eq!(state.cursor_offset(true), 1);
    }
}
