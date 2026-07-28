use crate::generic_commands::{GenericCommandGroup, GenericCommands};

pub(crate) struct GenericCommandsDialog {
    /// Flattened navigation rows: `(group_index, command_index)`.
    /// `command_index == None` is a group header; `group_index == usize::MAX` is the synthetic "All Commands" row.
    items: Vec<(usize, Option<usize>)>,
    groups: Vec<GenericCommandGroup>,
    cursor: usize,
}

impl GenericCommandsDialog {
    pub(crate) fn new(generic: &GenericCommands) -> Self {
        let mut items: Vec<(usize, Option<usize>)> = Vec::new();
        let mut groups = Vec::new();

        items.push((usize::MAX, None));

        for (group_idx, group) in generic.groups.iter().enumerate() {
            items.push((group_idx, None));
            for cmd_idx in 0..group.commands.len() {
                items.push((group_idx, Some(cmd_idx)));
            }
            groups.push(group.clone());
        }

        Self {
            items,
            groups,
            cursor: 0,
        }
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        if self.items.is_empty() {
            return;
        }
        let max = self.items.len().saturating_sub(1) as i32;
        let next = (self.cursor as i32 + delta).clamp(0, max);
        self.cursor = next as usize;
    }

    pub(crate) fn toggle_selected(&mut self) {
        let Some((group_idx, cmd_idx)) = self.items.get(self.cursor).copied() else {
            return;
        };

        if let Some(cmd_idx) = cmd_idx {
            if let Some(group) = self.groups.get_mut(group_idx)
                && let Some(cmd) = group.commands.get_mut(cmd_idx)
            {
                cmd.enabled = !cmd.enabled;
            }
        } else if group_idx == usize::MAX {
            let all_enabled = self
                .groups
                .iter()
                .all(|g| g.selection_state() == Some(true));
            let new_state = !all_enabled;
            for group in &mut self.groups {
                for cmd in &mut group.commands {
                    cmd.enabled = new_state;
                }
            }
        } else {
            if let Some(group) = self.groups.get_mut(group_idx) {
                let all_enabled = group.selection_state() == Some(true);
                let new_state = !all_enabled;
                for cmd in &mut group.commands {
                    cmd.enabled = new_state;
                }
            }
        }
    }

    pub(crate) fn to_config(&self) -> (Vec<String>, Vec<String>) {
        let enabled_groups: Vec<String> = self
            .groups
            .iter()
            .filter(|g| g.selection_state() == Some(true))
            .map(|g| g.name.clone())
            .collect();

        let disabled_commands: Vec<String> = self
            .groups
            .iter()
            .flat_map(|g| {
                g.commands
                    .iter()
                    .filter(|cmd| !cmd.enabled)
                    .map(|cmd| cmd.alias.clone())
            })
            .collect();

        (enabled_groups, disabled_commands)
    }

    pub(crate) fn view_model(&self) -> crate::ui::GenericCommandsDialogViewModel {
        let view_items: Vec<crate::ui::GenericCommandViewModel> = self
            .items
            .iter()
            .map(|(group_idx, cmd_idx)| {
                if *group_idx == usize::MAX {
                    let all_enabled = self
                        .groups
                        .iter()
                        .all(|g| g.selection_state() == Some(true));
                    crate::ui::GenericCommandViewModel {
                        alias: "All Commands".to_string(),
                        command: String::new(),
                        selected: all_enabled,
                        level: 0,
                    }
                } else if let Some(cmd_idx) = cmd_idx {
                    let group = &self.groups[*group_idx];
                    let cmd = &group.commands[*cmd_idx];
                    crate::ui::GenericCommandViewModel {
                        alias: cmd.alias.clone(),
                        command: cmd.display_command(),
                        selected: cmd.enabled,
                        level: 2,
                    }
                } else {
                    let group = &self.groups[*group_idx];
                    let state = group.selection_state();
                    crate::ui::GenericCommandViewModel {
                        alias: group.display_name.clone(),
                        command: String::new(),
                        selected: state.unwrap_or(false),
                        level: 1,
                    }
                }
            })
            .collect();

        crate::ui::GenericCommandsDialogViewModel {
            items: view_items,
            cursor: self.cursor,
        }
    }
}
