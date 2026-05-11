use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum GuildDialogFocus {
    #[default]
    GuildList,
    MountName,
    RiftwalkerEntities,
}

pub(crate) fn default_riftwalker_entity_labels() -> [String; 4] {
    std::array::from_fn(|_| String::new())
}

pub(crate) struct GuildDialog {
    definitions: Vec<crate::guilds::GuildDefinition>,
    selected: Vec<bool>,
    cursor: usize,
    mount_name: String,
    mount_cursor: usize,
    focus: GuildDialogFocus,
    riftwalker_labels: [String; 4],
    riftwalker_row: usize,
    riftwalker_cursors: [usize; 4],
}

impl GuildDialog {
    pub(crate) fn new(
        definitions: Vec<crate::guilds::GuildDefinition>,
        selected: Vec<bool>,
        mount_name: String,
        riftwalker_labels: [String; 4],
    ) -> Self {
        let mut selected = selected;
        if selected.len() < definitions.len() {
            selected.resize(definitions.len(), false);
        }
        let cursor = 0;
        let mount_cursor = mount_name.len();
        let riftwalker_cursors =
            std::array::from_fn(|index| riftwalker_labels[index].chars().count());
        Self {
            definitions,
            selected,
            cursor,
            mount_name,
            mount_cursor,
            focus: GuildDialogFocus::GuildList,
            riftwalker_labels,
            riftwalker_row: 0,
            riftwalker_cursors,
        }
    }

    pub(crate) fn mount_name(&self) -> String {
        self.mount_name.clone()
    }

    pub(crate) fn riftwalker_entity_labels(&self) -> [String; 4] {
        [
            self.riftwalker_labels[0].clone(),
            self.riftwalker_labels[1].clone(),
            self.riftwalker_labels[2].clone(),
            self.riftwalker_labels[3].clone(),
        ]
    }

    #[cfg(test)]
    pub(crate) fn focus(&self) -> GuildDialogFocus {
        self.focus
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
        self.adjust_focus_targets();
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

    pub(crate) fn is_riftwalker_selected(&self) -> bool {
        self.definitions
            .iter()
            .zip(self.selected.iter())
            .any(|(def, sel)| def.key == "riftwalker" && *sel)
    }

    fn adjust_focus_targets(&mut self) {
        if self.focus == GuildDialogFocus::MountName && !self.is_tzarakk_selected() {
            self.focus = if self.is_riftwalker_selected() {
                GuildDialogFocus::RiftwalkerEntities
            } else {
                GuildDialogFocus::GuildList
            };
        }
        if self.focus == GuildDialogFocus::RiftwalkerEntities && !self.is_riftwalker_selected() {
            self.focus = GuildDialogFocus::GuildList;
        }
    }

    fn reset_focus_if_invalid(&mut self) {
        self.adjust_focus_targets();
    }

    fn advance_focus_forward(&mut self) {
        self.reset_focus_if_invalid();
        match self.focus {
            GuildDialogFocus::GuildList => {
                if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                } else if self.is_riftwalker_selected() {
                    self.focus = GuildDialogFocus::RiftwalkerEntities;
                    self.riftwalker_row = 0;
                }
            }
            GuildDialogFocus::MountName => {
                if self.is_riftwalker_selected() {
                    self.focus = GuildDialogFocus::RiftwalkerEntities;
                    self.riftwalker_row = 0;
                } else {
                    self.focus = GuildDialogFocus::GuildList;
                }
            }
            GuildDialogFocus::RiftwalkerEntities => {
                self.focus = GuildDialogFocus::GuildList;
            }
        }
    }

    fn advance_focus_backward(&mut self) {
        self.reset_focus_if_invalid();
        match self.focus {
            GuildDialogFocus::GuildList => {
                if self.is_riftwalker_selected() {
                    self.focus = GuildDialogFocus::RiftwalkerEntities;
                    self.riftwalker_row = 3;
                } else if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                }
            }
            GuildDialogFocus::MountName => {
                self.focus = GuildDialogFocus::GuildList;
            }
            GuildDialogFocus::RiftwalkerEntities => {
                if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                } else {
                    self.focus = GuildDialogFocus::GuildList;
                }
            }
        }
    }

    fn insert_mount_char(&mut self, c: char) {
        self.mount_name.insert(self.mount_cursor, c);
        self.mount_cursor += 1;
    }

    fn mount_backspace(&mut self) {
        if self.mount_cursor > 0 {
            self.mount_cursor -= 1;
            self.mount_name.remove(self.mount_cursor);
        }
    }

    fn mount_cursor_left(&mut self) {
        if self.mount_cursor > 0 {
            self.mount_cursor -= 1;
        }
    }

    fn mount_cursor_right(&mut self) {
        if self.mount_cursor < self.mount_name.len() {
            self.mount_cursor += 1;
        }
    }

    fn insert_riftwalker_char(&mut self, c: char) {
        let row = self.riftwalker_row;
        let cursor = self.riftwalker_cursors[row];
        let mut symbols: Vec<char> = self.riftwalker_labels[row].chars().collect();
        symbols.insert(cursor, c);
        self.riftwalker_labels[row] = symbols.into_iter().collect();
        self.riftwalker_cursors[row] += 1;
    }

    fn riftwalker_backspace(&mut self) {
        let row = self.riftwalker_row;
        let cursor = self.riftwalker_cursors[row];
        if cursor > 0 {
            let mut symbols: Vec<char> = self.riftwalker_labels[row].chars().collect();
            symbols.remove(cursor - 1);
            self.riftwalker_labels[row] = symbols.into_iter().collect();
            self.riftwalker_cursors[row] -= 1;
        }
    }

    fn riftwalker_cursor_left(&mut self) {
        let row = self.riftwalker_row;
        if self.riftwalker_cursors[row] > 0 {
            self.riftwalker_cursors[row] -= 1;
        }
    }

    fn riftwalker_cursor_right(&mut self) {
        let row = self.riftwalker_row;
        let len = self.riftwalker_labels[row].chars().count();
        if self.riftwalker_cursors[row] < len {
            self.riftwalker_cursors[row] += 1;
        }
    }

    pub(crate) fn view_model(&self) -> crate::ui::GuildDialogViewModel {
        const RIFT_ROW_TITLES: [&str; 4] = ["Fire", "Air", "Water", "Earth"];
        let riftwalker_rows: Vec<crate::ui::RiftwalkerEntityRowVm> = RIFT_ROW_TITLES
            .iter()
            .enumerate()
            .map(|(row_index, &title)| crate::ui::RiftwalkerEntityRowVm {
                title,
                value: self.riftwalker_labels[row_index].clone(),
                cursor: self.riftwalker_cursors[row_index],
                active_row: self.focus == GuildDialogFocus::RiftwalkerEntities
                    && self.riftwalker_row == row_index,
            })
            .collect();

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
            mount_input_focused: self.focus == GuildDialogFocus::MountName,
            show_riftwalker_entity_inputs: self.is_riftwalker_selected(),
            riftwalker_rows,
        }
    }
}

pub(crate) fn apply_guild_dialog_keystroke(dialog: &mut GuildDialog, event: KeyEvent) {
    let text_modifiers_ok = !event.modifiers.contains(KeyModifiers::CONTROL)
        && !event.modifiers.contains(KeyModifiers::ALT);
    dialog.reset_focus_if_invalid();

    match event.code {
        KeyCode::Tab | KeyCode::Char('\t') if !event.modifiers.contains(KeyModifiers::SHIFT) => {
            dialog.advance_focus_forward();
            return;
        }
        KeyCode::BackTab | KeyCode::Tab if event.modifiers.contains(KeyModifiers::SHIFT) => {
            dialog.advance_focus_backward();
            return;
        }
        _ => {}
    }

    match dialog.focus {
        GuildDialogFocus::GuildList => match event.code {
            KeyCode::Up => {
                dialog.move_cursor(-1);
            }
            KeyCode::Down => {
                dialog.move_cursor(1);
            }
            KeyCode::Char(' ') => {
                dialog.toggle_selected();
            }
            KeyCode::Char(c)
                if text_modifiers_ok && dialog.is_tzarakk_selected() && !c.is_control() =>
            {
                dialog.focus = GuildDialogFocus::MountName;
                dialog.insert_mount_char(c);
            }
            _ => {}
        },
        GuildDialogFocus::MountName => match event.code {
            KeyCode::Char(c) if text_modifiers_ok && !c.is_control() => {
                dialog.insert_mount_char(c);
            }
            KeyCode::Backspace => {
                dialog.mount_backspace();
            }
            KeyCode::Left => {
                dialog.mount_cursor_left();
            }
            KeyCode::Right => {
                dialog.mount_cursor_right();
            }
            _ => {}
        },
        GuildDialogFocus::RiftwalkerEntities => match event.code {
            KeyCode::Up if dialog.riftwalker_row > 0 => {
                dialog.riftwalker_row -= 1;
            }
            KeyCode::Down if dialog.riftwalker_row < 3 => {
                dialog.riftwalker_row += 1;
            }
            KeyCode::Char(c) if text_modifiers_ok && !c.is_control() => {
                dialog.insert_riftwalker_char(c);
            }
            KeyCode::Backspace => {
                dialog.riftwalker_backspace();
            }
            KeyCode::Left => {
                dialog.riftwalker_cursor_left();
            }
            KeyCode::Right => {
                dialog.riftwalker_cursor_right();
            }
            _ => {}
        },
    }
}

#[cfg(test)]
mod guild_dialog_keystroke_tests {
    use super::{GuildDialog, GuildDialogFocus, apply_guild_dialog_keystroke};
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
        GuildDialog::new(
            definitions,
            selected,
            String::new(),
            super::default_riftwalker_entity_labels(),
        )
    }

    fn dialog_riftwalker_only() -> GuildDialog {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.key == "riftwalker")
            .collect();
        GuildDialog::new(
            definitions,
            selected,
            String::new(),
            super::default_riftwalker_entity_labels(),
        )
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
        assert_eq!(dialog.focus(), GuildDialogFocus::MountName);
    }

    #[test]
    fn printable_keys_ignored_for_mount_when_tzarakk_not_selected() {
        let definitions = guild_definitions();
        let count = definitions.len();
        let mut dialog = GuildDialog::new(
            definitions,
            vec![false; count],
            String::new(),
            super::default_riftwalker_entity_labels(),
        );
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
    fn tab_advances_focus_from_list_to_mount_when_tzarakk_selected() {
        let mut dialog = dialog_tzarakk_only();
        assert_eq!(dialog.focus(), GuildDialogFocus::GuildList);
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Tab, KeyModifiers::empty(), KeyEventKind::Press),
        );
        assert_eq!(dialog.focus(), GuildDialogFocus::MountName);
    }

    #[test]
    fn tab_cycles_riftwalker_labels() {
        let mut dialog = dialog_riftwalker_only();
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Tab, KeyModifiers::empty(), KeyEventKind::Press),
        );
        assert_eq!(dialog.focus(), GuildDialogFocus::RiftwalkerEntities);
        apply_guild_dialog_keystroke(&mut dialog, key_char('a'));
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::empty(), KeyEventKind::Press),
        );
        apply_guild_dialog_keystroke(&mut dialog, key_char('b'));
        assert_eq!(dialog.riftwalker_entity_labels()[0], "a");
        assert_eq!(dialog.riftwalker_entity_labels()[1], "b");
    }
}
