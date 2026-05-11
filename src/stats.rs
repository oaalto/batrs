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
    soul_companion: Option<SoulCompanionStatus>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SoulCompanionStatus {
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

    pub fn update_from_prompt(&mut self, stats: [i32; 7]) {
        let soul_companion = self.soul_companion.clone();
        let money = self.money;
        *self = Self::new(stats);
        self.soul_companion = soul_companion;
        self.money = money;
    }

    pub fn update_from_short_score(&mut self, stats: [i32; 13]) {
        let soul_companion = self.soul_companion.clone();
        *self = Self::new_from_sc(stats);
        self.soul_companion = soul_companion;
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

    pub fn render_inline(&self) -> Line<'static> {
        let mut spans = Vec::new();
        spans.extend(self.inline_stat_spans("Hp", self.hp, self.max_hp, self.diff_hp));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_stat_spans("Sp", self.sp, self.max_sp, self.diff_sp));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_stat_spans("Ep", self.ep, self.max_ep, self.diff_ep));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_value_spans("Exp", self.exp, self.diff_exp));
        spans.push(Span::raw("  "));
        spans.extend(self.inline_value_spans("Money", self.money, self.diff_money));

        Line::from(spans)
    }

    pub fn render_soul_inline(&self) -> Line<'static> {
        let Some(soul_companion) = &self.soul_companion else {
            return Line::from("");
        };

        let mut spans = vec![
            Span::raw("Soul: "),
            Span::styled(
                format!("{}%", soul_companion.percent),
                Style::default().fg(progress_color(soul_companion.percent as f32 / 100.0)),
            ),
            Span::raw(" "),
        ];
        spans.extend(soul_description_spans(&soul_companion.description));

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
            Span::styled(
                max_value.to_string(),
                Style::default()
                    .fg(Color::White)
                    .add_modifier(Modifier::BOLD),
            ),
        ];

        if let Some(diff_text) = diff_value {
            let diff_color = if diff > 0 { Color::Green } else { Color::Red };
            spans.push(Span::raw(" "));
            spans.push(Span::raw("("));
            spans.push(Span::styled(diff_text, Style::default().fg(diff_color)));
            spans.push(Span::raw(")"));
        }

        spans
    }

    fn inline_value_spans(&self, label: &str, value: i32, diff: i32) -> Vec<Span<'static>> {
        let value_style = Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD);
        let mut spans = vec![
            Span::raw(format!("{label}: ")),
            Span::styled(value.to_string(), value_style),
        ];
        if diff != 0 {
            let diff_color = if diff > 0 { Color::Green } else { Color::Red };
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
            let color = if ch == '+' { Color::Green } else { Color::Red };
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

fn progress_color(value: f32) -> Color {
    if value < 0.1 {
        Color::Rgb(128, 0, 0)
    } else if value < 0.2 {
        Color::Red
    } else if value < 0.3 {
        Color::LightRed
    } else if value < 0.4 {
        Color::Yellow
    } else if value < 0.5 {
        Color::LightYellow
    } else if value < 0.6 {
        Color::Rgb(0, 0, 128)
    } else if value < 0.7 {
        Color::Blue
    } else if value < 0.8 {
        Color::LightBlue
    } else if value < 0.9 {
        Color::Rgb(0, 128, 0)
    } else {
        Color::Green
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
        assert_eq!(exp_val.style.fg, Some(Color::White));

        let exp_diff = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "-20")
            .expect("exp diff span");
        assert_eq!(exp_diff.style.fg, Some(Color::Red));

        let money_val = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "88888")
            .expect("money value span");
        assert!(
            money_val.style.add_modifier.contains(Modifier::BOLD),
            "money value should be bold"
        );
        assert_eq!(money_val.style.fg, Some(Color::White));

        let money_diff = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+15")
            .expect("money diff span");
        assert_eq!(money_diff.style.fg, Some(Color::Green));
    }

    #[test]
    fn render_soul_inline_includes_soul_companion_status_without_name() {
        let mut stats = Stats::default();
        stats.set_soul_companion(75, "resting".to_string());

        assert_eq!(line_text(&stats.render_soul_inline()), "Soul: 75% resting");
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
        assert_eq!(minus.style.fg, Some(Color::Red));

        let plus = line
            .spans
            .iter()
            .find(|s| s.content.as_ref() == "+")
            .expect("plus span");
        assert_eq!(plus.style.fg, Some(Color::Green));
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
            line.contains("Money: 2786"),
            "expected money from short score after prompt; got {line:?}"
        );
        assert!(
            !line.contains("Money: 0"),
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
            line.contains("Money: 500"),
            "expected money from short score after prompt; got {line:?}"
        );
        assert!(
            !line.contains("Money: 500 (+"),
            "diff should stay cleared when zero"
        );
    }

    #[test]
    fn prompt_clears_money_diff_like_other_diffs() {
        let mut stats = Stats::default();
        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 100, -5, 8, 0]);

        stats.update_from_prompt([1, 2, 3, 4, 5, 6, 7]);

        let line = line_text(&stats.render_inline());
        assert!(line.contains("Money: 100"));
        assert!(
            !line.contains("-5"),
            "money delta from short score should not persist after prompt-only line; got {line:?}"
        );
    }

    #[test]
    fn short_score_updates_preserve_soul_companion_status() {
        let mut stats = Stats::default();
        stats.set_soul_companion(45, "nearby".to_string());

        stats.update_from_short_score([1, 2, 0, 3, 4, 0, 5, 6, 0, 7, 0, 8, 0]);

        assert_eq!(line_text(&stats.render_soul_inline()), "Soul: 45% nearby");
    }
}
