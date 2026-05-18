use crate::ansi::palette;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use unicode_width::UnicodeWidthStr;

#[derive(Default, Debug, Clone)]
pub struct Stats {
    hp: i32,
    max_hp: i32,
    diff_hp: i32,
    sp: i32,
    max_sp: i32,
    diff_sp: i32,
    ep: i32,
    max_ep: i32,
    diff_ep: i32,
    exp: i32,
    diff_exp: i32,
    money: i32,
    diff_money: i32,
    combat_round_active: bool,
    soul_companion: Option<SoulCompanionStatus>,
    tzarakk_mount: Option<TzarakkMountStatus>,
    riftwalker_entity: Option<RiftwalkerEntityStatus>,
    pub(crate) nergal_minions: [Option<NergalMinion>; 3],
    /// Green `c` in the recovery bracket: on when the MUD hints you may want to camp; off while resting (lie-down line).
    recovery_bracket_camping: bool,
    /// Yellow `m`: on after meditation harmony line, off when meditation starts.
    recovery_bracket_meditation: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatsEffect {
    UpdatePrompt([i32; 7]),
    UpdateShortScore([i32; 13]),
    StartCombatRound,
    EndCombat,
    SetRecoveryBracketCamping(bool),
    SetRecoveryBracketMeditation(bool),
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
    ClearNergalMinions,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RiftwalkerEntityStatus {
    pub label: String,
    pub hp: i32,
    pub max_hp: Option<i32>,
    /// Non-numeric parenthetical from the MUD when `max_hp` is absent (e.g. percent).
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

impl Stats {
    pub fn new(stats: [i32; 7]) -> Self {
        Self {
            hp: stats[0],
            max_hp: stats[1],
            sp: stats[2],
            max_sp: stats[3],
            ep: stats[4],
            max_ep: stats[5],
            exp: stats[6],
            ..Default::default()
        }
    }

    pub fn new_from_sc(stats: [i32; 13]) -> Self {
        Self {
            hp: stats[0],
            max_hp: stats[1],
            diff_hp: stats[2],
            sp: stats[3],
            max_sp: stats[4],
            diff_sp: stats[5],
            ep: stats[6],
            max_ep: stats[7],
            diff_ep: stats[8],
            money: stats[9],
            diff_money: stats[10],
            exp: stats[11],
            diff_exp: stats[12],
            ..Default::default()
        }
    }

    pub fn apply_effect(&mut self, effect: StatsEffect) {
        match effect {
            StatsEffect::UpdatePrompt(stats) => self.update_from_prompt(stats),
            StatsEffect::UpdateShortScore(stats) => self.update_from_short_score(stats),
            StatsEffect::StartCombatRound => self.start_combat_round(),
            StatsEffect::EndCombat => self.end_combat(),
            StatsEffect::SetRecoveryBracketCamping(active) => {
                self.set_recovery_bracket_camping(active);
            }
            StatsEffect::SetRecoveryBracketMeditation(active) => {
                self.set_recovery_bracket_meditation(active);
            }
            StatsEffect::SetSoulCompanion {
                percent,
                description,
            } => self.set_soul_companion(percent, description),
            StatsEffect::SetTzarakkMountStatus {
                name,
                percent,
                description,
            } => self.set_tzarakk_mount_status(name, percent, description),
            StatsEffect::ClearTzarakkMountStatus => self.clear_tzarakk_mount_status(),
            StatsEffect::MergeRiftwalkerBattleLabel(label) => {
                self.merge_riftwalker_battle_label(label);
            }
            StatsEffect::MergeRiftwalkerBattleHpFromListen {
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
            StatsEffect::ClearRiftwalkerEntityStatus => self.clear_riftwalker_entity_status(),
            StatsEffect::UpsertNergalMinion(minion) => self.upsert_nergal_minion(minion),
            StatsEffect::ClearNergalMinions => self.clear_nergal_minions(),
        }
    }

    pub fn update_from_prompt(&mut self, stats: [i32; 7]) {
        let diff_hp = self.diff_hp;
        let diff_sp = self.diff_sp;
        let diff_ep = self.diff_ep;
        let diff_exp = self.diff_exp;
        let diff_money = self.diff_money;
        let combat_round_active = self.combat_round_active;
        let soul_companion = self.soul_companion.clone();
        let tzarakk_mount = self.tzarakk_mount.clone();
        let riftwalker_entity = self.riftwalker_entity.clone();
        let nergal_minions = self.nergal_minions.clone();
        let money = self.money;
        let recovery_bracket_camping = self.recovery_bracket_camping;
        let recovery_bracket_meditation = self.recovery_bracket_meditation;
        *self = Self::new(stats);
        self.diff_hp = diff_hp;
        self.diff_sp = diff_sp;
        self.diff_ep = diff_ep;
        self.diff_exp = diff_exp;
        self.diff_money = diff_money;
        self.combat_round_active = combat_round_active;
        self.soul_companion = soul_companion;
        self.tzarakk_mount = tzarakk_mount;
        self.riftwalker_entity = riftwalker_entity;
        self.nergal_minions = nergal_minions;
        self.money = money;
        self.recovery_bracket_camping = recovery_bracket_camping;
        self.recovery_bracket_meditation = recovery_bracket_meditation;
    }

    pub fn update_from_short_score(&mut self, stats: [i32; 13]) {
        let diff_hp = self.diff_hp;
        let diff_sp = self.diff_sp;
        let diff_ep = self.diff_ep;
        let diff_exp = self.diff_exp;
        let diff_money = self.diff_money;
        let combat_round_active = self.combat_round_active;
        let soul_companion = self.soul_companion.clone();
        let tzarakk_mount = self.tzarakk_mount.clone();
        let riftwalker_entity = self.riftwalker_entity.clone();
        let nergal_minions = self.nergal_minions.clone();
        let recovery_bracket_camping = self.recovery_bracket_camping;
        let recovery_bracket_meditation = self.recovery_bracket_meditation;
        *self = Self::new_from_sc(stats);
        if combat_round_active {
            self.diff_hp += diff_hp;
            self.diff_sp += diff_sp;
            self.diff_ep += diff_ep;
            self.diff_exp += diff_exp;
            self.diff_money += diff_money;
        }
        self.combat_round_active = combat_round_active;
        self.soul_companion = soul_companion;
        self.tzarakk_mount = tzarakk_mount;
        self.riftwalker_entity = riftwalker_entity;
        self.nergal_minions = nergal_minions;
        self.recovery_bracket_camping = recovery_bracket_camping;
        self.recovery_bracket_meditation = recovery_bracket_meditation;
    }

    pub(crate) fn start_combat_round(&mut self) {
        self.clear_diffs();
        self.combat_round_active = true;
    }

    pub(crate) fn end_combat(&mut self) {
        self.combat_round_active = false;
    }

    fn clear_diffs(&mut self) {
        self.diff_hp = 0;
        self.diff_sp = 0;
        self.diff_ep = 0;
        self.diff_exp = 0;
        self.diff_money = 0;
    }

    pub fn set_recovery_bracket_defaults_for_login(&mut self) {
        self.recovery_bracket_camping = true;
        self.recovery_bracket_meditation = true;
    }

    pub(crate) fn set_recovery_bracket_camping(&mut self, active: bool) {
        self.recovery_bracket_camping = active;
    }

    pub(crate) fn set_recovery_bracket_meditation(&mut self, active: bool) {
        self.recovery_bracket_meditation = active;
    }

    pub fn set_soul_companion(&mut self, percent: i32, description: String) {
        self.soul_companion = Some(SoulCompanionStatus {
            percent,
            description,
        });
    }

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

    /// HP-only merge (preserves other battle fields). Battle listen uses [`merge_riftwalker_battle_hp_from_listen`].
    #[allow(dead_code)]
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

    /// Parsed battle-listen HP line: `HP:curr(max_or_other) […] […] […]`.
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

    pub fn has_riftwalker_entity_status(&self) -> bool {
        self.riftwalker_entity.is_some()
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

    pub fn has_nergal_minions(&self) -> bool {
        self.nergal_minions.iter().any(|slot| slot.is_some())
    }

    /// Slot update rules mirroring legacy `save_minion_stats`: update slot with same name, else first empty;
    /// no-op if all three occupied and name is new.
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

    /// Pack minion status into full terminal rows; each minion stays on one line (may wrap to next row).
    pub fn render_nergal_minion_lines(&self, width: u16) -> Vec<Line<'static>> {
        let mut pieces: Vec<Vec<Span<'static>>> = Vec::new();
        for slot in &self.nergal_minions {
            let Some(minion) = slot else {
                continue;
            };
            pieces.push(self.minion_stat_spans(minion));
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

    fn minion_stat_spans(&self, minion: &NergalMinion) -> Vec<Span<'static>> {
        let mut out = vec![Span::raw(format!("{}: ", minion.name))];
        out.extend(self.inline_stat_spans("Hp", minion.hp, minion.max_hp, 0));
        out.push(Span::raw(" "));
        out.extend(self.inline_stat_spans("Sp", minion.sp, minion.max_sp, 0));
        out.push(Span::raw(" "));
        out.extend(self.inline_stat_spans("Ep", minion.ep, minion.max_ep, 0));
        out
    }

    pub fn render_inline(&self) -> Line<'static> {
        let mut spans = Vec::new();
        spans.extend(self.inline_stat_spans("Hp", self.hp, self.max_hp, self.diff_hp));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_stat_spans("Sp", self.sp, self.max_sp, self.diff_sp));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_stat_spans("Ep", self.ep, self.max_ep, self.diff_ep));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_value_spans("$", self.money, self.diff_money));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_value_spans("Exp", self.exp, self.diff_exp));
        spans.push(Span::raw("  ["));
        if self.recovery_bracket_camping {
            spans.push(Span::styled("c", Style::default().fg(palette::GREEN)));
        }
        if self.recovery_bracket_meditation {
            spans.push(Span::styled("m", Style::default().fg(palette::YELLOW)));
        }
        spans.push(Span::raw("]"));

        Line::from(spans)
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

    fn inline_stat_spans(
        &self,
        label: &str,
        value: i32,
        max_value: i32,
        diff: i32,
    ) -> Vec<Span<'static>> {
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

    fn inline_value_spans(&self, label: &str, value: i32, diff: i32) -> Vec<Span<'static>> {
        let mut spans = vec![
            Span::raw(format!("{label}: ")),
            Span::styled(value.to_string(), bold_white_style()),
        ];
        if diff != 0 {
            let diff_color = if diff > 0 {
                palette::GREEN
            } else {
                palette::RED
            };
            spans.push(Span::raw(" "));
            spans.push(Span::raw("("));
            spans.push(Span::styled(
                format!("{diff:+}"),
                Style::default().fg(diff_color),
            ));
            spans.push(Span::raw(")"));
        }
        spans
    }
}

/// Soul companion trailing text (e.g. `+` / `-` trend markers): color runs of `+` green and `-` red.
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

fn riftwalker_bracket_delim_style() -> Style {
    Style::default().fg(palette::WHITE)
}

/// HP delta bracket: `[` / `]` normal white; inner green / red / default by sign.
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

/// Third battle-listen bracket: `[` / `]` normal white; inner default text color.
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

/// Second battle-listen bracket: `[` / `]` normal white; inner bold white (player max style).
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

/// Inner text of first battle-listen bracket (HP delta): green / red / default by sign.
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

/// Element token in the battle-listen label (`Fire entity`, custom text containing `water`, …).
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

/// Same progression as player HP in [`Stats::inline_stat_spans`] when max is known.
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

fn progress_color(value: f32) -> Color {
    if value < 0.1 {
        Color::Rgb(139, 0, 0)
    } else if value < 0.2 {
        Color::Rgb(255, 48, 48)
    } else if value < 0.3 {
        Color::Rgb(255, 79, 109)
    } else if value < 0.4 {
        Color::Rgb(255, 79, 216)
    } else if value < 0.5 {
        Color::Rgb(214, 92, 255)
    } else if value < 0.6 {
        Color::Rgb(255, 176, 0)
    } else if value < 0.7 {
        Color::Rgb(255, 215, 0)
    } else if value < 0.8 {
        Color::Rgb(255, 241, 118)
    } else if value < 0.9 {
        Color::Rgb(184, 243, 90)
    } else {
        Color::Rgb(48, 209, 88)
    }
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

    #[test]
    fn render_inline_does_not_include_soul_companion_status() {
        let mut stats = Stats::default();
        stats.set_soul_companion(75, "resting".to_string());

        assert!(!line_text(&stats.render_inline()).contains("Soul:"));
    }

    #[test]
    fn render_inline_does_not_include_tzarakk_mount_status() {
        let mut stats = Stats::default();
        stats.set_tzarakk_mount_status("Vedir".to_string(), 100, "in excellent shape".to_string());

        assert!(!line_text(&stats.render_inline()).contains("Vedir:"));
    }

    #[test]
    fn render_inline_exp_and_money_values_are_bold_white_with_colored_diffs() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 100, 0, 2, 200, 0, 3, 300, 0, 88_888, 15, 77_777, -20]);
        let line = stats.render_inline();

        let exp_val = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "77777")
            .expect("exp value span");
        assert!(
            exp_val.style.add_modifier.contains(Modifier::BOLD),
            "exp value should be bold"
        );
        assert_eq!(exp_val.style.fg, Some(palette::BOLD_WHITE));

        let exp_diff = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "-20")
            .expect("exp diff span");
        assert_eq!(exp_diff.style.fg, Some(palette::RED));

        let money_val = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "88888")
            .expect("money value span");
        assert!(
            money_val.style.add_modifier.contains(Modifier::BOLD),
            "money value should be bold"
        );
        assert_eq!(money_val.style.fg, Some(palette::BOLD_WHITE));

        let hp_max = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "100")
            .expect("hp max span");
        assert!(hp_max.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(hp_max.style.fg, Some(palette::BOLD_WHITE));

        let money_diff = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+15")
            .expect("money diff span");
        assert_eq!(money_diff.style.fg, Some(palette::GREEN));
    }

    #[test]
    fn render_inline_recovery_bracket_suffix() {
        let mut stats = Stats::default();
        stats.update_from_short_score([905, 905, 0, 143, 143, 0, 474, 474, 0, 0, 0, 87_333, 0]);
        assert!(line_text(&stats.render_inline()).ends_with("  []"));

        stats.set_recovery_bracket_camping(true);
        stats.set_recovery_bracket_meditation(false);
        assert!(line_text(&stats.render_inline()).ends_with("  [c]"));

        stats.set_recovery_bracket_meditation(true);
        let line = line_text(&stats.render_inline());
        assert!(line.ends_with("  [cm]"), "got {line:?}");

        let rendered = stats.render_inline();
        let camping = rendered
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "c")
            .expect("c span");
        assert_eq!(camping.style.fg, Some(palette::GREEN));
        let meditation = rendered
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "m")
            .expect("m span");
        assert_eq!(meditation.style.fg, Some(palette::YELLOW));
    }

    #[test]
    fn set_recovery_bracket_defaults_for_login_enables_both() {
        let mut stats = Stats::default();
        stats.set_recovery_bracket_camping(false);
        stats.set_recovery_bracket_meditation(false);
        stats.set_recovery_bracket_defaults_for_login();
        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);
        assert!(line_text(&stats.render_inline()).ends_with("  [cm]"));
    }

    #[test]
    fn prompt_updates_preserve_recovery_bracket_flags() {
        let mut stats = Stats::default();
        stats.set_recovery_bracket_camping(false);
        stats.set_recovery_bracket_meditation(true);
        stats.update_from_prompt([10, 20, 30, 40, 50, 60, 70]);
        let line = line_text(&stats.render_inline());
        assert!(line.ends_with("  [m]"), "got {line:?}");
    }

    #[test]
    fn short_score_updates_preserve_recovery_bracket_flags() {
        let mut stats = Stats::default();
        stats.set_recovery_bracket_camping(true);
        stats.set_recovery_bracket_meditation(false);
        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);
        assert!(line_text(&stats.render_inline()).ends_with("  [c]"));
    }

    #[test]
    fn render_soul_inline_includes_soul_companion_status_without_name() {
        let mut stats = Stats::default();
        stats.set_soul_companion(75, "resting".to_string());

        assert_eq!(line_text(&stats.render_soul_inline()), "Soul: 75% resting");

        let line = stats.render_soul_inline();
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
        let mut stats = Stats::default();
        stats.set_soul_companion(88, "- x +".to_string());
        let line = stats.render_soul_inline();

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
    fn prompt_updates_preserve_soul_companion_status() {
        let mut stats = Stats::default();
        stats.set_soul_companion(90, "following".to_string());

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            line_text(&stats.render_soul_inline()),
            "Soul: 90% following"
        );
    }

    #[test]
    fn prompt_updates_preserve_money_from_short_score() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 2786, 10, 21657, 0]);

        stats.update_from_prompt([10, 20, 30, 40, 50, 60, 70]);

        let line = line_text(&stats.render_inline());
        assert!(
            line.contains("$: 2786"),
            "expected money from short score after prompt; got {line:?}"
        );
        assert!(
            !line.contains("$: 0"),
            "prompt must not reset money to zero; got {line:?}"
        );
    }

    #[test]
    fn prompt_updates_preserve_money_with_zero_diff() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 500, 0, 8, 0]);

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        let line = line_text(&stats.render_inline());
        assert!(
            line.contains("$: 500"),
            "expected money from short score after prompt; got {line:?}"
        );
        assert!(
            !line.contains("$: 500 (+"),
            "diff should stay cleared when zero"
        );
    }

    #[test]
    fn prompt_updates_preserve_visible_diffs_from_short_score() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);

        stats.update_from_prompt([10, 20, 30, 40, 50, 60, 70]);

        let line = line_text(&stats.render_inline());
        assert!(line.contains("$: 100"));
        assert!(line.contains("+10"), "hp diff should persist; got {line:?}");
        assert!(
            line.contains("-5"),
            "sp/money diff should persist; got {line:?}"
        );
        assert!(
            line.contains("+9"),
            "exp diff should persist after prompt-only line; got {line:?}"
        );
    }

    #[test]
    fn zero_diff_short_score_during_combat_round_preserves_accumulated_diffs() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);

        let line = line_text(&stats.render_inline());
        assert!(line.contains("+10"), "hp diff should persist; got {line:?}");
        assert!(
            line.contains("-5"),
            "sp/money diff should persist; got {line:?}"
        );
        assert!(
            line.contains("+9"),
            "exp diff should persist after zero-diff short score; got {line:?}"
        );
    }

    #[test]
    fn zero_diff_short_score_outside_combat_replaces_previous_diffs() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);

        let line = line_text(&stats.render_inline());
        assert!(
            !line.contains("+10") && !line.contains("-5") && !line.contains("+9"),
            "zero-diff short score outside combat should clear previous diffs; got {line:?}"
        );
    }

    #[test]
    fn combat_round_short_score_diffs_accumulate() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);
        stats.update_from_short_score([1, 2, -2, 3, 4, 0, 5, 6, -3, 100, 4, 8, 1]);

        let line = line_text(&stats.render_inline());
        assert!(
            line.contains("+8"),
            "hp diffs should accumulate; got {line:?}"
        );
        assert!(line.contains("-5"), "sp diff should persist; got {line:?}");
        assert!(
            line.contains("$: 100 (-1)"),
            "money diff should accumulate; got {line:?}"
        );
        assert!(
            line.contains("+10"),
            "exp diffs should accumulate; got {line:?}"
        );
        assert!(
            line.contains("Ep: 5/6 (+4)"),
            "ep diff should accumulate from later short score; got {line:?}"
        );
    }

    #[test]
    fn starting_new_combat_round_clears_previous_diffs() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);

        stats.start_combat_round();

        let line = line_text(&stats.render_inline());
        assert!(
            !line.contains("+10") && !line.contains("-5") && !line.contains("+9"),
            "new combat round should clear previous diffs; got {line:?}"
        );
    }

    #[test]
    fn ending_combat_preserves_final_round_diffs() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);

        stats.end_combat();

        let line = line_text(&stats.render_inline());
        assert!(line.contains("+10"), "hp diff should remain; got {line:?}");
        assert!(
            line.contains("-5"),
            "sp/money diff should remain; got {line:?}"
        );
        assert!(line.contains("+9"), "exp diff should remain; got {line:?}");
    }

    #[test]
    fn short_score_after_combat_end_replaces_final_round_diffs() {
        let mut stats = Stats::default();
        stats.start_combat_round();
        stats.update_from_short_score([1, 2, 10, 3, 4, -5, 5, 6, 7, 100, -5, 8, 9]);
        stats.end_combat();

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, 0, 8, 0]);

        let line = line_text(&stats.render_inline());
        assert!(
            !line.contains("+10") && !line.contains("-5") && !line.contains("+9"),
            "post-combat short score should replace final round diffs; got {line:?}"
        );
    }

    #[test]
    fn short_score_updates_preserve_soul_companion_status() {
        let mut stats = Stats::default();
        stats.set_soul_companion(45, "nearby".to_string());

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);

        assert_eq!(line_text(&stats.render_soul_inline()), "Soul: 45% nearby");
    }

    #[test]
    fn render_tzarakk_mount_inline_includes_name_and_percent() {
        let mut stats = Stats::default();
        stats.set_tzarakk_mount_status("Vedir".to_string(), 100, "in excellent shape".to_string());

        assert_eq!(
            line_text(&stats.render_tzarakk_mount_inline()),
            "Vedir: 100%"
        );

        let line = stats.render_tzarakk_mount_inline();
        let name = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "Vedir")
            .expect("mount name span");
        assert!(name.style.add_modifier.contains(Modifier::BOLD));
        assert_eq!(name.style.fg, Some(palette::BOLD_WHITE));
    }

    #[test]
    fn prompt_updates_preserve_tzarakk_mount_status() {
        let mut stats = Stats::default();
        stats.set_tzarakk_mount_status("Vedir".to_string(), 90, "in good shape".to_string());

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            line_text(&stats.render_tzarakk_mount_inline()),
            "Vedir: 90%"
        );
    }

    #[test]
    fn short_score_updates_preserve_tzarakk_mount_status() {
        let mut stats = Stats::default();
        stats.set_tzarakk_mount_status("Orthos".to_string(), 45, "hurt".to_string());

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);

        assert_eq!(
            line_text(&stats.render_tzarakk_mount_inline()),
            "Orthos: 45%"
        );
    }

    #[test]
    fn prompt_updates_preserve_riftwalker_entity_status() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_label("glowing fire entity".to_string());
        stats.merge_riftwalker_battle_hp(300);

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        assert_eq!(
            line_text(&stats.render_riftwalker_entity_inline()),
            "glowing fire entity  HP:300 [] [] []"
        );
    }

    #[test]
    fn short_score_updates_preserve_riftwalker_entity_status() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_hp(180);

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);

        assert_eq!(
            line_text(&stats.render_riftwalker_entity_inline()),
            "HP:180 [] [] []"
        );
    }

    #[test]
    fn render_riftwalker_entity_hp_diff_bracket_green_red() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_hp_from_listen(
            100,
            "100",
            Some("[+7]"),
            Some("[]"),
            Some("[]"),
        );
        let line = stats.render_riftwalker_entity_inline();
        let pos = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+7")
            .expect("+7 span");
        assert_eq!(pos.style.fg, Some(palette::GREEN));

        stats.merge_riftwalker_battle_hp_from_listen(
            90,
            "100",
            Some("[-3]"),
            Some("[]"),
            Some("[]"),
        );
        let line = stats.render_riftwalker_entity_inline();
        let neg = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "-3")
            .expect("-3 span");
        assert_eq!(neg.style.fg, Some(palette::RED));

        stats.merge_riftwalker_battle_hp_from_listen(
            50,
            "100",
            Some("[0]"),
            Some("[]"),
            Some("[]"),
        );
        let line = stats.render_riftwalker_entity_inline();
        let zero = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "0")
            .expect("0 span");
        assert_eq!(zero.style.fg, Some(palette::TEXT));

        stats.merge_riftwalker_battle_hp_from_listen(
            50,
            "100",
            Some("[x]"),
            Some("[]"),
            Some("[]"),
        );
        let line = stats.render_riftwalker_entity_inline();
        let other = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "x")
            .expect("x span");
        assert_eq!(other.style.fg, Some(palette::TEXT));
    }

    #[test]
    fn render_riftwalker_entity_middle_bracket_inner_is_bold_white() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_hp_from_listen(
            50,
            "100",
            Some("[+1]"),
            Some("[mid]"),
            Some("[]"),
        );
        let line = stats.render_riftwalker_entity_inline();
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
        let stats = Stats::default();
        assert_eq!(line_text(&stats.render_riftwalker_entity_inline()), "");
    }

    #[test]
    fn clear_riftwalker_entity_status_removes_secondary_line() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_hp(400);
        stats.clear_riftwalker_entity_status();
        assert!(!stats.has_riftwalker_entity_status());
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
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(sample_minion_a());
        stats.upsert_nergal_minion(sample_minion_b());

        let lines_wide = stats.render_nergal_minion_lines(200);
        assert_eq!(
            lines_wide.len(),
            1,
            "wide terminal should keep both minions on one row"
        );

        let lines_narrow = stats.render_nergal_minion_lines(40);
        assert_eq!(
            lines_narrow.len(),
            2,
            "narrow terminal should move the second minion to the next row"
        );
    }

    #[test]
    fn nergal_minion_lines_one_per_row_when_width_zero() {
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(sample_minion_a());
        stats.upsert_nergal_minion(sample_minion_b());

        let lines = stats.render_nergal_minion_lines(0);
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn prompt_updates_preserve_nergal_minions() {
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(sample_minion_a());

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        assert!(stats.has_nergal_minions());
        assert_eq!(
            stats.nergal_minions[0].as_ref().map(|m| m.name.as_str()),
            Some("X")
        );
    }

    #[test]
    fn short_score_updates_preserve_nergal_minions() {
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(sample_minion_b());

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);

        assert_eq!(
            stats.nergal_minions[0].as_ref().map(|m| m.name.as_str()),
            Some("Y")
        );
    }

    #[test]
    fn upsert_nergal_minion_no_new_slot_when_three_full_and_name_unknown() {
        let mut stats = Stats::default();
        stats.upsert_nergal_minion(NergalMinion {
            name: "a".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        stats.upsert_nergal_minion(NergalMinion {
            name: "b".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        stats.upsert_nergal_minion(NergalMinion {
            name: "c".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        stats.upsert_nergal_minion(NergalMinion {
            name: "d".into(),
            hp: 9,
            max_hp: 9,
            sp: 9,
            max_sp: 9,
            ep: 9,
            max_ep: 9,
        });

        assert!(
            !stats
                .nergal_minions
                .iter()
                .any(|slot| { slot.as_ref().is_some_and(|creature| creature.name == "d") })
        );
    }
}
