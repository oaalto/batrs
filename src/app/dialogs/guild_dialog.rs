use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use crate::guilds::GuildDefinition;
use crate::guilds::catalog::GuildKey;
use crate::guilds::grouping::DEFAULT_GUILD_PRIMARY_KEYWORD;
use crate::guilds::grouping::{
    MULTI_BACKGROUND_LABEL, THEMES_UX_ORDER, clear_selected_outside_thematic_bucket,
    guild_grouping, thematic_index_for_keyword, visible_indices_multi_drill,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub(crate) enum GuildDialogFocus {
    #[default]
    GuildList,
    MountName,
    SabreWeapon,
    RiftwalkerEntities,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum GuildDrillSource {
    Thematic(usize),
    MultiOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum GuildDialogBrowseMode {
    PickBackground,
    DrillGuild,
}

#[derive(Clone, PartialEq, Eq)]
enum DrillRow {
    Banner(String),
    Toggle { definition_index: usize },
}

pub(crate) fn default_riftwalker_entity_labels() -> [String; 4] {
    std::array::from_fn(|_| "entity".to_string())
}

pub(crate) struct GuildDialog {
    definitions: Vec<GuildDefinition>,
    selected: Vec<bool>,
    /// Index in [`THEMES_UX_ORDER`] (0–4).
    active_primary: usize,
    browse_cursor: usize,
    mode: GuildDialogBrowseMode,
    drill_source: Option<GuildDrillSource>,
    drill_rows: Vec<DrillRow>,
    guild_cursor: usize,
    mount_name: String,
    mount_cursor: usize,
    sabre_weapon: String,
    sabre_weapon_cursor: usize,
    focus: GuildDialogFocus,
    riftwalker_labels: [String; 4],
    riftwalker_row: usize,
    riftwalker_cursors: [usize; 4],
}

impl GuildDialog {
    pub(crate) fn new(
        definitions: Vec<GuildDefinition>,
        selected: Vec<bool>,
        active_primary_keyword: &str,
        mount_name: String,
        sabre_weapon: String,
        riftwalker_labels: [String; 4],
    ) -> Self {
        let active_primary =
            thematic_index_for_keyword(active_primary_keyword).unwrap_or_else(|| {
                thematic_index_for_keyword(DEFAULT_GUILD_PRIMARY_KEYWORD)
                    .expect("DEFAULT_GUILD_PRIMARY_KEYWORD resolved")
            });

        let mut selected = selected;
        if selected.len() < definitions.len() {
            selected.resize(definitions.len(), false);
        }

        clear_selected_outside_thematic_bucket(
            &definitions,
            selected.as_mut_slice(),
            active_primary,
        );

        let mount_cursor = mount_name.len();
        let sabre_weapon_cursor = sabre_weapon.len();
        let riftwalker_cursors =
            std::array::from_fn(|index| riftwalker_labels[index].chars().count());
        Self {
            definitions,
            selected,
            active_primary,
            browse_cursor: active_primary.min(THEMES_UX_ORDER.len().saturating_sub(1)),
            mode: GuildDialogBrowseMode::PickBackground,
            drill_source: None,
            drill_rows: Vec::new(),
            guild_cursor: 0,
            mount_name,
            mount_cursor,
            sabre_weapon,
            sabre_weapon_cursor,
            focus: GuildDialogFocus::GuildList,
            riftwalker_labels,
            riftwalker_row: 0,
            riftwalker_cursors,
        }
    }

    pub(crate) fn is_browsing_backgrounds(&self) -> bool {
        self.mode == GuildDialogBrowseMode::PickBackground
    }

    pub(crate) fn open_drill_from_browse_cursor(&mut self) {
        let source = if self.browse_cursor < THEMES_UX_ORDER.len() {
            GuildDrillSource::Thematic(self.browse_cursor)
        } else {
            GuildDrillSource::MultiOnly
        };
        self.drill_source = Some(source);
        self.rebuild_drill_rows(source);
        self.mode = GuildDialogBrowseMode::DrillGuild;
        self.place_cursor_first_toggle_if_any();
        self.focus = GuildDialogFocus::GuildList;
    }

    pub(crate) fn back_to_browse(&mut self) {
        self.mode = GuildDialogBrowseMode::PickBackground;
        self.drill_source = None;
        self.drill_rows.clear();
        self.focus = GuildDialogFocus::GuildList;
    }

    pub(crate) fn active_primary_keyword(&self) -> &'static str {
        THEMES_UX_ORDER[self.active_primary].0
    }

    fn rebuild_drill_rows(&mut self, source: GuildDrillSource) {
        self.drill_rows.clear();

        match source {
            GuildDrillSource::Thematic(thematic_ix) => {
                let bucket = &guild_grouping().thematic[thematic_ix];
                let thematic_indices: Vec<usize> = bucket
                    .playable_def_indices
                    .iter()
                    .copied()
                    .filter(|definition_index| *definition_index < self.definitions.len())
                    .collect();

                let multi_filtered: Vec<usize> = visible_indices_multi_drill()
                    .into_iter()
                    .filter(|definition_index| *definition_index < self.definitions.len())
                    .collect();

                if thematic_indices.is_empty() && multi_filtered.is_empty() {
                    self.drill_rows.push(DrillRow::Banner(
                        "Nothing implemented yet for this thematic drill.".into(),
                    ));
                } else if thematic_indices.is_empty() {
                    self.drill_rows.push(DrillRow::Banner(
                        "(No playable guild in this thematic group yet)".into(),
                    ));
                    self.drill_rows
                        .push(DrillRow::Banner("Multi-background guilds".into()));
                }

                for &definition_index in &thematic_indices {
                    self.drill_rows.push(DrillRow::Toggle { definition_index });
                }

                if !thematic_indices.is_empty() && !multi_filtered.is_empty() {
                    self.drill_rows
                        .push(DrillRow::Banner("Multi-background guilds".into()));
                }

                for &definition_index in &multi_filtered {
                    self.drill_rows.push(DrillRow::Toggle { definition_index });
                }
            }
            GuildDrillSource::MultiOnly => {
                let multis: Vec<_> = visible_indices_multi_drill()
                    .into_iter()
                    .filter(|definition_index| *definition_index < self.definitions.len())
                    .collect();
                if multis.is_empty() {
                    self.drill_rows.push(DrillRow::Banner(
                        "No multi-background guilds implemented.".into(),
                    ));
                }
                for definition_index in multis {
                    self.drill_rows.push(DrillRow::Toggle { definition_index });
                }
            }
        }

        self.place_cursor_first_toggle_if_any();
    }

    fn place_cursor_first_toggle_if_any(&mut self) {
        if let Some(ix) = closest_toggle_forward(self.drill_rows.as_slice(), 0usize, true) {
            self.guild_cursor = ix;
        } else if self.drill_rows.is_empty() {
            self.guild_cursor = 0;
        }
    }

    pub(crate) fn mount_name(&self) -> String {
        self.mount_name.clone()
    }

    pub(crate) fn sabre_weapon(&self) -> String {
        self.sabre_weapon.clone()
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

    fn toggle_at_cursor(&mut self) {
        if self.mode != GuildDialogBrowseMode::DrillGuild {
            return;
        }
        let Some(entry) = self.drill_rows.get(self.guild_cursor) else {
            return;
        };
        let definition_index = match entry {
            DrillRow::Toggle { definition_index } => *definition_index,
            DrillRow::Banner(_) => return,
        };
        if let Some(selection) = self.selected.get_mut(definition_index) {
            *selection = !*selection;
            self.adjust_focus_targets();
        }
    }

    fn move_drill_cursor(&mut self, delta: i32) {
        if self.drill_rows.is_empty() {
            return;
        }
        let start = self
            .guild_cursor
            .min(self.drill_rows.len().saturating_sub(1));

        let next =
            closest_toggle_relative(self.drill_rows.as_slice(), start, delta).unwrap_or(start);
        self.guild_cursor = next;
    }

    fn apply_thematic_primary_selection(&mut self) {
        if self.browse_cursor >= THEMES_UX_ORDER.len() {
            return;
        }
        self.active_primary = self.browse_cursor;
        clear_selected_outside_thematic_bucket(
            &self.definitions,
            &mut self.selected,
            self.active_primary,
        );
    }

    fn move_browse_cursor(&mut self, delta: i32) {
        let last_index = THEMES_UX_ORDER.len();
        let next = (self.browse_cursor as i32 + delta).clamp(0i32, last_index as i32) as usize;
        self.browse_cursor = next;
    }

    pub(crate) fn move_cursor(&mut self, delta: i32) {
        match self.mode {
            GuildDialogBrowseMode::PickBackground => self.move_browse_cursor(delta),
            GuildDialogBrowseMode::DrillGuild => self.move_drill_cursor(delta),
        }
    }

    pub(crate) fn toggle_selected(&mut self) {
        match self.mode {
            GuildDialogBrowseMode::PickBackground => self.apply_thematic_primary_selection(),
            GuildDialogBrowseMode::DrillGuild => self.toggle_at_cursor(),
        }
        self.adjust_focus_targets();
    }

    pub(crate) fn selected_keys(&self) -> Vec<String> {
        self.definitions
            .iter()
            .zip(self.selected.iter())
            .filter_map(|(definition, selected)| selected.then_some(definition.key.to_string()))
            .collect()
    }

    fn is_guild_selected(&self, guild_key: GuildKey) -> bool {
        let Some(position) = self
            .definitions
            .iter()
            .position(|definition| definition.guild_key == guild_key)
        else {
            return false;
        };
        self.selected.get(position).is_some_and(|value| *value)
    }

    pub(crate) fn is_tzarakk_selected(&self) -> bool {
        self.is_guild_selected(GuildKey::Tzarakk)
    }

    pub(crate) fn is_sabres_selected(&self) -> bool {
        self.is_guild_selected(GuildKey::Sabres)
    }

    pub(crate) fn is_riftwalker_selected(&self) -> bool {
        self.is_guild_selected(GuildKey::Riftwalker)
    }

    fn adjust_focus_targets(&mut self) {
        if self.focus == GuildDialogFocus::MountName && !self.is_tzarakk_selected() {
            self.focus = self.first_aux_focus_after_mount();
        }
        if self.focus == GuildDialogFocus::SabreWeapon && !self.is_sabres_selected() {
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

    fn first_aux_focus_after_mount(&self) -> GuildDialogFocus {
        if self.is_sabres_selected() {
            GuildDialogFocus::SabreWeapon
        } else if self.is_riftwalker_selected() {
            GuildDialogFocus::RiftwalkerEntities
        } else {
            GuildDialogFocus::GuildList
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
                } else if self.is_sabres_selected() {
                    self.focus = GuildDialogFocus::SabreWeapon;
                } else if self.is_riftwalker_selected() {
                    self.focus = GuildDialogFocus::RiftwalkerEntities;
                    self.riftwalker_row = 0;
                }
            }
            GuildDialogFocus::MountName => {
                if self.is_sabres_selected() {
                    self.focus = GuildDialogFocus::SabreWeapon;
                } else if self.is_riftwalker_selected() {
                    self.focus = GuildDialogFocus::RiftwalkerEntities;
                    self.riftwalker_row = 0;
                } else {
                    self.focus = GuildDialogFocus::GuildList;
                }
            }
            GuildDialogFocus::SabreWeapon => {
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
                } else if self.is_sabres_selected() {
                    self.focus = GuildDialogFocus::SabreWeapon;
                } else if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                }
            }
            GuildDialogFocus::MountName => {
                self.focus = GuildDialogFocus::GuildList;
            }
            GuildDialogFocus::SabreWeapon => {
                if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                } else {
                    self.focus = GuildDialogFocus::GuildList;
                }
            }
            GuildDialogFocus::RiftwalkerEntities => {
                if self.is_sabres_selected() {
                    self.focus = GuildDialogFocus::SabreWeapon;
                } else if self.is_tzarakk_selected() {
                    self.focus = GuildDialogFocus::MountName;
                } else {
                    self.focus = GuildDialogFocus::GuildList;
                }
            }
        }
    }

    fn insert_mount_char(&mut self, character: char) {
        self.mount_name.insert(self.mount_cursor, character);
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

    fn insert_sabre_weapon_char(&mut self, character: char) {
        self.sabre_weapon
            .insert(self.sabre_weapon_cursor, character);
        self.sabre_weapon_cursor += 1;
    }

    fn sabre_weapon_backspace(&mut self) {
        if self.sabre_weapon_cursor > 0 {
            self.sabre_weapon_cursor -= 1;
            self.sabre_weapon.remove(self.sabre_weapon_cursor);
        }
    }

    fn sabre_weapon_cursor_left(&mut self) {
        if self.sabre_weapon_cursor > 0 {
            self.sabre_weapon_cursor -= 1;
        }
    }

    fn sabre_weapon_cursor_right(&mut self) {
        if self.sabre_weapon_cursor < self.sabre_weapon.len() {
            self.sabre_weapon_cursor += 1;
        }
    }

    fn insert_riftwalker_char(&mut self, character: char) {
        let row = self.riftwalker_row;
        let cursor_slot = self.riftwalker_cursors[row];
        let mut symbols: Vec<char> = self.riftwalker_labels[row].chars().collect();
        symbols.insert(cursor_slot, character);
        self.riftwalker_labels[row] = symbols.into_iter().collect();
        self.riftwalker_cursors[row] += 1;
    }

    fn riftwalker_backspace(&mut self) {
        let row = self.riftwalker_row;
        let cursor_slot = self.riftwalker_cursors[row];
        if cursor_slot > 0 {
            let mut symbols: Vec<char> = self.riftwalker_labels[row].chars().collect();
            symbols.remove(cursor_slot - 1);
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
        let length = self.riftwalker_labels[row].chars().count();
        if self.riftwalker_cursors[row] < length {
            self.riftwalker_cursors[row] += 1;
        }
    }

    pub(crate) fn view_model(&self) -> crate::ui::GuildDialogViewModel {
        const ENTITY_TITLES: [&str; 4] = ["Fire", "Air", "Water", "Earth"];

        let riftwalker_entity_rows: Vec<crate::ui::RiftwalkerEntityRowVm> = ENTITY_TITLES
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

        let phase_present = match self.mode {
            GuildDialogBrowseMode::PickBackground => {
                let browse_labels = THEMES_UX_ORDER
                    .iter()
                    .map(|(_, ui_label)| ui_label.to_string())
                    .chain(std::iter::once(MULTI_BACKGROUND_LABEL.into()))
                    .collect::<Vec<_>>();
                crate::ui::GuildDialogPresentation::BrowseRows {
                    labels: browse_labels,
                    cursor: self.browse_cursor,
                    active_primary_index: self.active_primary,
                }
            }
            GuildDialogBrowseMode::DrillGuild => {
                let subtitle = match self.drill_source {
                    Some(GuildDrillSource::Thematic(ix)) => {
                        guild_grouping().thematic[ix].label.to_string()
                    }
                    Some(GuildDrillSource::MultiOnly) => MULTI_BACKGROUND_LABEL.into(),
                    None => "".into(),
                };

                let lines = self.drill_rows_iter().collect::<Vec<_>>();
                crate::ui::GuildDialogPresentation::GuildDrill {
                    subtitle,
                    lines,
                    cursor: self.guild_cursor,
                }
            }
        };

        crate::ui::GuildDialogViewModel {
            presentation: phase_present,
            mount_name: self.mount_name.clone(),
            show_mount_input: self.is_tzarakk_selected(),
            mount_input_cursor: self.mount_cursor,
            mount_input_focused: self.focus == GuildDialogFocus::MountName,
            sabre_weapon: self.sabre_weapon.clone(),
            show_sabre_weapon_input: self.is_sabres_selected(),
            sabre_weapon_input_cursor: self.sabre_weapon_cursor,
            sabre_weapon_input_focused: self.focus == GuildDialogFocus::SabreWeapon,
            show_riftwalker_entity_inputs: self.is_riftwalker_selected(),
            riftwalker_rows: riftwalker_entity_rows,
        }
    }

    fn drill_rows_iter(&self) -> impl Iterator<Item = crate::ui::GuildDrillLineVm> + '_ {
        self.drill_rows.iter().map(|entry| match entry {
            DrillRow::Banner(text) => crate::ui::GuildDrillLineVm::Banner(text.clone()),
            DrillRow::Toggle { definition_index } => {
                let definition = &self.definitions[*definition_index];
                crate::ui::GuildDrillLineVm::Guild {
                    title: definition.name.to_string(),
                    selected: self
                        .selected
                        .get(*definition_index)
                        .copied()
                        .unwrap_or(false),
                }
            }
        })
    }
}

fn closest_toggle_relative(rows: &[DrillRow], from: usize, delta: i32) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    let from = from.min(rows.len().saturating_sub(1));

    match delta.cmp(&0i32) {
        std::cmp::Ordering::Equal => {
            if matches!(rows[from], DrillRow::Toggle { .. }) {
                Some(from)
            } else {
                closest_toggle_forward(rows, from, false)
            }
        }
        std::cmp::Ordering::Greater => closest_toggle_forward(rows, from, false),
        std::cmp::Ordering::Less => from
            .checked_sub(1)
            .and_then(|previous_row| closest_toggle_backward(rows, previous_row))
            .or_else(|| closest_toggle_forward(rows, 0usize, true)),
    }
}

fn closest_toggle_forward(rows: &[DrillRow], start: usize, include_start: bool) -> Option<usize> {
    let start = start.min(rows.len());
    let begin = if include_start {
        start
    } else {
        start.saturating_add(1)
    }
    .min(rows.len());

    rows.iter()
        .enumerate()
        .skip(begin)
        .find_map(|(index, row)| matches!(row, DrillRow::Toggle { .. }).then_some(index))
}

fn closest_toggle_backward(rows: &[DrillRow], start: usize) -> Option<usize> {
    if rows.is_empty() {
        return None;
    }

    rows.iter()
        .enumerate()
        .take(start.saturating_add(1))
        .rfind(|(_, row)| matches!(row, DrillRow::Toggle { .. }))
        .map(|(idx, _)| idx)
}

pub(crate) fn apply_guild_dialog_keystroke(dialog: &mut GuildDialog, event: KeyEvent) {
    let textual_mod_ok = !event.modifiers.contains(KeyModifiers::CONTROL)
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
            KeyCode::Up => dialog.move_cursor(-1),
            KeyCode::Down => dialog.move_cursor(1),
            KeyCode::Char(' ') => dialog.toggle_selected(),
            KeyCode::Char(character)
                if textual_mod_ok
                    && matches!(dialog.mode, GuildDialogBrowseMode::DrillGuild)
                    && dialog.is_tzarakk_selected()
                    && !character.is_control() =>
            {
                dialog.focus = GuildDialogFocus::MountName;
                dialog.insert_mount_char(character);
            }
            KeyCode::Char(character)
                if textual_mod_ok
                    && matches!(dialog.mode, GuildDialogBrowseMode::DrillGuild)
                    && dialog.is_sabres_selected()
                    && !dialog.is_tzarakk_selected()
                    && !character.is_control() =>
            {
                dialog.focus = GuildDialogFocus::SabreWeapon;
                dialog.insert_sabre_weapon_char(character);
            }
            _ => {}
        },
        GuildDialogFocus::MountName => match event.code {
            KeyCode::Char(character) if textual_mod_ok && !character.is_control() => {
                dialog.insert_mount_char(character);
            }
            KeyCode::Backspace => dialog.mount_backspace(),
            KeyCode::Left => dialog.mount_cursor_left(),
            KeyCode::Right => dialog.mount_cursor_right(),
            _ => {}
        },
        GuildDialogFocus::SabreWeapon => match event.code {
            KeyCode::Char(character) if textual_mod_ok && !character.is_control() => {
                dialog.insert_sabre_weapon_char(character);
            }
            KeyCode::Backspace => dialog.sabre_weapon_backspace(),
            KeyCode::Left => dialog.sabre_weapon_cursor_left(),
            KeyCode::Right => dialog.sabre_weapon_cursor_right(),
            _ => {}
        },
        GuildDialogFocus::RiftwalkerEntities => match event.code {
            KeyCode::Up if dialog.riftwalker_row > 0 => {
                dialog.riftwalker_row -= 1;
            }
            KeyCode::Down if dialog.riftwalker_row < 3 => {
                dialog.riftwalker_row += 1;
            }
            KeyCode::Char(character) if textual_mod_ok && !character.is_control() => {
                dialog.insert_riftwalker_char(character);
            }
            KeyCode::Backspace => dialog.riftwalker_backspace(),
            KeyCode::Left => dialog.riftwalker_cursor_left(),
            KeyCode::Right => dialog.riftwalker_cursor_right(),
            _ => {}
        },
    }
}

#[cfg(test)]
mod guild_dialog_keystroke_tests {
    use super::{GuildDialog, GuildDialogFocus, apply_guild_dialog_keystroke};
    use crate::guilds::catalog::GuildKey;
    use crate::guilds::grouping::{DEFAULT_GUILD_PRIMARY_KEYWORD, THEMES_UX_ORDER};
    use crate::guilds::guild_definitions;
    use crate::ui::{GuildDialogPresentation, GuildDrillLineVm};
    use crossterm::event::{KeyCode, KeyEvent, KeyEventKind, KeyModifiers};

    fn key_code(code: KeyCode) -> KeyEvent {
        KeyEvent::new_with_kind(code, KeyModifiers::empty(), KeyEventKind::Press)
    }

    fn key_character(symbol: char) -> KeyEvent {
        KeyEvent::new_with_kind(
            KeyCode::Char(symbol),
            KeyModifiers::empty(),
            KeyEventKind::Press,
        )
    }

    fn evil_keyword() -> &'static str {
        THEMES_UX_ORDER
            .iter()
            .copied()
            .find(|(keyword, _)| *keyword == "evil_religious")
            .map(|pair| pair.0)
            .expect("evil_religious")
    }

    fn magical_keyword() -> &'static str {
        THEMES_UX_ORDER
            .iter()
            .copied()
            .find(|(keyword, _)| *keyword == "magical")
            .map(|pair| pair.0)
            .expect("magical")
    }

    fn drill_cursor_and_guild_rows(dialog: &GuildDialog) -> (usize, Vec<usize>) {
        match dialog.view_model().presentation {
            GuildDialogPresentation::GuildDrill { lines, cursor, .. } => {
                let guild_rows = lines
                    .iter()
                    .enumerate()
                    .filter_map(|(row_index, line)| {
                        matches!(line, GuildDrillLineVm::Guild { .. }).then_some(row_index)
                    })
                    .collect();
                (cursor, guild_rows)
            }
            GuildDialogPresentation::BrowseRows { .. } => panic!("expected guild drill view"),
        }
    }

    #[test]
    fn down_moves_one_guild_at_a_time_and_reaches_final_drill_row() {
        let definitions = guild_definitions();
        let count = definitions.len();
        let mut dialog = GuildDialog::new(
            definitions,
            vec![false; count],
            magical_keyword(),
            String::new(),
            String::new(),
            super::default_riftwalker_entity_labels(),
        );

        dialog.browse_cursor = THEMES_UX_ORDER
            .iter()
            .position(|(canonical, _)| *canonical == "magical")
            .expect("magical");
        dialog.open_drill_from_browse_cursor();

        let (initial_cursor, guild_rows) = drill_cursor_and_guild_rows(&dialog);
        assert!(
            guild_rows.len() >= 3,
            "regression test requires at least three guild rows"
        );
        assert_eq!(initial_cursor, guild_rows[0]);

        apply_guild_dialog_keystroke(&mut dialog, key_code(KeyCode::Down));
        assert_eq!(drill_cursor_and_guild_rows(&dialog).0, guild_rows[1]);

        for expected_row in guild_rows.iter().skip(2) {
            apply_guild_dialog_keystroke(&mut dialog, key_code(KeyCode::Down));
            assert_eq!(drill_cursor_and_guild_rows(&dialog).0, *expected_row);
        }

        apply_guild_dialog_keystroke(&mut dialog, key_code(KeyCode::Down));
        assert_eq!(
            drill_cursor_and_guild_rows(&dialog).0,
            *guild_rows.last().expect("guild row")
        );
    }

    #[test]
    fn typing_mount_when_tzarakk_selected_drill_moves_focus_mount() {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.guild_key == GuildKey::Tzarakk)
            .collect();
        let mut dialog = GuildDialog::new(
            definitions,
            selected,
            evil_keyword(),
            String::new(),
            String::new(),
            super::default_riftwalker_entity_labels(),
        );
        dialog.open_drill_from_browse_cursor();
        apply_guild_dialog_keystroke(&mut dialog, key_character('v'));
        assert_eq!(dialog.mount_name(), "v");
        assert_eq!(dialog.focus(), GuildDialogFocus::MountName);
    }

    #[test]
    fn printable_ignored_mount_browse_tzarakk_not_drilled_yet() {
        let definitions = guild_definitions();
        let count = definitions.len();
        let mut dialog = GuildDialog::new(
            definitions,
            vec![false; count],
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            String::new(),
            String::new(),
            super::default_riftwalker_entity_labels(),
        );
        apply_guild_dialog_keystroke(&mut dialog, key_character('x'));
        assert_eq!(dialog.mount_name(), "");
    }

    #[test]
    fn tab_advances_into_mount_tzarakk_drill() {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.guild_key == GuildKey::Tzarakk)
            .collect();
        let mut dialog = GuildDialog::new(
            definitions,
            selected,
            evil_keyword(),
            String::new(),
            String::new(),
            super::default_riftwalker_entity_labels(),
        );
        dialog.open_drill_from_browse_cursor();
        assert_eq!(dialog.focus(), GuildDialogFocus::GuildList);
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Tab, KeyModifiers::empty(), KeyEventKind::Press),
        );
        assert_eq!(dialog.focus(), GuildDialogFocus::MountName);
    }

    #[test]
    fn tab_cycles_riftwalker_labels_after_magical_drill() {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.guild_key == GuildKey::Riftwalker)
            .collect();
        let empty_riftwalker = std::array::from_fn(|_| String::new());
        let mut dialog = GuildDialog::new(
            definitions,
            selected,
            magical_keyword(),
            String::new(),
            String::new(),
            empty_riftwalker,
        );

        dialog.browse_cursor = THEMES_UX_ORDER
            .iter()
            .position(|(canonical, _)| *canonical == "magical")
            .expect("magical");
        dialog.open_drill_from_browse_cursor();

        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Tab, KeyModifiers::empty(), KeyEventKind::Press),
        );
        assert_eq!(dialog.focus(), GuildDialogFocus::RiftwalkerEntities);
        apply_guild_dialog_keystroke(&mut dialog, key_character('a'));
        apply_guild_dialog_keystroke(
            &mut dialog,
            KeyEvent::new_with_kind(KeyCode::Down, KeyModifiers::empty(), KeyEventKind::Press),
        );
        apply_guild_dialog_keystroke(&mut dialog, key_character('b'));
        assert_eq!(dialog.riftwalker_entity_labels()[0], "a");
        assert_eq!(dialog.riftwalker_entity_labels()[1], "b");
    }

    #[test]
    fn ctrl_letter_does_not_mount_insert() {
        let definitions = guild_definitions();
        let selected = definitions
            .iter()
            .map(|definition| definition.guild_key == GuildKey::Tzarakk)
            .collect();
        let mut dialog = GuildDialog::new(
            definitions,
            selected,
            evil_keyword(),
            String::new(),
            String::new(),
            super::default_riftwalker_entity_labels(),
        );
        dialog.open_drill_from_browse_cursor();
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
}
