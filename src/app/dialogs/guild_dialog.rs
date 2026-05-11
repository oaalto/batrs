use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

pub(crate) struct GuildDialog {
    definitions: Vec<crate::guilds::GuildDefinition>,
    selected: Vec<bool>,
    cursor: usize,
    mount_name: String,
    editing_mount: bool,
    mount_cursor: usize,
}

impl GuildDialog {
    pub(crate) fn new(
        definitions: Vec<crate::guilds::GuildDefinition>,
        selected: Vec<bool>,
        mount_name: String,
    ) -> Self {
        let mut selected = selected;
        if selected.len() < definitions.len() {
            selected.resize(definitions.len(), false);
        }
        let cursor = 0;
        let mount_cursor = mount_name.len();
        Self {
            definitions,
            selected,
            cursor,
            mount_name,
            editing_mount: false,
            mount_cursor,
        }
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        if self.definitions.is_empty() {
            return;
        }
        let max = self.definitions.len().saturating_sub(1) as i32;
        let next = (self.cursor as i32 + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    pub(crate) fn toggle_selected(&mut self) {
        if let Some(selected) = self.selected.get_mut(self.cursor) {
            *selected = !*selected;
        }
    }

    pub(crate) fn selected_keys(&self) -> Vec<String> {
        self.definitions
            .iter()
            .zip(self.selected.iter())
            .filter_map(|(def, selected)| selected.then_some(def.key.to_string()))
            .collect()
    }

    pub(crate) fn is_tzarakk_selected(&self) -> bool {
        self.definitions
            .iter()
            .zip(self.selected.iter())
            .any(|(def, sel)| def.key == "tzarakk" && *sel)
    }

    fn toggle_edit_mount(&mut self) {
        if self.is_tzarakk_selected() {
            self.editing_mount = !self.editing_mount;
        } else {
            self.editing_mount = false;
        }
    }

    fn enter_mount_edit_mode(&mut self) {
        if self.is_tzarakk_selected() {
            self.editing_mount = true;
        }
    }

    fn insert_mount_char(&mut self, c: char) {
        if self.editing_mount {
            self.mount_name.insert(self.mount_cursor, c);
            self.mount_cursor += 1;
        }
    }

    fn mount_backspace(&mut self) {
        if self.editing_mount && self.mount_cursor > 0 {
            self.mount_cursor -= 1;
            self.mount_name.remove(self.mount_cursor);
        }
    }

    fn mount_cursor_left(&mut self) {
        if self.editing_mount && self.mount_cursor > 0 {
            self.mount_cursor -= 1;
        }
    }

    fn mount_cursor_right(&mut self) {
        if self.editing_mount && self.mount_cursor < self.mount_name.len() {
            self.mount_cursor += 1;
        }
    }

    pub(crate) fn mount_name(&self) -> String {
        self.mount_name.clone()
    }

    pub(crate) fn view_model(&self) -> crate::ui::GuildDialogViewModel {
        crate::ui::GuildDialogViewModel {
            items: self
                .definitions
                .iter()
                .zip(self.selected.iter())
                .map(|(def, selected)| crate::ui::GuildDialogItem {
                    name: def.name.to_string(),
                    selected: *selected,
                })
                .collect(),
            cursor: self.cursor,
            mount_name: self.mount_name.clone(),
            show_mount_input: self.is_tzarakk_selected(),
            mount_input_cursor: self.mount_cursor,
        }
    }
}

pub(crate) fn apply_guild_dialog_keystroke(dialog: &mut GuildDialog, event: KeyEvent) {
    let text_modifiers_ok = !event.modifiers.contains(KeyModifiers::CONTROL)
        && !event.modifiers.contains(KeyModifiers::ALT);
    match event.code {
        KeyCode::Up if !dialog.editing_mount => {
            dialog.move_cursor(-1);
        }
        KeyCode::Down if !dialog.editing_mount => {
            dialog.move_cursor(1);
        }
        KeyCode::Char(' ') if !dialog.editing_mount => {
            dialog.toggle_selected();
        }
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('\t') => {
            dialog.toggle_edit_mount();
        }
        KeyCode::Char(c)
            if text_modifiers_ok
                && dialog.is_tzarakk_selected()
                && !dialog.editing_mount
                && !c.is_control() =>
        {
            dialog.enter_mount_edit_mode();
            dialog.insert_mount_char(c);
        }
        KeyCode::Char(c) if dialog.editing_mount && text_modifiers_ok => {
            dialog.insert_mount_char(c);
        }
        KeyCode::Backspace if dialog.editing_mount => {
            dialog.mount_backspace();
        }
        KeyCode::Left if dialog.editing_mount => {
            dialog.mount_cursor_left();
        }
        KeyCode::Right if dialog.editing_mount => {
            dialog.mount_cursor_right();
        }
        _ => {}
    }
}

#[cfg(test)]
mod guild_dialog_keystroke_tests {
    use super::{GuildDialog, apply_guild_dialog_keystroke};
    use crate::guilds::guild_definitions;
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn key_char(c: char) -> KeyEvent {
        KeyEvent::new_with_kind(KeyCode::Char(c), KeyModifiers::empty(), KeyEventKind::Press)
    }

    fn dialog_tzarakk_only() -> GuildDialog {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.key == "tzarakk")
            .collect();
        GuildDialog::new(definitions, selected, String::new())
    }

    #[test]
    fn typing_mount_name_without_tab_when_tzarakk_selected_inserts_text() {
        let mut dialog = dialog_tzarakk_only();
        apply_guild_dialog_keystroke(&mut dialog, key_char('V'));
        apply_guild_dialog_keystroke(&mut dialog, key_char('e'));
        apply_guild_dialog_keystroke(&mut dialog, key_char('d'));
        apply_guild_dialog_keystroke(&mut dialog, key_char('i'));
        apply_guild_dialog_keystroke(&mut dialog, key_char('r'));
        assert_eq!(dialog.mount_name(), "Vedir");
    }

    #[test]
    fn printable_keys_ignored_for_mount_when_tzarakk_not_selected() {
        let definitions = guild_definitions();
        let count = definitions.len();
        let mut dialog = GuildDialog::new(definitions, vec![false; count], String::new());
        apply_guild_dialog_keystroke(&mut dialog, key_char('x'));
        assert_eq!(dialog.mount_name(), "");
    }

    #[test]
    fn ctrl_letter_does_not_insert_into_mount_when_not_editing() {
        let mut dialog = dialog_tzarakk_only();
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(
                KeyCode::Char('v'),
                KeyModifiers::CONTROL,
                KeyEventKind::Press,
            ),
        );
        assert_eq!(dialog.mount_name(), "");
    }

    #[test]
    fn tab_char_toggle_mount_edit_when_tzarakk_selected() {
        let mut dialog = dialog_tzarakk_only();
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(
                KeyCode::Char('\t'),
                KeyModifiers::empty(),
                KeyEventKind::Press,
            ),
        );
        apply_guild_dialog_keystroke(&mut dialog, key_char('Z'));
        assert_eq!(dialog.mount_name(), "Z");
    }
}
