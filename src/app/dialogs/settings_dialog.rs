use crate::config::SettingEntry;

pub(crate) struct SettingsDialog {
    entries: Vec<SettingEntry>,
    cursor: usize,
}

impl SettingsDialog {
    pub(crate) fn new(entries: Vec<SettingEntry>) -> Self {
        let cursor = 0;
        Self { entries, cursor }
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        if self.entries.is_empty() {
            return;
        }
        let max = self.entries.len().saturating_sub(1) as i32;
        let next = (self.cursor as i32 + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    pub(crate) fn insert_char(&mut self, c: char) {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            entry.value.push(c);
        }
    }

    pub(crate) fn backspace(&mut self) {
        if let Some(entry) = self.entries.get_mut(self.cursor) {
            entry.value.pop();
        }
    }

    pub(crate) fn entries(&self) -> Vec<SettingEntry> {
        self.entries.clone()
    }

    pub(crate) fn view_model(&self) -> crate::ui::SettingsDialogViewModel {
        crate::ui::SettingsDialogViewModel {
            items: self
                .entries
                .iter()
                .map(|entry| crate::ui::SettingsDialogItem {
                    key: entry.key.clone(),
                    value: entry.value.clone(),
                })
                .collect(),
            cursor: self.cursor,
        }
    }
}
