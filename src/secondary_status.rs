use crate::ansi::palette;
use crate::guilds::catalog::{GuildKey, GuildSelection};
use crate::stats::progress_color;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

#[derive(Default, Debug, Clone)]
pub struct SecondaryStatus {
    soul_companion: Option<SoulCompanionStatus>,
    tzarakk_mount: Option<TzarakkMountStatus>,
    riftwalker_entity: Option<RiftwalkerEntityStatus>,
    nergal_minions: [Option<NergalMinion>; 3],
    nergal_resource_status: Option<NergalResourceStatus>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SecondaryStatusEffect {
    SetSoulCompanion {
        percent: i32,
        description: String,
    },
    SetTzarakkMountStatus {
        name: String,
        percent: i32,
        description: String,
    },
    ClearTzarakkMountStatus,
    MergeRiftwalkerBattleLabel(String),
    MergeRiftwalkerBattleHpFromListen {
        hp: i32,
        paren_inside: String,
        brackets: [Option<String>; 3],
    },
    ClearRiftwalkerEntityStatus,
    UpsertNergalMinion(NergalMinion),
    SetNergalResourceStatus(NergalResourceStatus),
    ClearNergalMinions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiftwalkerEntityStatus {
    pub label: String,
    pub hp: i32,
    pub max_hp: Option<i32>,
    pub hp_paren_raw: Option<String>,
    pub brackets: [String; 3],
}

fn default_riftwalker_entity_brackets() -> [String; 3] {
    ["[]".to_string(), "[]".to_string(), "[]".to_string()]
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NergalMinion {
    pub name: String,
    pub hp: i32,
    pub max_hp: i32,
    pub sp: i32,
    pub max_sp: i32,
    pub ep: i32,
    pub max_ep: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct NergalResourceStatus {
    pub vitae: i32,
    pub max_vitae: i32,
    pub potentia: i32,
    pub max_potentia: i32,
    pub evolution_points: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct SoulCompanionStatus {
    percent: i32,
    description: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TzarakkMountStatus {
    name: String,
    percent: i32,
    description: String,
}

impl SecondaryStatus {
    pub fn apply_effect(&mut self, effect: SecondaryStatusEffect) {
        match effect {
            SecondaryStatusEffect::SetSoulCompanion {
                percent,
                description,
            } => self.set_soul_companion(percent, description),
            SecondaryStatusEffect::SetTzarakkMountStatus {
                name,
                percent,
                description,
            } => self.set_tzarakk_mount_status(name, percent, description),
            SecondaryStatusEffect::ClearTzarakkMountStatus => self.clear_tzarakk_mount_status(),
            SecondaryStatusEffect::MergeRiftwalkerBattleLabel(label) => {
                self.merge_riftwalker_battle_label(label);
            }
            SecondaryStatusEffect::MergeRiftwalkerBattleHpFromListen {
                hp,
                paren_inside,
                brackets,
            } => self.merge_riftwalker_battle_hp_from_listen(
                hp,
                &paren_inside,
                brackets[0].as_deref(),
                brackets[1].as_deref(),
                brackets[2].as_deref(),
            ),
            SecondaryStatusEffect::ClearRiftwalkerEntityStatus => {
                self.clear_riftwalker_entity_status();
            }
            SecondaryStatusEffect::UpsertNergalMinion(minion) => self.upsert_nergal_minion(minion),
            SecondaryStatusEffect::SetNergalResourceStatus(status) => {
                self.set_nergal_resource_status(status);
            }
            SecondaryStatusEffect::ClearNergalMinions => self.clear_nergal_minions(),
        }
    }

    pub fn sync_guild_selection(&mut self, selection: &GuildSelection) {
        if !selection.is_selected(GuildKey::Animist) {
            self.soul_companion = None;
        }
        if !selection.is_selected(GuildKey::Riftwalker) {
            self.riftwalker_entity = None;
        }
        if !selection.is_selected(GuildKey::Tzarakk) {
            self.tzarakk_mount = None;
        }
        if !selection.is_selected(GuildKey::Nergal) {
            self.nergal_minions = [None, None, None];
            self.nergal_resource_status = None;
        }
    }

    pub fn render_lines(&self, width: u16, selection: &GuildSelection) -> Vec<Line<'static>> {
        let mut lines = Vec::new();
        if selection.is_selected(GuildKey::Animist) {
            lines.push(self.render_soul_inline());
        }
        if selection.is_selected(GuildKey::Riftwalker) {
            lines.push(self.render_riftwalker_entity_inline());
        }
        if selection.is_selected(GuildKey::Tzarakk) {
            lines.push(self.render_tzarakk_mount_inline());
        }
        if selection.is_selected(GuildKey::Nergal) {
            lines.extend(self.render_nergal_status_lines(width));
        }
        lines
    }

    pub fn set_soul_companion(&mut self, percent: i32, description: String) {
        self.soul_companion = Some(SoulCompanionStatus {
            percent,
            description,
        });
    }

    #[cfg(test)]
    pub fn has_soul_companion_status(&self) -> bool {
        self.soul_companion.is_some()
    }

    pub fn set_tzarakk_mount_status(&mut self, name: String, percent: i32, description: String) {
        self.tzarakk_mount = Some(TzarakkMountStatus {
            name,
            percent,
            description,
        });
    }

    pub fn clear_tzarakk_mount_status(&mut self) {
        self.tzarakk_mount = None;
    }

    #[cfg(test)]
    pub fn has_tzarakk_mount_status(&self) -> bool {
        self.tzarakk_mount.is_some()
    }

    pub fn merge_riftwalker_battle_label(&mut self, label: String) {
        let label = label.trim().to_string();
        self.riftwalker_entity = Some(match self.riftwalker_entity.take() {
            Some(mut s) => {
                s.label = label;
                s
            }
            None => RiftwalkerEntityStatus {
                label,
                hp: 0,
                max_hp: None,
                hp_paren_raw: None,
                brackets: default_riftwalker_entity_brackets(),
            },
        });
    }

    #[cfg(test)]
    pub fn merge_riftwalker_battle_hp(&mut self, hp: i32) {
        self.riftwalker_entity = Some(match self.riftwalker_entity.take() {
            Some(mut s) => {
                s.hp = hp;
                s
            }
            None => RiftwalkerEntityStatus {
                label: String::new(),
                hp,
                max_hp: None,
                hp_paren_raw: None,
                brackets: default_riftwalker_entity_brackets(),
            },
        });
    }

    pub fn merge_riftwalker_battle_hp_from_listen(
        &mut self,
        hp: i32,
        paren_inside: &str,
        bracket1: Option<&str>,
        bracket2: Option<&str>,
        bracket3: Option<&str>,
    ) {
        let (max_hp, hp_paren_raw) = if paren_inside.is_empty() {
            (None, None)
        } else {
            match paren_inside.parse::<i32>() {
                Ok(m) => (Some(m), None),
                Err(_) => (None, Some(paren_inside.to_string())),
            }
        };
        let norm_br = |o: Option<&str>| {
            o.map(str::trim)
                .filter(|s| !s.is_empty())
                .map(str::to_string)
                .unwrap_or_else(|| "[]".to_string())
        };
        let brackets = [norm_br(bracket1), norm_br(bracket2), norm_br(bracket3)];
        self.riftwalker_entity = Some(match self.riftwalker_entity.take() {
            Some(mut s) => {
                s.hp = hp;
                s.max_hp = max_hp;
                s.hp_paren_raw = hp_paren_raw;
                s.brackets = brackets;
                s
            }
            None => RiftwalkerEntityStatus {
                label: String::new(),
                hp,
                max_hp,
                hp_paren_raw,
                brackets,
            },
        });
    }

    pub fn clear_riftwalker_entity_status(&mut self) {
        self.riftwalker_entity = None;
    }

    #[cfg(test)]
    pub fn has_riftwalker_entity_status(&self) -> bool {
        self.riftwalker_entity.is_some()
    }

    pub fn set_nergal_resource_status(&mut self, status: NergalResourceStatus) {
        self.nergal_resource_status = Some(status);
    }

    #[cfg(test)]
    pub fn has_nergal_resource_status(&self) -> bool {
        self.nergal_resource_status.is_some()
    }

    pub fn upsert_nergal_minion(&mut self, minion: NergalMinion) {
        let name = minion.name.clone();
        for slot in &mut self.nergal_minions {
            if let Some(existing) = slot
                && existing.name == name
            {
                *slot = Some(minion);
                return;
            }
        }
        for slot in &mut self.nergal_minions {
            if slot.is_none() {
                *slot = Some(minion);
                return;
            }
        }
    }

    pub fn clear_nergal_minions(&mut self) {
        self.nergal_minions = [None, None, None];
    }

    #[cfg(test)]
    pub fn has_nergal_minions(&self) -> bool {
        self.nergal_minions.iter().any(|slot| slot.is_some())
    }

    #[cfg(test)]
    pub(crate) fn nergal_minions(&self) -> &[Option<NergalMinion>; 3] {
        &self.nergal_minions
    }

    pub fn render_riftwalker_entity_inline(&self) -> Line<'static> {
        let Some(entity) = &self.riftwalker_entity else {
            return Line::from("");
        };

        let label_fg = riftwalker_entity_label_fg(&entity.label);
        let mut spans = vec![];
        if !entity.label.is_empty() {
            spans.push(Span::styled(
                format!("{}  ", entity.label),
                Style::default().fg(label_fg),
            ));
        }
        spans.push(Span::styled("HP:", Style::default().fg(palette::TEXT)));
        let cur_fg = riftwalker_entity_current_hp_fg(entity.hp, entity.max_hp);
        spans.push(Span::styled(
            entity.hp.to_string(),
            Style::default().fg(cur_fg),
        ));

        if let Some(m) = entity.max_hp {
            spans.push(Span::styled("(", Style::default().fg(palette::TEXT)));
            spans.push(Span::styled(m.to_string(), bold_white_style()));
            spans.push(Span::styled(")", Style::default().fg(palette::TEXT)));
        } else if let Some(r) = entity.hp_paren_raw.as_ref() {
            spans.push(Span::styled("(", Style::default().fg(palette::TEXT)));
            spans.push(Span::styled(r.clone(), bold_white_style()));
            spans.push(Span::styled(")", Style::default().fg(palette::TEXT)));
        }

        for (idx, b) in entity.brackets.iter().enumerate() {
            spans.push(Span::raw(" "));
            match idx {
                0 => push_riftwalker_hp_diff_bracket_spans(&mut spans, b),
                1 => push_riftwalker_middle_bracket_spans(&mut spans, b),
                _ => push_riftwalker_trailing_bracket_spans(&mut spans, b),
            }
        }

        Line::from(spans)
    }

    pub fn render_nergal_minion_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut pieces: Vec<Vec<Span<'static>>> = Vec::new();
        for slot in &self.nergal_minions {
            let Some(minion) = slot else {
                continue;
            };
            pieces.push(minion_stat_spans(minion));
        }

        if pieces.is_empty() {
            return Vec::new();
        }

        if width == 0 {
            return pieces.into_iter().map(Line::from).collect();
        }

        let effective_width = width as usize;

        let piece_widths: Vec<usize> = pieces
            .iter()
            .map(|spans| spans_display_width(spans))
            .collect();

        let mut lines: Vec<Vec<Span<'static>>> = Vec::new();
        let mut current_row: Vec<Span<'static>> = Vec::new();
        let mut current_width: usize = 0;

        for (idx, spans) in pieces.into_iter().enumerate() {
            let piece_w = piece_widths[idx];
            let gap = if current_row.is_empty() { 0 } else { 2 };
            if !current_row.is_empty() && current_width + gap + piece_w > effective_width {
                lines.push(std::mem::take(&mut current_row));
                current_width = 0;
            }
            if !current_row.is_empty() {
                current_row.push(Span::raw("  "));
                current_width += 2;
            }
            current_width += piece_w;
            current_row.extend(spans);
        }

        if !current_row.is_empty() {
            lines.push(current_row);
        }

        lines.into_iter().map(Line::from).collect()
    }

    pub fn render_nergal_status_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut lines = self.render_nergal_minion_lines(width);
        if let Some(status) = &self.nergal_resource_status {
            lines.push(Line::from(nergal_resource_status_spans(status)));
        }
        lines
    }

    pub fn render_soul_inline(&self) -> Line<'static> {
        let Some(soul_companion) = &self.soul_companion else {
            return Line::from("");
        };

        let mut spans = vec![
            Span::styled("Soul: ", bold_white_style()),
            Span::styled(
                format!("{}%", soul_companion.percent),
                Style::default().fg(progress_color(soul_companion.percent as f32 / 100.0)),
            ),
            Span::raw(" "),
        ];
        spans.extend(soul_description_spans(&soul_companion.description));

        Line::from(spans)
    }

    pub fn render_tzarakk_mount_inline(&self) -> Line<'static> {
        let Some(mount) = &self.tzarakk_mount else {
            return Line::from("");
        };

        Line::from(vec![
            Span::styled(mount.name.clone(), bold_white_style()),
            Span::raw(": "),
            Span::styled(
                format!("{}%", mount.percent),
                Style::default().fg(progress_color(mount.percent as f32 / 100.0)),
            ),
        ])
    }
}

fn minion_stat_spans(minion: &NergalMinion) -> Vec<Span<'static>> {
    let mut out = vec![Span::raw(format!("{}: ", minion.name))];
    out.extend(inline_stat_spans("Hp", minion.hp, minion.max_hp, 0));
    out.push(Span::raw(" "));
    out.extend(inline_stat_spans("Sp", minion.sp, minion.max_sp, 0));
    out.push(Span::raw(" "));
    out.extend(inline_stat_spans("Ep", minion.ep, minion.max_ep, 0));
    out
}

fn nergal_resource_status_spans(status: &NergalResourceStatus) -> Vec<Span<'static>> {
    let mut out = Vec::new();
    out.extend(nergal_resource_stat_spans(
        "Vitae",
        status.vitae,
        status.max_vitae,
    ));
    out.push(Span::raw(" "));
    out.extend(nergal_resource_stat_spans(
        "Potentia",
        status.potentia,
        status.max_potentia,
    ));
    out.push(Span::raw(", "));
    out.push(Span::styled("Evolution points: ", bold_white_style()));
    out.push(Span::styled(
        status.evolution_points.to_string(),
        bold_white_style(),
    ));
    out
}

fn inline_stat_spans(label: &str, value: i32, max_value: i32, diff: i32) -> Vec<Span<'static>> {
    let diff_value = if diff == 0 {
        None
    } else {
        Some(format!("{diff:+}"))
    };
    let progress = if value == 0 || max_value == 0 {
        0.0
    } else {
        value as f32 / max_value as f32
    };
    let mut spans = vec![
        Span::raw(format!("{label}: ")),
        Span::styled(
            value.to_string(),
            Style::default().fg(progress_color(progress)),
        ),
        Span::raw("/"),
        Span::styled(max_value.to_string(), bold_white_style()),
    ];

    if let Some(diff_text) = diff_value {
        let diff_color = if diff > 0 {
            palette::GREEN
        } else {
            palette::RED
        };
        spans.push(Span::raw(" "));
        spans.push(Span::raw("("));
        spans.push(Span::styled(diff_text, Style::default().fg(diff_color)));
        spans.push(Span::raw(")"));
    }

    spans
}

fn soul_description_spans(description: &str) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    let mut plain = String::new();
    let mut chars = description.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '+' || ch == '-' {
            if !plain.is_empty() {
                spans.push(Span::raw(std::mem::take(&mut plain)));
            }
            let color = if ch == '+' {
                palette::GREEN
            } else {
                palette::RED
            };
            let mut run = ch.to_string();
            while chars.peek() == Some(&ch) {
                if let Some(c) = chars.next() {
                    run.push(c);
                }
            }
            spans.push(Span::styled(run, Style::default().fg(color)));
        } else {
            plain.push(ch);
        }
    }

    if !plain.is_empty() {
        spans.push(Span::raw(plain));
    }

    spans
}

fn spans_display_width(spans: &[Span<'_>]) -> usize {
    spans.iter().map(|span| span.content.width()).sum()
}

fn nergal_resource_stat_spans(label: &str, value: i32, max_value: i32) -> Vec<Span<'static>> {
    let progress = if value == 0 || max_value == 0 {
        0.0
    } else {
        value as f32 / max_value as f32
    };
    vec![
        Span::styled(format!("{label}: "), bold_white_style()),
        Span::styled(
            value.to_string(),
            Style::default().fg(progress_color(progress)),
        ),
        Span::raw("/"),
        Span::styled(max_value.to_string(), bold_white_style()),
    ]
}

fn riftwalker_bracket_delim_style() -> Style {
    Style::default().fg(palette::WHITE)
}

fn push_riftwalker_hp_diff_bracket_spans(spans: &mut Vec<Span<'static>>, bracket: &str) {
    let delim = riftwalker_bracket_delim_style();
    if let Some(inner) = bracket.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        let inner = inner.trim();
        spans.push(Span::styled("[", delim));
        if !inner.is_empty() {
            spans.push(Span::styled(
                inner.to_string(),
                riftwalker_entity_hp_diff_inner_style(inner),
            ));
        }
        spans.push(Span::styled("]", delim));
    } else {
        spans.push(Span::styled(
            bracket.to_string(),
            Style::default().fg(palette::TEXT),
        ));
    }
}

fn push_riftwalker_trailing_bracket_spans(spans: &mut Vec<Span<'static>>, bracket: &str) {
    let delim = riftwalker_bracket_delim_style();
    let inner_style = Style::default().fg(palette::TEXT);
    if let Some(inner) = bracket.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        spans.push(Span::styled("[", delim));
        if !inner.is_empty() {
            spans.push(Span::styled(inner.to_string(), inner_style));
        }
        spans.push(Span::styled("]", delim));
    } else {
        spans.push(Span::styled(bracket.to_string(), inner_style));
    }
}

fn push_riftwalker_middle_bracket_spans(spans: &mut Vec<Span<'static>>, bracket: &str) {
    let delim = riftwalker_bracket_delim_style();
    if let Some(inner) = bracket.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
        spans.push(Span::styled("[", delim));
        if !inner.is_empty() {
            spans.push(Span::styled(inner.to_string(), bold_white_style()));
        }
        spans.push(Span::styled("]", delim));
    } else {
        spans.push(Span::styled(bracket.to_string(), bold_white_style()));
    }
}

fn riftwalker_entity_hp_diff_inner_style(inner: &str) -> Style {
    if inner.is_empty() {
        return Style::default().fg(palette::TEXT);
    }
    let Ok(n) = inner.parse::<i32>() else {
        return Style::default().fg(palette::TEXT);
    };
    let fg = if n > 0 {
        palette::GREEN
    } else if n < 0 {
        palette::RED
    } else {
        palette::TEXT
    };
    Style::default().fg(fg)
}

fn riftwalker_entity_label_fg(label: &str) -> Color {
    for word in label
        .split(|c: char| !c.is_ascii_alphanumeric())
        .filter(|s| !s.is_empty())
    {
        if word.eq_ignore_ascii_case("fire") {
            return palette::RED;
        }
        if word.eq_ignore_ascii_case("air") {
            return Color::Rgb(140, 200, 255);
        }
        if word.eq_ignore_ascii_case("water") {
            return Color::Rgb(36, 72, 190);
        }
        if word.eq_ignore_ascii_case("earth") {
            return Color::Rgb(168, 110, 55);
        }
    }
    palette::TEXT
}

fn riftwalker_entity_current_hp_fg(hp: i32, max_hp: Option<i32>) -> Color {
    let Some(max) = max_hp.filter(|&m| m > 0) else {
        return palette::TEXT;
    };
    let progress = if hp <= 0 {
        0.0
    } else {
        (hp as f32 / max as f32).min(1.0)
    };
    progress_color(progress)
}

fn bold_white_style() -> Style {
    Style::default()
        .fg(palette::BOLD_WHITE)
        .add_modifier(Modifier::BOLD)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line_text(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn animist_selection() -> GuildSelection {
        GuildSelection::from_playable_keys([GuildKey::Animist], Some("nature"))
    }

    fn riftwalker_selection() -> GuildSelection {
        GuildSelection::from_playable_keys([GuildKey::Riftwalker], Some("magical"))
    }

    fn tzarakk_selection() -> GuildSelection {
        GuildSelection::from_playable_keys([GuildKey::Tzarakk], Some("nomad"))
    }

    fn nergal_selection() -> GuildSelection {
        GuildSelection::from_playable_keys([GuildKey::Nergal], Some("evil_religious"))
    }

    #[test]
    fn render_lines_empty_when_no_guild_selected() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(75, "resting".to_string());
        assert!(
            status
                .render_lines(80, &GuildSelection::default())
                .is_empty()
        );
    }

    #[test]
    fn render_soul_only_when_animist_selected() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(75, "resting".to_string());

        let lines = status.render_lines(80, &animist_selection());
        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&lines[0]), "Soul: 75% resting");
    }

    #[test]
    fn sync_guild_selection_clears_deselected_guild_state() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(75, "resting".to_string());
        status.merge_riftwalker_battle_hp(100);
        status.set_tzarakk_mount_status("Vedir".into(), 90, "ok".into());
        status.set_nergal_resource_status(NergalResourceStatus {
            vitae: 1,
            max_vitae: 10,
            potentia: 2,
            max_potentia: 10,
            evolution_points: 0,
        });

        status.sync_guild_selection(&GuildSelection::default());

        assert!(!status.has_soul_companion_status());
        assert!(!status.has_riftwalker_entity_status());
        assert!(!status.has_tzarakk_mount_status());
        assert!(!status.has_nergal_resource_status());
    }

    #[test]
    fn render_soul_inline_includes_soul_companion_status_without_name() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(75, "resting".to_string());

        assert_eq!(line_text(&status.render_soul_inline()), "Soul: 75% resting");

        let line = status.render_soul_inline();
        let label = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "Soul: ")
            .expect("soul label span");
        assert!(label.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(label.style.fg, Some(palette::BOLD_WHITE));
    }

    #[test]
    fn render_soul_inline_styles_plus_green_minus_red() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(88, "- x +".to_string());
        let line = status.render_soul_inline();

        assert_eq!(line_text(&line), "Soul: 88% - x +");

        let minus = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "-")
            .expect("minus span");
        assert_eq!(minus.style.fg, Some(palette::RED));

        let plus = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+")
            .expect("plus span");
        assert_eq!(plus.style.fg, Some(palette::GREEN));
    }

    #[test]
    fn render_tzarakk_mount_inline_includes_name_and_percent() {
        let mut status = SecondaryStatus::default();
        status.set_tzarakk_mount_status("Vedir".to_string(), 100, "in excellent shape".to_string());

        assert_eq!(
            line_text(&status.render_tzarakk_mount_inline()),
            "Vedir: 100%"
        );

        let line = status.render_tzarakk_mount_inline();
        let name = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "Vedir")
            .expect("mount name span");
        assert!(name.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(name.style.fg, Some(palette::BOLD_WHITE));
    }

    #[test]
    fn render_riftwalker_entity_hp_diff_bracket_green_red() {
        let mut status = SecondaryStatus::default();
        status.merge_riftwalker_battle_hp_from_listen(
            100,
            "100",
            Some("[+7]"),
            Some("[]"),
            Some("[]"),
        );
        let line = status.render_riftwalker_entity_inline();
        let pos = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+7")
            .expect("+7 span");
        assert_eq!(pos.style.fg, Some(palette::GREEN));

        status.merge_riftwalker_battle_hp_from_listen(
            90,
            "100",
            Some("[-3]"),
            Some("[]"),
            Some("[]"),
        );
        let line = status.render_riftwalker_entity_inline();
        let neg = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "-3")
            .expect("-3 span");
        assert_eq!(neg.style.fg, Some(palette::RED));
    }

    #[test]
    fn render_riftwalker_entity_middle_bracket_inner_is_bold_white() {
        let mut status = SecondaryStatus::default();
        status.merge_riftwalker_battle_hp_from_listen(
            50,
            "100",
            Some("[+1]"),
            Some("[mid]"),
            Some("[]"),
        );
        let line = status.render_riftwalker_entity_inline();
        let mid = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "mid")
            .expect("mid span");
        assert!(
            mid.style.add_modifier.contains(Modifier::BOLD),
            "middle bracket contents should be bold"
        );
        assert_eq!(mid.style.fg, Some(palette::BOLD_WHITE));
        assert_eq!(line_text(&line), "HP:50(100) [+1] [mid] []");
    }

    #[test]
    fn render_riftwalker_entity_inline_empty_when_cleared() {
        let status = SecondaryStatus::default();
        assert_eq!(line_text(&status.render_riftwalker_entity_inline()), "");
    }

    #[test]
    fn clear_riftwalker_entity_status_removes_secondary_line() {
        let mut status = SecondaryStatus::default();
        status.merge_riftwalker_battle_hp(400);
        status.clear_riftwalker_entity_status();
        assert!(!status.has_riftwalker_entity_status());
    }

    fn sample_minion_a() -> NergalMinion {
        NergalMinion {
            name: "X".to_string(),
            hp: 1,
            max_hp: 10,
            sp: 2,
            max_sp: 20,
            ep: 3,
            max_ep: 30,
        }
    }

    fn sample_minion_b() -> NergalMinion {
        NergalMinion {
            name: "Y".to_string(),
            hp: 4,
            max_hp: 40,
            sp: 5,
            max_sp: 50,
            ep: 6,
            max_ep: 60,
        }
    }

    #[test]
    fn nergal_minion_lines_split_when_row_narrow() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(sample_minion_a());
        status.upsert_nergal_minion(sample_minion_b());

        let lines_wide = status.render_nergal_minion_lines(200);
        assert_eq!(lines_wide.len(), 1);

        let lines_narrow = status.render_nergal_minion_lines(40);
        assert_eq!(lines_narrow.len(), 2);
    }

    #[test]
    fn nergal_status_lines_render_resource_status_below_minions() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(sample_minion_a());
        status.set_nergal_resource_status(NergalResourceStatus {
            vitae: 22,
            max_vitae: 1000,
            potentia: 752,
            max_potentia: 1000,
            evolution_points: 0,
        });

        let lines = status.render_nergal_status_lines(200);

        assert_eq!(lines.len(), 2);
        assert!(line_text(&lines[0]).starts_with("X: Hp: 1/10"));
        assert_eq!(
            line_text(&lines[1]),
            "Vitae: 22/1000 Potentia: 752/1000, Evolution points: 0"
        );
    }

    #[test]
    fn render_lines_hides_soul_when_animist_not_selected_even_with_data() {
        let mut status = SecondaryStatus::default();
        status.set_soul_companion(75, "resting".to_string());
        assert!(
            status
                .render_lines(80, &GuildSelection::default())
                .is_empty()
        );
    }

    #[test]
    fn render_riftwalker_only_when_riftwalker_selected() {
        let mut status = SecondaryStatus::default();
        status.merge_riftwalker_battle_hp(100);
        let lines = status.render_lines(80, &riftwalker_selection());
        assert_eq!(lines.len(), 1);
        assert!(line_text(&lines[0]).starts_with("HP:"));
    }

    #[test]
    fn upsert_nergal_minion_no_new_slot_when_three_full_and_name_unknown() {
        let mut status = SecondaryStatus::default();
        for name in ["a", "b", "c"] {
            status.upsert_nergal_minion(NergalMinion {
                name: name.into(),
                hp: 1,
                max_hp: 1,
                sp: 1,
                max_sp: 1,
                ep: 1,
                max_ep: 1,
            });
        }
        status.upsert_nergal_minion(NergalMinion {
            name: "d".into(),
            hp: 9,
            max_hp: 9,
            sp: 9,
            max_sp: 9,
            ep: 9,
            max_ep: 9,
        });

        assert!(
            !status
                .nergal_minions()
                .iter()
                .any(|slot| slot.as_ref().is_some_and(|creature| creature.name == "d"))
        );
    }

    #[test]
    fn render_tzarakk_only_when_tzarakk_selected() {
        let mut status = SecondaryStatus::default();
        status.set_tzarakk_mount_status("Vedir".into(), 100, "ok".into());
        let lines = status.render_lines(80, &tzarakk_selection());
        assert_eq!(lines.len(), 1);
        assert_eq!(line_text(&lines[0]), "Vedir: 100%");
    }

    #[test]
    fn render_nergal_only_when_nergal_selected() {
        let mut status = SecondaryStatus::default();
        status.set_nergal_resource_status(NergalResourceStatus {
            vitae: 22,
            max_vitae: 1000,
            potentia: 752,
            max_potentia: 1000,
            evolution_points: 0,
        });
        let lines = status.render_lines(200, &nergal_selection());
        assert_eq!(lines.len(), 1);
    }
}
