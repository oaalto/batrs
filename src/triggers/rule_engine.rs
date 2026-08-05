use crate::ansi::TextStyle;
use crate::automation::Action;
use crate::triggers::money_summary::push_money_summary;
use crate::triggers::{LineEffect, TriggerEffects, TriggerFacts};
use regex::{Captures, Regex};

#[derive(Clone, Copy)]
pub(crate) enum HiliteTarget {
    Whole,
    Group(usize),
}

pub(crate) enum RuleCondition {
    FlagSet(&'static str),
}

pub(crate) enum RuleAction {
    Hilite {
        target: HiliteTarget,
        style: TextStyle,
    },
    MoneySummary {
        list_index: usize,
    },
    Echo {
        text: &'static str,
        style: TextStyle,
    },
    Send(&'static str),
}

pub(crate) enum RuleMatcher {
    Simple(&'static str),
    Regex(Regex),
}

pub(crate) struct Rule {
    priority: i32,
    order: usize,
    pub(crate) matcher: RuleMatcher,
    condition: Option<RuleCondition>,
    actions: Vec<RuleAction>,
}

enum MatchData<'a> {
    Simple,
    Regex(Captures<'a>),
}

impl RuleMatcher {
    pub(crate) fn pattern_display(&self) -> String {
        match self {
            RuleMatcher::Simple(pattern) => (*pattern).to_string(),
            RuleMatcher::Regex(regex) => regex.as_str().to_string(),
        }
    }

    fn match_line<'a>(&self, line: &'a str) -> Option<MatchData<'a>> {
        match self {
            RuleMatcher::Simple(pattern) => {
                if line == *pattern {
                    Some(MatchData::Simple)
                } else {
                    None
                }
            }
            RuleMatcher::Regex(regex) => regex.captures(line).map(MatchData::Regex),
        }
    }
}

impl Rule {
    fn condition_met(&self, facts: &TriggerFacts) -> bool {
        match self.condition {
            Some(RuleCondition::FlagSet(key)) => facts.flag_is_set(key),
            None => true,
        }
    }

    fn apply(&self, match_data: &MatchData<'_>, output: &mut TriggerEffects) {
        for action in self.actions.iter().filter(|action| {
            matches!(
                action,
                RuleAction::Hilite {
                    target: HiliteTarget::Whole,
                    ..
                }
            )
        }) {
            apply_rule_action(action, match_data, output);
        }

        for action in &self.actions {
            if matches!(
                action,
                RuleAction::Hilite {
                    target: HiliteTarget::Whole,
                    ..
                }
            ) {
                continue;
            }
            apply_rule_action(action, match_data, output);
        }
    }
}

pub(crate) fn describe_rule_actions(actions: &[RuleAction]) -> String {
    let mut parts = Vec::new();
    for action in actions {
        match action {
            RuleAction::Hilite { .. } => parts.push("Highlight line"),
            RuleAction::MoneySummary { .. } => parts.push("Show money summary"),
            RuleAction::Echo { text, .. } => parts.push(*text),
            RuleAction::Send(template) => parts.push(template),
        }
    }
    parts.dedup();
    parts.join("; ")
}

pub(crate) fn rule_catalog_entries(rules: &[Rule]) -> Vec<crate::command::TriggerCatalogEntry> {
    rules
        .iter()
        .map(|rule| {
            crate::command::TriggerCatalogEntry::new(
                rule.matcher.pattern_display(),
                describe_rule_actions(&rule.actions),
            )
        })
        .collect()
}

pub(crate) fn apply_rules<'a>(
    // ponytail: callers must pass rules already sorted via sort_rules (priority desc, order asc)
    rules: impl IntoIterator<Item = &'a Rule>,
    plain_line: &str,
    facts: &TriggerFacts,
    output: &mut TriggerEffects,
) {
    for rule in rules {
        let Some(match_data) = rule.matcher.match_line(plain_line) else {
            continue;
        };
        if !rule.condition_met(facts) {
            continue;
        }
        rule.apply(&match_data, output);
    }
}

fn apply_rule_action(action: &RuleAction, match_data: &MatchData<'_>, output: &mut TriggerEffects) {
    match action {
        RuleAction::Hilite {
            target: HiliteTarget::Whole,
            style,
        } => {
            output.original.edits.push(LineEffect::StyleLine(*style));
        }
        RuleAction::Hilite {
            target: HiliteTarget::Group(index),
            style,
        } => {
            if let MatchData::Regex(captures) = match_data {
                apply_capture_hilite(output, captures, *index, *style);
            }
        }
        RuleAction::MoneySummary { list_index } => {
            if let MatchData::Regex(captures) = match_data
                && let Some(m) = captures.get(*list_index)
            {
                push_money_summary(m.as_str(), &mut output.lines);
            }
        }
        RuleAction::Echo { text, style } => {
            let mut line = crate::ansi::StyledLine::new(text);
            line.set_line_style(*style);
            output.lines.push(line);
        }
        RuleAction::Send(template) => {
            output.actions.push(Action::Send((*template).to_string()));
        }
    }
}

fn apply_capture_hilite(
    output: &mut TriggerEffects,
    captures: &Captures<'_>,
    index: usize,
    style: TextStyle,
) {
    let Some(m) = captures.get(index) else {
        return;
    };
    output.original.edits.push(LineEffect::StylePlainByteRange {
        range: m.range(),
        style,
    });
}

pub(crate) fn tf_hilite(code: &str, target: HiliteTarget) -> RuleAction {
    let style = tf_style(code);
    RuleAction::Hilite { target, style }
}

pub(crate) fn tf_echo(code: &str, text: &'static str) -> RuleAction {
    let style = tf_style(code);
    RuleAction::Echo { text, style }
}

pub(crate) fn tf_style(code: &str) -> TextStyle {
    match code {
        "Cred" => TextStyle::RED,
        "Cgreen" => TextStyle::GREEN,
        "Cyellow" => TextStyle::YELLOW,
        "Cblue" => TextStyle::BLUE,
        "Cmagenta" => TextStyle::MAGENTA,
        "Ccyan" => TextStyle::CYAN,
        "Cwhite" => TextStyle::WHITE,
        "BCred" => TextStyle::BRIGHT_RED,
        "BCgreen" => TextStyle::BRIGHT_GREEN,
        "BCyellow" => TextStyle::BRIGHT_YELLOW,
        "BCblue" => TextStyle::BRIGHT_BLUE,
        "BCmagenta" => TextStyle::BRIGHT_MAGENTA,
        "BCcyan" => TextStyle::BRIGHT_CYAN,
        "BCwhite" => TextStyle::BRIGHT_WHITE,
        _ => TextStyle::WHITE,
    }
}

pub(crate) fn sort_rules(rules: &mut [Rule]) {
    rules.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.order.cmp(&b.order))
    });
}

pub(crate) fn push_rule(
    rules: &mut Vec<Rule>,
    order: &mut usize,
    matcher: RuleMatcher,
    priority: i32,
    condition: Option<RuleCondition>,
    actions: Vec<RuleAction>,
) {
    rules.push(Rule {
        priority,
        order: *order,
        matcher,
        condition,
        actions,
    });
    *order += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::automation::Action;

    fn run_rule(line: &str, rule: &Rule, facts: &TriggerFacts) -> (TriggerEffects, StyledLine) {
        let mut output = TriggerEffects::default();
        apply_rules(std::iter::once(rule), line, facts, &mut output);
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        (output, styled)
    }

    #[test]
    fn simple_matcher_matches_exact_line() {
        let mut rules = Vec::new();
        let mut order = 0;
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("hello"),
            10,
            None,
            vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
        );

        let (output, styled) = run_rule("hello", &rules[0], &TriggerFacts::default());
        assert!(!output.original.edits.is_empty());
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);

        let (output, _) = run_rule("goodbye", &rules[0], &TriggerFacts::default());
        assert!(output.original.edits.is_empty());
    }

    #[test]
    fn regex_matcher_captures_group_for_hilite() {
        let mut rules = Vec::new();
        let mut order = 0;
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"^(.+) dies$").unwrap()),
            10,
            None,
            vec![tf_hilite("Cred", HiliteTarget::Group(1))],
        );

        let (output, styled) = run_rule("orc dies", &rules[0], &TriggerFacts::default());
        assert_eq!(output.original.edits.len(), 1);
        assert_eq!(styled.styled_chars[0].color, AnsiCode::Red);
    }

    #[test]
    fn flag_condition_gates_rule_application() {
        let mut rules = Vec::new();
        let mut order = 0;
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("drain"),
            10,
            Some(RuleCondition::FlagSet("is_lich")),
            vec![RuleAction::Send("@lich drain")],
        );

        let (output, _) = run_rule("drain", &rules[0], &TriggerFacts::default());
        assert!(output.actions.is_empty());

        let mut flags = std::collections::HashMap::new();
        flags.insert("is_lich".to_string(), true);
        let facts = TriggerFacts::new(
            flags,
            Default::default(),
            None,
            None,
            crate::guilds::MonkSkillsConfig::default(),
        );
        let (output, _) = run_rule("drain", &rules[0], &facts);
        assert!(matches!(&output.actions[0], Action::Send(cmd) if cmd == "@lich drain"));
    }

    #[test]
    fn echo_action_emits_styled_line() {
        let mut rules = Vec::new();
        let mut order = 0;
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("stunned"),
            10,
            None,
            vec![tf_echo("BCred", "STUNNED!")],
        );

        let (output, _) = run_rule("stunned", &rules[0], &TriggerFacts::default());
        assert_eq!(output.lines.len(), 1);
        assert_eq!(output.lines[0].plain_line, "STUNNED!");
        assert_eq!(output.lines[0].styled_chars[0].color, AnsiCode::Red);
        assert!(output.lines[0].styled_chars[0].bold);
    }
}
