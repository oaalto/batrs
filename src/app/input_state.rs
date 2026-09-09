use std::mem;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default, Debug)]
pub struct InputState {
    current_typed_input: String,
    displayed_input: String,
    cursor_position: usize,
    cursor_display_offset: u16,
    history: Vec<String>,
    cur_history_pos: usize,
}

impl InputState {
    pub fn new() -> Self {
        Self::default()
    }

    /// Recompute the visible cursor offset from the current grapheme prefix.
    fn refresh_cursor_display_offset(&mut self) {
        self.cursor_display_offset = self.displayed_input[..self.cursor_position]
            .graphemes(true)
            .count() as u16;
    }

    pub fn insert_char(&mut self, c: char) {
        self.displayed_input.insert(self.cursor_position, c);
        self.cursor_position += c.len_utf8();
        self.refresh_cursor_display_offset();
        self.sync_current_typed_input();
    }

    pub fn insert_str(&mut self, text: &str) {
        if text.is_empty() {
            return;
        }

        self.displayed_input.insert_str(self.cursor_position, text);
        self.cursor_position += text.len();
        self.refresh_cursor_display_offset();
        self.sync_current_typed_input();
    }

    pub fn backspace(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        let Some((index, _)) = self.displayed_input[..self.cursor_position]
            .grapheme_indices(true)
            .next_back()
        else {
            return;
        };

        self.displayed_input.drain(index..self.cursor_position);
        self.cursor_position = index;
        self.refresh_cursor_display_offset();
        self.sync_current_typed_input();
    }

    pub fn delete(&mut self) {
        if self.cursor_position >= self.displayed_input.len() {
            return;
        }

        let remainder = &self.displayed_input[self.cursor_position..];
        let Some((offset, grapheme)) = remainder.grapheme_indices(true).next() else {
            return;
        };

        let start = self.cursor_position + offset;
        let end = start + grapheme.len();
        self.displayed_input.drain(start..end);
        self.refresh_cursor_display_offset();
        self.sync_current_typed_input();
    }

    pub fn move_cursor_left(&mut self) {
        if let Some((index, _)) = self.displayed_input[..self.cursor_position]
            .grapheme_indices(true)
            .next_back()
        {
            self.cursor_position = index;
            self.refresh_cursor_display_offset();
        }
    }

    pub fn move_cursor_right(&mut self) {
        if self.cursor_position >= self.displayed_input.len() {
            return;
        }

        let remainder = &self.displayed_input[self.cursor_position..];
        if let Some((offset, grapheme)) = remainder.grapheme_indices(true).next() {
            self.cursor_position += offset + grapheme.len();
            self.refresh_cursor_display_offset();
        }
    }

    pub fn move_cursor_word_left(&mut self) {
        if self.cursor_position == 0 {
            return;
        }

        self.cursor_position = self.displayed_input[..self.cursor_position]
            .unicode_word_indices()
            .map(|(index, _)| index)
            .next_back()
            .unwrap_or(0);
        self.refresh_cursor_display_offset();
    }

    pub fn move_cursor_word_right(&mut self) {
        let Some((index, _)) = self
            .displayed_input
            .unicode_word_indices()
            .find(|(index, _)| *index > self.cursor_position)
        else {
            self.cursor_position = self.displayed_input.len();
            self.refresh_cursor_display_offset();
            return;
        };

        self.cursor_position = index;
        self.refresh_cursor_display_offset();
    }

    pub fn move_cursor_to_start(&mut self) {
        self.cursor_position = 0;
        self.cursor_display_offset = 0;
    }

    pub fn move_cursor_to_end(&mut self) {
        self.cursor_position = self.displayed_input.len();
        self.refresh_cursor_display_offset();
    }

    fn sync_current_typed_input(&mut self) {
        if self.cur_history_pos == self.history.len() {
            return;
        }

        self.current_typed_input.clone_from(&self.displayed_input);
        self.cur_history_pos = self.history.len();
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
            if prev_pos == self.history.len() {
                self.current_typed_input.clone_from(&self.displayed_input);
            }

            if self.cur_history_pos < self.history.len() {
                self.displayed_input
                    .clone_from(&self.history[self.cur_history_pos]);
            } else if self.cur_history_pos == self.history.len() {
                self.displayed_input.clone_from(&self.current_typed_input);
            }
            self.cursor_position = self.displayed_input.len();
            self.refresh_cursor_display_offset();
        }
    }

    pub fn cursor_offset(&self, hide_input: bool) -> u16 {
        if hide_input {
            1
        } else {
            self.cursor_display_offset + 1
        }
    }

    pub fn displayed_input(&self) -> &str {
        &self.displayed_input
    }

    pub fn take_displayed_input(&mut self) -> String {
        self.cursor_position = 0;
        self.cursor_display_offset = 0;
        mem::take(&mut self.displayed_input)
    }

    pub fn push_history(&mut self, input: String) {
        if input.is_empty() || self.history.last() == Some(&input) {
            self.cur_history_pos = self.history.len();
            return;
        }

        self.history.push(input);
        self.cur_history_pos = self.history.len();
    }

    pub fn clear_all(&mut self) {
        self.displayed_input.clear();
        self.current_typed_input.clear();
        self.cursor_position = 0;
        self.cursor_display_offset = 0;
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
        assert_eq!(state.cursor_offset(false), 4);

        state.insert_char('!');
        assert_eq!(state.displayed_input(), "bye!");
    }

    #[test]
    fn history_skips_empty_entries_and_consecutive_duplicates() {
        let mut state = InputState::new();

        state.push_history(String::new());
        state.push_history("look".to_string());
        state.push_history("look".to_string());
        state.push_history("north".to_string());

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "north");

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "look");

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "look");
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
    fn cursor_left_and_right_move_one_grapheme() {
        let mut state = InputState::new();
        state.insert_char('a');
        state.insert_char('é');
        state.insert_char('b');

        state.move_cursor_left();
        assert_eq!(state.cursor_offset(false), 3);

        state.move_cursor_left();
        assert_eq!(state.cursor_offset(false), 2);

        state.move_cursor_right();
        assert_eq!(state.cursor_offset(false), 3);
    }

    #[test]
    fn insert_and_backspace_work_in_middle_of_input() {
        let mut state = InputState::new();
        state.insert_char('h');
        state.insert_char('i');
        state.move_cursor_left();
        state.insert_char('!');

        assert_eq!(state.displayed_input(), "h!i");
        assert_eq!(state.cursor_offset(false), 3);

        state.backspace();
        assert_eq!(state.displayed_input(), "hi");
        assert_eq!(state.cursor_offset(false), 2);
    }

    #[test]
    fn delete_removes_grapheme_after_cursor() {
        let mut state = InputState::new();
        for character in "aéb".chars() {
            state.insert_char(character);
        }
        state.move_cursor_left();
        state.move_cursor_left();

        state.delete();

        assert_eq!(state.displayed_input(), "ab");
        assert_eq!(state.cursor_offset(false), 2);
    }

    #[test]
    fn insert_str_inserts_text_at_cursor_once() {
        let mut state = InputState::new();
        state.insert_str("hello");
        state.move_cursor_left();
        state.move_cursor_left();

        state.insert_str("yy");

        assert_eq!(state.displayed_input(), "helyylo");
        assert_eq!(state.cursor_offset(false), 6);
    }

    #[test]
    fn word_left_jumps_to_previous_word_start() {
        let mut state = InputState::new();
        for character in "look, north now".chars() {
            state.insert_char(character);
        }

        state.move_cursor_word_left();
        assert_eq!(state.cursor_offset(false), 13);

        state.move_cursor_word_left();
        assert_eq!(state.cursor_offset(false), 7);

        state.move_cursor_word_left();
        assert_eq!(state.cursor_offset(false), 1);
    }

    #[test]
    fn word_right_jumps_to_next_word_start_or_input_end() {
        let mut state = InputState::new();
        for character in "look, north now".chars() {
            state.insert_char(character);
        }
        state.move_cursor_word_left();
        state.move_cursor_word_left();
        state.move_cursor_word_left();

        state.move_cursor_word_right();
        assert_eq!(state.cursor_offset(false), 7);

        state.move_cursor_word_right();
        assert_eq!(state.cursor_offset(false), 13);

        state.move_cursor_word_right();
        assert_eq!(state.cursor_offset(false), 16);
    }

    #[test]
    fn home_and_end_move_to_input_boundaries() {
        let mut state = InputState::new();
        for character in "héllo".chars() {
            state.insert_char(character);
        }
        state.move_cursor_left();

        state.move_cursor_to_start();
        assert_eq!(state.cursor_offset(false), 1);

        state.insert_char('>');
        assert_eq!(state.displayed_input(), ">héllo");

        state.move_cursor_to_end();
        assert_eq!(state.cursor_offset(false), 7);

        state.insert_char('<');
        assert_eq!(state.displayed_input(), ">héllo<");
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
    fn history_navigation_returns_to_empty_input_after_submit() {
        // Regression: after submitting a command, navigating history up then
        // down must return an empty input, not the previously submitted command.
        let mut state = InputState::new();
        state.insert_char('l');
        state.insert_char('o');
        state.insert_char('o');
        state.insert_char('k');
        let history_entry = state.take_displayed_input();
        state.push_history(history_entry);
        state.clear_current_typed_input();

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "look");

        state.move_history(1);
        assert_eq!(state.displayed_input(), "");
    }

    #[test]
    fn hidden_input_cursor_offset_starts_after_prompt() {
        let state = InputState::new();

        assert_eq!(state.cursor_offset(true), 1);
    }

    #[test]
    fn cursor_offset_stays_correct_after_history_and_clear_paths() {
        let mut state = InputState::new();
        state.insert_str("hé");
        let history_entry = state.take_displayed_input();
        state.push_history(history_entry);

        state.insert_str("🙂x");
        assert_eq!(state.cursor_offset(false), 3);

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "hé");
        assert_eq!(state.cursor_offset(false), 3);

        state.move_history(1);
        assert_eq!(state.displayed_input(), "🙂x");
        assert_eq!(state.cursor_offset(false), 3);

        state.clear_all();
        assert_eq!(state.displayed_input(), "");
        assert_eq!(state.cursor_offset(false), 1);
    }

    #[test]
    fn editing_history_entry_promotes_it_to_current_typed_input() {
        let mut state = InputState::new();
        state.push_history("look".to_string());
        state.insert_str("say");

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "look");

        state.insert_char('!');
        assert_eq!(state.displayed_input(), "look!");

        state.move_history(-1);
        assert_eq!(state.displayed_input(), "look");

        state.move_history(1);
        assert_eq!(state.displayed_input(), "look!");
    }
}
