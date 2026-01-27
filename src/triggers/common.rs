use crate::ansi::{AnsiCode, StyledLine};
use crate::automation::Action;
use crate::triggers::{TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy)]
enum HiliteTarget {
    Whole,
    Group(usize),
}

enum RuleCondition {
    FlagSet(&'static str),
}

enum RuleAction {
    Hilite {
        target: HiliteTarget,
        color: AnsiCode,
        bold: bool,
    },
    Echo {
        text: &'static str,
        color: AnsiCode,
        bold: bool,
    },
    Send(&'static str),
    SetFlag {
        key: &'static str,
        value: bool,
    },
}

enum RuleMatcher {
    Simple(&'static str),
    Regex(Regex),
}

struct Rule {
    priority: i32,
    order: usize,
    matcher: RuleMatcher,
    condition: Option<RuleCondition>,
    actions: Vec<RuleAction>,
}

enum MatchData<'a> {
    Simple,
    Regex(Captures<'a>),
}

impl RuleMatcher {
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
    fn condition_met(&self, ctx: &TriggerContext<'_>) -> bool {
        match self.condition {
            Some(RuleCondition::FlagSet(key)) => ctx.automation.flag_is_set(key),
            None => true,
        }
    }

    fn apply(
        &self,
        match_data: &MatchData<'_>,
        styled_line: &mut StyledLine,
        output_lines: &mut Vec<StyledLine>,
        actions: &mut Vec<Action>,
    ) {
        for action in self.actions.iter().filter(|action| {
            matches!(
                action,
                RuleAction::Hilite {
                    target: HiliteTarget::Whole,
                    ..
                }
            )
        }) {
            apply_rule_action(action, match_data, styled_line, output_lines, actions);
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
            apply_rule_action(action, match_data, styled_line, output_lines, actions);
        }
    }
}

fn apply_rule_action(
    action: &RuleAction,
    match_data: &MatchData<'_>,
    styled_line: &mut StyledLine,
    output_lines: &mut Vec<StyledLine>,
    actions: &mut Vec<Action>,
) {
    match action {
        RuleAction::Hilite {
            target: HiliteTarget::Whole,
            color,
            bold,
        } => {
            styled_line.set_line_color(*color, *bold);
        }
        RuleAction::Hilite {
            target: HiliteTarget::Group(index),
            color,
            bold,
        } => {
            if let MatchData::Regex(captures) = match_data {
                apply_capture_hilite(styled_line, captures, *index, *color, *bold);
            }
        }
        RuleAction::Echo { text, color, bold } => {
            let mut line = StyledLine::new(text);
            line.set_line_color(*color, *bold);
            output_lines.push(line);
        }
        RuleAction::Send(template) => {
            actions.push(Action::Send((*template).to_string()));
        }
        RuleAction::SetFlag { key, value } => {
            actions.push(Action::SetFlag((*key).to_string(), *value));
        }
    }
}

fn apply_capture_hilite(
    styled_line: &mut StyledLine,
    captures: &Captures<'_>,
    index: usize,
    color: AnsiCode,
    bold: bool,
) {
    let Some(m) = captures.get(index) else {
        return;
    };

    let start = byte_to_grapheme_index(&styled_line.plain_line, m.start());
    let end = byte_to_grapheme_index(&styled_line.plain_line, m.end());
    let len = styled_line.styled_chars.len();
    let start = start.min(len);
    let end = end.min(len);

    for i in start..end {
        styled_line.styled_chars[i].color = color;
        styled_line.styled_chars[i].bold = bold;
    }
}

fn byte_to_grapheme_index(text: &str, byte_index: usize) -> usize {
    text.get(..byte_index)
        .map(|slice| slice.graphemes(true).count())
        .unwrap_or_default()
}

fn tf_hilite(code: &str, target: HiliteTarget) -> RuleAction {
    let (color, bold) = tf_color(code).unwrap_or((AnsiCode::White, false));
    RuleAction::Hilite {
        target,
        color,
        bold,
    }
}

fn tf_echo(code: &str, text: &'static str) -> RuleAction {
    let (color, bold) = tf_color(code).unwrap_or((AnsiCode::White, false));
    RuleAction::Echo { text, color, bold }
}

fn tf_color(code: &str) -> Option<(AnsiCode, bool)> {
    let (bold, rest) = if let Some(stripped) = code.strip_prefix('B') {
        (true, stripped)
    } else {
        (false, code)
    };

    let color = match rest {
        "Cred" => AnsiCode::Red,
        "Cgreen" => AnsiCode::Green,
        "Cyellow" => AnsiCode::Yellow,
        "Cblue" => AnsiCode::Blue,
        "Cmagenta" => AnsiCode::Magenta,
        "Ccyan" => AnsiCode::Cyan,
        "Cwhite" => AnsiCode::White,
        _ => return None,
    };

    Some((color, bold))
}

fn push_rule(
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

lazy_static! {
    static ref RULES: Vec<Rule> = {
        let mut rules = Vec::new();
        let mut order = 0usize;

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"^[\*]+ Round .* [\*]+$").unwrap()),
            10000,
            None,
            vec![
                RuleAction::Send("@scan all"),
                RuleAction::Send("@sc"),
                RuleAction::SetFlag {
                    key: "in_battle",
                    value: true,
                },
            ],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("You are not in combat right now."),
            1000,
            None,
            vec![RuleAction::SetFlag {
                key: "in_battle",
                value: false,
            }],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"(.+) is DEAD, R.I.P.").unwrap()),
            10000,
            None,
            vec![
                tf_hilite("BCred", HiliteTarget::Whole),
                RuleAction::Send("@scan"),
            ],
        );

        for (pattern, color) in [
            ("is in excellent shape", "BCgreen"),
            ("is in a good shape", "Cgreen"),
            ("is slightly hurt", "Ccyan"),
            ("is noticeably hurt", "BCcyan"),
            ("is not in a good shape", "Cyellow"),
            ("is in bad shape", "BCyellow"),
            ("is in very bad shape", "BCred"),
            ("is near death", "Cred"),
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Regex(Regex::new(pattern).unwrap()),
                10,
                None,
                vec![tf_hilite(color, HiliteTarget::Whole)],
            );
        }

        for pattern in [
            "You cannot leave, you have been AMBUSHED.",
            "You've been ambushed!",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![tf_hilite("BCred", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(
                Regex::new(r"You wish your wounds would stop (BLEEDING) so much!").unwrap(),
            ),
            10,
            None,
            vec![tf_hilite("BCred", HiliteTarget::Group(1))],
        );

        for (pattern, color) in [
            (r"You score a (CRITICAL) hit!", "Cwhite"),
            (r"You score a (\*CRITICAL\*) hit!", "BCwhite"),
            (r"You score a (.*CRITICAL.*) hit!", "BCwhite"),
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Regex(Regex::new(pattern).unwrap()),
                10,
                None,
                vec![tf_hilite(color, HiliteTarget::Group(1))],
            );
        }

        for (pattern, color) in [
            (
                "You awaken from your short rest, and feel slightly better.",
                "BCgreen",
            ),
            ("You feel a bit tired.", "BCyellow"),
            ("You stretch yourself and consider camping.", "BCyellow"),
            ("You feel like camping a little.", "BCyellow"),
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![tf_hilite(color, HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple(
                "You feel in harmony with yourself, the universe and life in general.",
            ),
            10,
            None,
            vec![tf_hilite("BCyellow", HiliteTarget::Whole)],
        );

        for (pattern, color) in [
            ("You are stunned.", "BCred"),
            ("You are no longer stunned.", "BCgreen"),
            ("Your inner strength keeps your head clear!", "BCgreen"),
            ("...BUT you break it off.", "BCgreen"),
            (
                "...BUT you break it off with intense concentration.",
                "BCgreen",
            ),
            ("It doesn't hurt at all!", "BCgreen"),
            ("Your thoughts still feel clear and calm.", "BCgreen"),
            ("You are stunned and unable to do anything.", "Cred"),
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![tf_hilite(color, HiliteTarget::Whole)],
            );
        }

        for pattern in [
            "You get hit, and your eyes lose focus slightly.",
            "You try to concentrate but your head spins like a whirligig!",
            "You lose connection to reality, becoming truly STUNNED.",
            "You become somewhat confused, losing your edge.",
            "Your mind reels and the world becomes blurred.",
            "You get hit badly, and have problems staying in balance.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![
                    tf_hilite("BCred", HiliteTarget::Whole),
                    tf_echo("BCred", "STUNNED!"),
                ],
            );
        }

        for pattern in [
            r"You (stun|STUN)",
            r"Your attack causes (.+) to lose focus slightly.",
            r"You hurt (.+) who seems to become somewhat confused.",
            r"You make (.+) stagger helplessly in pain and confusion.",
            r"You STUN (.+), who loses connection to reality.",
            r"You cause (.+) world to become blurred and unfocused.",
            r"(.+) is suddenly almost unable to stay in balance.",
            r"(.+) is STUNNED.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Regex(Regex::new(pattern).unwrap()),
                100,
                None,
                vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"..WHO breaks it off.").unwrap()),
            100,
            None,
            vec![tf_hilite("Cred", HiliteTarget::Whole)],
        );

        for pattern in [
            "You are disturbed by something, your spell misfires.",
            "Your concentration drifts away as you think you feel a malignant aura.",
            "You stumble and lose your concentration.",
            "Your concentration fails and so does your spell.",
            "You lose touch with the magic and the spell fizzles.",
            "You stutter the magic words and fail the spell.",
            "Your mind plays a trick with you and you fail in your spell.",
            "You fail miserably in your spell.",
            "Your spell just sputters.",
            "Something touches you and spoils your concentration ruining the spell.",
            "You poke yourself in the eye and your spell misfires.",
            "You fail to chant the spell correctly.",
            "You do not have enough spell points to cast the spell.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![tf_hilite("BCred", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You falter and fumble the spell.").unwrap()),
            10,
            None,
            vec![tf_hilite("BCyellow", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You clap your hands and whisper '(.+)'").unwrap()),
            10,
            None,
            vec![tf_hilite("BCwhite", HiliteTarget::Group(1))],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(
                Regex::new(r"You watch with self-pride as your (.+) hits (.+).").unwrap(),
            ),
            10,
            None,
            vec![
                tf_hilite("BCgreen", HiliteTarget::Group(1)),
                tf_hilite("BCwhite", HiliteTarget::Group(2)),
            ],
        );
        for pattern in [
            r"You boom in sinister voice '(.+)'",
            r"You utter the magic words '(.+)'",
            r"You fill up your cheeks with air and exhale '(.+)'",
            r"You slowly cut your arm with your finger-nail and darkly whisper '(.+)'",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Regex(Regex::new(pattern).unwrap()),
                10,
                None,
                vec![tf_hilite("BCwhite", HiliteTarget::Group(1))],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"[A|An] (.+) hits you.").unwrap()),
            10,
            None,
            vec![tf_hilite("BCred", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple(
                "You feel like you managed to channel additional POWER to your spell.",
            ),
            10,
            None,
            vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You are about to (DIE)!").unwrap()),
            10,
            None,
            vec![tf_hilite("BCred", HiliteTarget::Group(1))],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"^(.+): ([#]+)$").unwrap()),
            10,
            None,
            vec![
                tf_hilite("BCyellow", HiliteTarget::Group(1)),
                tf_hilite("BCwhite", HiliteTarget::Group(2)),
            ],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("You sizzle with magical energy."),
            10,
            None,
            vec![tf_hilite("BCmagenta", HiliteTarget::Whole)],
        );

        for pattern in [
            "Tactically shielded, you thwart a potentially devastating critical strike.",
            "Guarding flaws, you endure, deflecting a looming critical strike.",
            "Protecting flaws, you endure, evading a lethal critical strike.",
            "Covering weak spots, you defy a critical strike's impact.",
            "Adapting swiftly, you nullify the impact of a critical strike.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                0,
                None,
                vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
            );
        }

        for pattern in [
            "The desire to choose between good and evil overwhelms you, causing you to",
            "inflict damage upon yourself.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                10,
                None,
                vec![tf_hilite("Cred", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"(.+) spills some of (.+) essence.").unwrap()),
            10,
            None,
            vec![tf_hilite("Cblue", HiliteTarget::Whole)],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"(.+) spills some of (.+) essence.").unwrap()),
            10,
            Some(RuleCondition::FlagSet("in_battle")),
            vec![RuleAction::Send(
                "@get all essence;keep all essence;put all essence in {rig}",
            )],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("You discover a glowing ball of concentrated zinium <<radiating>>"),
            10,
            None,
            vec![
                tf_hilite("Cblue", HiliteTarget::Whole),
                RuleAction::Send("@keep all orb;put all orb in {rig}"),
            ],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You discover Rixx-Tec blueprint of").unwrap()),
            10,
            None,
            vec![
                tf_hilite("Cblue", HiliteTarget::Whole),
                RuleAction::Send("@keep all blueprint;store blueprint"),
            ],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You shiver and suffer from (POISON)!!").unwrap()),
            1000,
            None,
            vec![
                tf_hilite("Cred", HiliteTarget::Whole),
                tf_hilite("BCred", HiliteTarget::Group(1)),
            ],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("The sauna cured your poison!"),
            1000,
            None,
            vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"You have been scheduled for a trip to (.+).").unwrap()),
            1000,
            None,
            vec![RuleAction::Send(
                "@put mithril in purse;put batium in purse;put anipium in purse;put platinum in purse",
            )],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(
                Regex::new(r"You pay the ticketmaster (.+) gold, and he lets you board the ship.")
                    .unwrap(),
            ),
            1000,
            None,
            vec![RuleAction::Send("@get 250 platinum from purse")],
        );

        for pattern in [
            r"got mad at hostile actions.",
            r"is disturbed by spellcasting.",
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Regex(Regex::new(pattern).unwrap()),
                1000,
                None,
                vec![tf_hilite("Cred", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("Everything no longer seems so red."),
            1000,
            None,
            vec![tf_hilite("Cred", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(
                Regex::new(r"You feel like you just got slightly better in (.+).").unwrap(),
            ),
            1000,
            None,
            vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
        );

        for (pattern, color) in [
            ("You enter a frenzy, speeding up your actions!", "Cgreen"),
            ("You slip out of your frenzy.", "Cred"),
        ] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                1000,
                None,
                vec![tf_hilite(color, HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("You are already fighting!"),
            1000,
            None,
            vec![tf_hilite("Cred", HiliteTarget::Whole)],
        );

        for pattern in ["You dodge.", "You parry.", "...AND riposte."] {
            push_rule(
                &mut rules,
                &mut order,
                RuleMatcher::Simple(pattern),
                100,
                None,
                vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
            );
        }

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"(.+) misses.").unwrap()),
            100,
            None,
            vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"Your (.+) breaks into zillions of pieces.").unwrap()),
            0,
            None,
            vec![tf_hilite("Cred", HiliteTarget::Whole)],
        );

        rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.order.cmp(&b.order))
        });
        rules
    };
}

pub fn trigger(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput {
    let mut output = TriggerOutput::default();
    let plain_line = styled_line.plain_line.clone();
    if let Some(rig) = ctx.rig
        && !rig.is_empty()
    {
        ctx.automation.set_var("rig", rig.to_string());
    }

    for rule in RULES.iter() {
        let Some(match_data) = rule.matcher.match_line(&plain_line) else {
            continue;
        };
        if !rule.condition_met(ctx) {
            continue;
        }
        rule.apply(
            &match_data,
            styled_line,
            &mut output.lines,
            &mut output.actions,
        );
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn run_trigger(line: &str, rig: Option<&str>) -> (TriggerOutput, StyledLine, Automation) {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut automation,
            rig,
        };
        let mut styled_line = StyledLine::new(line);
        let output = trigger(&mut ctx, &mut styled_line);

        (output, styled_line, automation)
    }

    #[test]
    fn battle_round_sets_actions_and_flag() {
        let (output, _line, _automation) = run_trigger("*** Round 1 ***", None);
        let mut saw_scan_all = false;
        let mut saw_sc = false;
        let mut saw_flag = false;

        for action in &output.actions {
            match action {
                Action::Send(cmd) if cmd == "@scan all" => saw_scan_all = true,
                Action::Send(cmd) if cmd == "@sc" => saw_sc = true,
                Action::SetFlag(key, value) if key == "in_battle" && *value => saw_flag = true,
                _ => {}
            }
        }

        assert!(saw_scan_all);
        assert!(saw_sc);
        assert!(saw_flag);
    }

    #[test]
    fn stunned_lines_echo_local_notice() {
        let (output, _line, _automation) =
            run_trigger("You get hit, and your eyes lose focus slightly.", None);

        assert_eq!(output.lines.len(), 1);
        assert_eq!(output.lines[0].plain_line, "STUNNED!");
        let first_char = &output.lines[0].styled_chars[0];
        assert_eq!(first_char.color, AnsiCode::Red);
        assert!(first_char.bold);
    }

    #[test]
    fn zinium_ball_sends_keep_command() {
        let (output, _line, _automation) = run_trigger(
            "You discover a glowing ball of concentrated zinium <<radiating>>",
            Some("pack"),
        );
        let saw_send = output.actions.iter().any(|action| {
            matches!(
                action,
                Action::Send(cmd) if cmd == "@keep all orb;put all orb in {rig}"
            )
        });

        assert!(saw_send);
    }
}
