use crate::ansi::palette;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};

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
    /// Green `c` in the recovery bracket: on when the MUD hints you may want to camp; off while resting (lie-down line).
    recovery_bracket_camping: bool,
    /// Yellow `m`: on after meditation harmony line, off when meditation starts.
    recovery_bracket_meditation: bool,
    #[cfg(test)]
    end_combat_invocations: u32,
    #[cfg(test)]
    start_combat_round_invocations: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum StatsEffect {
    UpdatePrompt([i32; 7]),
    UpdateShortScore([i32; 13]),
    StartCombatRound,
    EndCombat,
    SetRecoveryBracketCamping(bool),
    SetRecoveryBracketMeditation(bool),
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
        }
    }

    pub fn update_from_prompt(&mut self, stats: [i32; 7]) {
        let diff_hp = self.diff_hp;
        let diff_sp = self.diff_sp;
        let diff_ep = self.diff_ep;
        let diff_exp = self.diff_exp;
        let diff_money = self.diff_money;
        let combat_round_active = self.combat_round_active;
        let money = self.money;
        let recovery_bracket_camping = self.recovery_bracket_camping;
        let recovery_bracket_meditation = self.recovery_bracket_meditation;
        #[cfg(test)]
        let end_combat_invocations = self.end_combat_invocations;
        #[cfg(test)]
        let start_combat_round_invocations = self.start_combat_round_invocations;
        *self = Self::new(stats);
        self.diff_hp = diff_hp;
        self.diff_sp = diff_sp;
        self.diff_ep = diff_ep;
        self.diff_exp = diff_exp;
        self.diff_money = diff_money;
        self.combat_round_active = combat_round_active;
        self.money = money;
        self.recovery_bracket_camping = recovery_bracket_camping;
        self.recovery_bracket_meditation = recovery_bracket_meditation;
        #[cfg(test)]
        {
            self.end_combat_invocations = end_combat_invocations;
            self.start_combat_round_invocations = start_combat_round_invocations;
        }
    }

    pub fn update_from_short_score(&mut self, stats: [i32; 13]) {
        let diff_hp = self.diff_hp;
        let diff_sp = self.diff_sp;
        let diff_ep = self.diff_ep;
        let diff_exp = self.diff_exp;
        let diff_money = self.diff_money;
        let combat_round_active = self.combat_round_active;
        let recovery_bracket_camping = self.recovery_bracket_camping;
        let recovery_bracket_meditation = self.recovery_bracket_meditation;
        #[cfg(test)]
        let end_combat_invocations = self.end_combat_invocations;
        #[cfg(test)]
        let start_combat_round_invocations = self.start_combat_round_invocations;
        *self = Self::new_from_sc(stats);
        if combat_round_active {
            self.diff_hp += diff_hp;
            self.diff_sp += diff_sp;
            self.diff_ep += diff_ep;
            self.diff_exp += diff_exp;
            self.diff_money += diff_money;
        }
        self.combat_round_active = combat_round_active;
        self.recovery_bracket_camping = recovery_bracket_camping;
        self.recovery_bracket_meditation = recovery_bracket_meditation;
        #[cfg(test)]
        {
            self.end_combat_invocations = end_combat_invocations;
            self.start_combat_round_invocations = start_combat_round_invocations;
        }
    }

    pub(crate) fn start_combat_round(&mut self) {
        #[cfg(test)]
        {
            self.start_combat_round_invocations += 1;
        }
        self.clear_diffs();
        self.combat_round_active = true;
    }

    pub(crate) fn end_combat(&mut self) {
        #[cfg(test)]
        {
            self.end_combat_invocations += 1;
        }
        self.combat_round_active = false;
    }

    #[cfg(test)]
    pub fn end_combat_invocations(&self) -> u32 {
        self.end_combat_invocations
    }

    #[cfg(test)]
    pub fn start_combat_round_invocations(&self) -> u32 {
        self.start_combat_round_invocations
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

fn bold_white_style() -> Style {
    Style::default()
        .fg(palette::BOLD_WHITE)
        .add_modifier(Modifier::BOLD)
}

pub(crate) fn progress_color(value: f32) -> Color {
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
}
