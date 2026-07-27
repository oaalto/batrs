use crate::triggers::TriggerConfig;

const ROW_LABELS: [&str; 4] = [
    "Guild triggers",
    "Spell vocals",
    "Common triggers",
    "Core triggers",
];

const KEY_HELP: &str = "Up/Down: move  Space: toggle  Enter: save  Esc: cancel";
pub(crate) const CORE_DISABLED_WARNING: &str = "Prompt/stats parsing disabled";
pub(crate) const SAVE_ERROR_CONFIG_UNAVAILABLE: &str = "Player config not available";
pub(crate) const SAVE_ERROR_PERSIST_FAILED: &str = "Failed to save trigger settings";

pub(crate) struct TriggersDialog {
    saved: TriggerConfig,
    draft: TriggerConfig,
    cursor: usize,
    footer_error: Option<String>,
}

impl TriggersDialog {
    pub(crate) fn new(config: &TriggerConfig) -> Self {
        let saved = config.clone();
        Self {
            saved: saved.clone(),
            draft: saved,
            cursor: 0,
            footer_error: None,
        }
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        self.footer_error = None;
        let max = ROW_LABELS.len().saturating_sub(1) as i32;
        let next = (self.cursor as i32 + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    pub(crate) fn toggle_selected(&mut self) {
        self.footer_error = None;
        match self.cursor {
            0 => self.draft.guild_triggers = !self.draft.guild_triggers,
            1 => self.draft.spell_vocals = !self.draft.spell_vocals,
            2 => self.draft.common_triggers = !self.draft.common_triggers,
            3 => self.draft.core_triggers = !self.draft.core_triggers,
            _ => {}
        }
    }

    pub(crate) fn draft_equals_saved(&self) -> bool {
        self.draft == self.saved
    }

    pub(crate) fn draft(&self) -> &TriggerConfig {
        &self.draft
    }

    pub(crate) fn commit_saved(&mut self, config: TriggerConfig) {
        self.saved = config.clone();
        self.draft = config;
    }

    pub(crate) fn set_footer_error(&mut self, message: impl Into<String>) {
        self.footer_error = Some(message.into());
    }

    pub(crate) fn view_model(&self) -> crate::ui::TriggersDialogViewModel {
        let rows = ROW_LABELS
            .iter()
            .enumerate()
            .map(|(index, label)| {
                let enabled = match index {
                    0 => self.draft.guild_triggers,
                    1 => self.draft.spell_vocals,
                    2 => self.draft.common_triggers,
                    3 => self.draft.core_triggers,
                    _ => true,
                };
                crate::ui::TriggersDialogRowViewModel {
                    label: (*label).to_string(),
                    enabled,
                }
            })
            .collect();

        let footer_line2 = if self.draft.core_triggers {
            None
        } else {
            Some(CORE_DISABLED_WARNING.to_string())
        };

        crate::ui::TriggersDialogViewModel {
            rows,
            cursor: self.cursor,
            footer_line1: self
                .footer_error
                .clone()
                .unwrap_or_else(|| KEY_HELP.to_string()),
            footer_line2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enter_unchanged_draft_equals_saved() {
        let dialog = TriggersDialog::new(&TriggerConfig::default());
        assert!(dialog.draft_equals_saved());
    }

    #[test]
    fn toggle_mutates_draft_only() {
        let mut dialog = TriggersDialog::new(&TriggerConfig::default());
        dialog.toggle_selected();
        assert!(!dialog.draft.guild_triggers);
        assert!(dialog.saved.guild_triggers);
        assert!(!dialog.draft_equals_saved());
    }

    #[test]
    fn footer_shows_core_warning_when_disabled() {
        let mut dialog = TriggersDialog::new(&TriggerConfig::default());
        dialog.cursor = 3;
        dialog.toggle_selected();
        let vm = dialog.view_model();
        assert_eq!(vm.footer_line2.as_deref(), Some(CORE_DISABLED_WARNING));
    }
}
