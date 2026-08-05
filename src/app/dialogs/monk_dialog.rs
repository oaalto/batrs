use crate::guilds::{MonkSkillTrack, MonkSkillsConfig};

const KEY_HELP: &str = "Up/Down: move  Space: toggle  Enter: save  Esc: cancel";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) struct MonkSkillRow {
    pub track: MonkSkillTrack,
    pub slot: u8,
}

pub(crate) struct MonkDialog {
    saved: MonkSkillsConfig,
    draft: MonkSkillsConfig,
    cursor: usize,
    footer_error: Option<String>,
}

impl MonkDialog {
    pub(crate) fn new(config: &MonkSkillsConfig) -> Self {
        let saved = config.clone();
        Self {
            saved: saved.clone(),
            draft: saved,
            cursor: 0,
            footer_error: None,
        }
    }

    pub(crate) fn skill_rows() -> [MonkSkillRow; 12] {
        let mut rows = [MonkSkillRow {
            track: MonkSkillTrack::Disrupt,
            slot: 1,
        }; 12];
        let mut index = 0;
        for track in MonkSkillsConfig::TRACKS {
            for slot in 1..=3 {
                rows[index] = MonkSkillRow { track, slot };
                index += 1;
            }
        }
        rows
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        self.footer_error = None;
        let rows = Self::skill_rows();
        let max = rows.len().saturating_sub(1) as i32;
        let next = (self.cursor as i32 + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    pub(crate) fn toggle_selected(&mut self) {
        self.footer_error = None;
        let rows = Self::skill_rows();
        let Some(row) = rows.get(self.cursor) else {
            return;
        };
        self.draft.toggle_slot(row.track, row.slot);
    }

    pub(crate) fn draft_equals_saved(&self) -> bool {
        self.draft == self.saved
    }

    pub(crate) fn draft(&self) -> &MonkSkillsConfig {
        &self.draft
    }

    pub(crate) fn commit_saved(&mut self, config: MonkSkillsConfig) {
        self.saved = config.clone();
        self.draft = config;
    }

    pub(crate) fn set_footer_error(&mut self, message: impl Into<String>) {
        self.footer_error = Some(message.into());
    }

    pub(crate) fn view_model(&self) -> crate::ui::MonkDialogViewModel {
        let mut rows = Vec::new();
        for track in MonkSkillsConfig::TRACKS {
            rows.push(crate::ui::MonkDialogRowViewModel::Header(
                MonkSkillsConfig::track_label(track).to_string(),
            ));
            for slot in 1..=3 {
                let skill_name = MonkSkillsConfig::skill_name(track, slot).unwrap_or("");
                rows.push(crate::ui::MonkDialogRowViewModel::Skill {
                    label: format!("{slot} — {skill_name}"),
                    enabled: self.draft.slot_enabled(track, slot),
                });
            }
        }

        let skill_row_indices: Vec<usize> = rows
            .iter()
            .enumerate()
            .filter_map(|(index, row)| {
                if matches!(row, crate::ui::MonkDialogRowViewModel::Skill { .. }) {
                    Some(index)
                } else {
                    None
                }
            })
            .collect();
        let highlight_index = skill_row_indices.get(self.cursor).copied().unwrap_or(0);

        crate::ui::MonkDialogViewModel {
            rows,
            highlight_index,
            footer_line1: self
                .footer_error
                .clone()
                .unwrap_or_else(|| KEY_HELP.to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn toggle_mutates_draft_only() {
        let mut dialog = MonkDialog::new(&MonkSkillsConfig::default());
        dialog.toggle_selected();
        assert!(!dialog.draft().disrupt.slot_1);
        assert!(dialog.saved.disrupt.slot_1);
        assert!(!dialog.draft_equals_saved());
    }

    #[test]
    fn selecting_slot_3_enables_prefix_in_draft() {
        let mut dialog = MonkDialog::new(&MonkSkillsConfig::default());
        dialog.draft.disrupt.set_slot(1, false);
        dialog.draft.disrupt.set_slot(2, false);
        dialog.draft.disrupt.set_slot(3, false);
        dialog.cursor = 2;
        dialog.toggle_selected();
        assert!(dialog.draft().disrupt.slot_1);
        assert!(dialog.draft().disrupt.slot_2);
        assert!(dialog.draft().disrupt.slot_3);
    }
}
