use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::combat_awareness::NOT_IN_COMBAT_LINE;
use crate::triggers::{LineEffect, TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::{Captures, Regex};
use std::sync::{Arc, Mutex};

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
            let mut line = StyledLine::new(text);
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

fn tf_hilite(code: &str, target: HiliteTarget) -> RuleAction {
    let style = tf_style(code);
    RuleAction::Hilite { target, style }
}

fn tf_echo(code: &str, text: &'static str) -> RuleAction {
    let style = tf_style(code);
    RuleAction::Echo { text, style }
}

fn tf_style(code: &str) -> TextStyle {
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

#[derive(Clone, Copy, Eq, PartialEq)]
enum CoinType {
    Anipium,
    Batium,
    Mithril,
    Platinum,
}

impl CoinType {
    fn from_str(value: &str) -> Option<Self> {
        match value {
            "anipium" => Some(Self::Anipium),
            "batium" => Some(Self::Batium),
            "mithril" => Some(Self::Mithril),
            "platinum" => Some(Self::Platinum),
            _ => None,
        }
    }

    fn display_name(self) -> &'static str {
        match self {
            Self::Anipium => "Anipium",
            Self::Batium => "Batium",
            Self::Mithril => "Mithril",
            Self::Platinum => "Platinum",
        }
    }

    fn multiplier(self) -> u64 {
        match self {
            Self::Anipium => 50,
            Self::Batium => 100,
            Self::Mithril => 500,
            Self::Platinum => 10,
        }
    }

    fn order_index(self) -> usize {
        match self {
            Self::Anipium => 0,
            Self::Batium => 1,
            Self::Mithril => 2,
            Self::Platinum => 3,
        }
    }
}

fn push_money_summary(list_text: &str, output_lines: &mut Vec<StyledLine>) {
    let normalized = list_text.trim().replace(" and ", ", ");
    let mut counts = [None; 4];
    let mut last_index = None;

    for entry in normalized.split(", ") {
        let mut parts = entry.splitn(2, ' ');
        let amount = parts.next().and_then(|value| value.parse::<u64>().ok());
        let coin = parts.next().and_then(CoinType::from_str);

        let (Some(amount), Some(coin)) = (amount, coin) else {
            return;
        };

        let idx = coin.order_index();
        if counts[idx].is_some() {
            return;
        }
        if let Some(last_idx) = last_index
            && idx <= last_idx
        {
            return;
        }

        counts[idx] = Some(amount);
        last_index = Some(idx);
    }

    if counts.iter().all(|value| value.is_none()) {
        return;
    }

    let mut total = 0u64;
    for coin in [
        CoinType::Platinum,
        CoinType::Anipium,
        CoinType::Batium,
        CoinType::Mithril,
    ] {
        if let Some(amount) = counts[coin.order_index()] {
            let value = amount * coin.multiplier();
            total += value;
            output_lines.push(StyledLine::new(&format!(
                "{} {} = {}",
                coin.display_name(),
                amount,
                value
            )));
        }
    }

    output_lines.push(StyledLine::new(&format!("Total = {}", total)));
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

static COMPANION_RULES_CACHE: Mutex<Option<(String, Arc<Vec<Rule>>)>> = Mutex::new(None);

fn companion_rules_arc(name: &str) -> Arc<Vec<Rule>> {
    let Some(name) = companion_rule_name(name) else {
        return Arc::new(Vec::new());
    };

    let mut guard = COMPANION_RULES_CACHE.lock().unwrap();
    if guard
        .as_ref()
        .is_some_and(|(stored, _)| stored.as_str() == name)
    {
        return Arc::clone(&guard.as_ref().unwrap().1);
    }
    let built = Arc::new(build_companion_rules(&name));
    *guard = Some((name, Arc::clone(&built)));
    built
}

fn companion_rule_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let mut chars = trimmed.chars();
    let first = chars.next()?;

    let mut normalized = first.to_uppercase().collect::<String>();
    normalized.push_str(&chars.as_str().to_lowercase());
    Some(normalized)
}

/// Soul-companion combat lines for Fueryon/Odefu-style companions, with the character name
/// taken from the application instead of hardcoded Fueryon/Odefu.
fn build_companion_rules(name: &str) -> Vec<Rule> {
    let escaped = regex::escape(name);
    const PRIORITY: i32 = 1000;
    let mut rules = Vec::new();
    let mut order = 0usize;

    // "{name} hits <other> …" — attacker is the player character (green), count is group 2.
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(
                r"^{} hits (.+) (?:once|twice|thrice|\d+ times) (.+)\.$",
                escaped
            ))
            .unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(Regex::new(&format!(r"^{} hits (.+) (once) (.+)\.$", escaped)).unwrap()),
        PRIORITY,
        None,
        vec![tf_hilite("Cblue", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^{} hits (.+) (twice) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cmagenta", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^{} hits (.+) (thrice) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("BCred", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^{} hits (.+) (\d+ times) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cred", HiliteTarget::Group(2))],
    );

    // "<other> hits {name} …" — player is the target (magenta), count is group 2.
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(
                r"^(.+) hits {} (?:once|twice|thrice|\d+ times) (.+)\.$",
                escaped
            ))
            .unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cmagenta", HiliteTarget::Whole)],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(Regex::new(&format!(r"^(.+) hits {} (once) (.+)\.$", escaped)).unwrap()),
        PRIORITY,
        None,
        vec![tf_hilite("Cblue", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^(.+) hits {} (twice) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("BCmagenta", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^(.+) hits {} (thrice) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("BCred", HiliteTarget::Group(2))],
    );
    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(r"^(.+) hits {} (\d+ times) (.+)\.$", escaped)).unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cred", HiliteTarget::Group(2))],
    );

    push_rule(
        &mut rules,
        &mut order,
        RuleMatcher::Regex(
            Regex::new(&format!(
                r"^A blue-glowing soul companion \[{}\]\.?$",
                escaped
            ))
            .unwrap(),
        ),
        PRIORITY,
        None,
        vec![tf_hilite("Cblue", HiliteTarget::Whole)],
    );

    rules.sort_by(|a, b| {
        b.priority
            .cmp(&a.priority)
            .then_with(|| a.order.cmp(&b.order))
    });
    rules
}

lazy_static! {
    static ref RULES: Vec<Rule> = {
        let mut rules = Vec::new();
        let mut order = 0usize;

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"^It contains (.+) coins\.$").unwrap()),
            1000,
            None,
            vec![RuleAction::MoneySummary { list_index: 1 }],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("There is not that much platinum in the purse."),
            1000,
            None,
            vec![RuleAction::Send("@get 50 anipium from purse")],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("There is not that much anipium in the purse."),
            1000,
            None,
            vec![RuleAction::Send("@get 25 batium from purse")],
        );
        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("There is not that much batium in the purse."),
            1000,
            None,
            vec![RuleAction::Send("@get 5 mithril from purse")],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Regex(Regex::new(r"^[^ ]+ is not wounded\.").unwrap()),
            10,
            None,
            vec![tf_hilite("BCgreen", HiliteTarget::Whole)],
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
            r"You raise your hands, gaze up and chant '(.+)'",
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
            RuleMatcher::Regex(Regex::new(r"^(?:A|An) (.+) hits you\.$").unwrap()),
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

        // boon/race highlights, lich drain / dig grave (generic)
        for (pattern, color) in [
            ("You leech some of your foes energy.", "BCgreen"),
            (
                "You realize a more effective way to use your horns!",
                "BCyellow",
            ),
            (
                "Your wings glow as they absorb more magic!",
                "BCyellow",
            ),
            ("Whee, your neat fur is dry again!", "Cgreen"),
            (
                "You gain insight to warhorse philosophy!",
                "BCyellow",
            ),
            (
                "You learn more about the praying mantis tactics!",
                "BCyellow",
            ),
            ("The water BURNS your skin.", "BCred"),
            (
                "You feel exhausted, being here in the dark.",
                "BCred",
            ),
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
            RuleMatcher::Simple(NOT_IN_COMBAT_LINE),
            1000,
            Some(RuleCondition::FlagSet("is_lich")),
            vec![RuleAction::Send("@lich drain")],
        );

        push_rule(
            &mut rules,
            &mut order,
            RuleMatcher::Simple("The consumed life force fills your being with ecstacy!"),
            10,
            None,
            vec![RuleAction::Send("@dig grave")],
        );

        rules.sort_by(|a, b| {
            b.priority
                .cmp(&a.priority)
                .then_with(|| a.order.cmp(&b.order))
        });
        rules
    };
}

pub fn trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
    let mut output = TriggerEffects::default();
    let plain_line = line.plain_line.to_string();
    if let Some(rig) = facts.rig()
        && !rig.is_empty()
    {
        output
            .actions
            .push(Action::SetVar("rig".to_string(), rig.to_string()));
    }

    let companion_rules = facts
        .player_name()
        .map(companion_rules_arc)
        .unwrap_or_else(|| Arc::new(Vec::new()));

    for rule in RULES.iter().chain(companion_rules.iter()) {
        let Some(match_data) = rule.matcher.match_line(&plain_line) else {
            continue;
        };
        if !rule.condition_met(facts) {
            continue;
        }
        rule.apply(&match_data, &mut output);
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::automation::Automation;
    use unicode_segmentation::UnicodeSegmentation;

    fn run_trigger(
        line: &str,
        rig: Option<&str>,
        player_name: Option<&str>,
    ) -> (TriggerEffects, StyledLine, Automation) {
        run_trigger_with_setup(line, rig, player_name, |_| {})
    }

    fn run_trigger_with_setup(
        line: &str,
        rig: Option<&str>,
        player_name: Option<&str>,
        setup: impl FnOnce(&mut Automation),
    ) -> (TriggerEffects, StyledLine, Automation) {
        let mut automation = Automation::new();
        setup(&mut automation);
        let facts = TriggerFacts::new(
            automation.snapshot_flags(),
            automation.snapshot_vars(),
            rig,
            player_name,
        );
        let mut styled_line = StyledLine::new(line);
        let output = trigger(&TriggerLine::new(line), &facts);
        output.apply_line_effects_to(&mut styled_line);

        (output, styled_line, automation)
    }

    #[test]
    fn stunned_lines_echo_local_notice() {
        let (output, _line, _automation) = run_trigger(
            "You get hit, and your eyes lose focus slightly.",
            None,
            None,
        );

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
            None,
        );
        let saw_send = output.actions.iter().any(|action| {
            matches!(
                action,
                Action::Send(cmd) if cmd == "@keep all orb;put all orb in {rig}"
            )
        });

        assert!(saw_send);
    }

    #[test]
    fn money_summary_allows_missing_coin_types() {
        let (output, _line, _automation) =
            run_trigger("It contains 2 anipium and 1 platinum coins.", None, None);

        let lines: Vec<&str> = output
            .lines
            .iter()
            .map(|line| line.plain_line.as_str())
            .collect();
        assert_eq!(
            lines,
            vec!["Platinum 1 = 10", "Anipium 2 = 100", "Total = 110"]
        );
    }

    #[test]
    fn soul_companion_announcement_matches_bracketed_player_name() {
        let text = "A blue-glowing soul companion [Nynn].";
        let (_output, styled, _automation) = run_trigger(text, None, Some("Nynn"));
        for styled_char in &styled.styled_chars {
            assert_eq!(styled_char.color, AnsiCode::Blue, "whole line blue");
        }
    }

    #[test]
    fn soul_companion_announcement_requires_application_player_name() {
        let text = "A blue-glowing soul companion [Nynn].";
        let (_output, styled, _automation) = run_trigger(text, None, Some("Other"));
        assert_eq!(
            styled.styled_chars[0].color,
            AnsiCode::DefaultColor,
            "wrong name: no highlight"
        );
    }

    #[test]
    fn avatar_hits_other_highlights_once_in_blue() {
        let text = "Nynn hits orc once with force.";
        let (_output, styled, _automation) = run_trigger(text, None, Some("Nynn"));
        let once_byte = text.find("once").expect("once in line");
        let idx = styled
            .plain_line
            .get(..once_byte)
            .map(|s| s.graphemes(true).count())
            .unwrap_or(0);
        assert_eq!(styled.styled_chars[idx].color, AnsiCode::Blue);
        assert_eq!(styled.styled_chars[idx + 1].color, AnsiCode::Blue);
        assert_eq!(styled.styled_chars[idx + 2].color, AnsiCode::Blue);
        assert_eq!(styled.styled_chars[idx + 3].color, AnsiCode::Blue);
    }

    #[test]
    fn avatar_hits_other_uses_capitalized_player_name_for_digit_count() {
        let text = "Odefu hits Man 4 times causing a nasty laceration.";
        let (_output, styled, _automation) = run_trigger(text, None, Some("odefu"));
        let count_byte = text.find("4 times").expect("count in line");
        let idx = styled
            .plain_line
            .get(..count_byte)
            .map(|s| s.graphemes(true).count())
            .unwrap_or(0);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert_eq!(styled.styled_chars[idx].color, AnsiCode::Red);
    }

    #[test]
    fn avatar_hits_other_uses_capitalized_player_name_for_twice() {
        let text = "Odefu hits Man twice inducing a nasty lesion.";
        let (_output, styled, _automation) = run_trigger(text, None, Some("odefu"));
        let twice_byte = text.find("twice").expect("twice in line");
        let idx = styled
            .plain_line
            .get(..twice_byte)
            .map(|s| s.graphemes(true).count())
            .unwrap_or(0);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
        assert_eq!(styled.styled_chars[idx].color, AnsiCode::Magenta);
    }

    #[test]
    fn misc_leech_line_hilite_green_bold() {
        let (_output, styled, _) = run_trigger("You leech some of your foes energy.", None, None);
        assert!(
            styled
                .styled_chars
                .iter()
                .all(|c| { c.color == AnsiCode::Green && c.bold })
        );
    }

    #[test]
    fn lich_not_in_combat_sends_drain_when_is_lich() {
        let (output, _, _) =
            run_trigger_with_setup("You are not in combat right now.", None, None, |auto| {
                auto.set_flag("is_lich", true);
            });
        assert!(
            output
                .actions
                .iter()
                .any(|a| matches!(a, Action::Send(cmd) if cmd == "@lich drain"))
        );
    }

    #[test]
    fn lich_not_in_combat_skips_drain_without_flag() {
        let (output, _, _) = run_trigger("You are not in combat right now.", None, None);
        assert!(
            !output
                .actions
                .iter()
                .any(|a| matches!(a, Action::Send(cmd) if cmd == "@lich drain"))
        );
    }

    #[test]
    fn consumed_life_force_sends_dig_grave() {
        let (output, _, _) = run_trigger(
            "The consumed life force fills your being with ecstacy!",
            None,
            None,
        );
        assert!(
            output
                .actions
                .iter()
                .any(|a| matches!(a, Action::Send(cmd) if cmd == "@dig grave"))
        );
    }

    #[test]
    fn raise_hands_chant_highlights_spell_vocal() {
        let text = "You raise your hands, gaze up and chant 'Avee Avee Aveallis'";
        let (_output, styled, _) = run_trigger(text, None, None);
        let vocal_start = text.find("Avee Avee Aveallis").expect("vocal in line");
        let idx = text
            .get(..vocal_start)
            .map(|s| s.graphemes(true).count())
            .unwrap_or(0);

        for styled_char in &styled.styled_chars[idx..idx + "Avee Avee Aveallis".len()] {
            assert_eq!(styled_char.color, AnsiCode::White);
            assert!(styled_char.bold);
        }
        assert!(!styled.styled_chars[0].bold);
    }

    #[test]
    fn article_hits_you_highlights_only_matching_full_line() {
        let (_output, styled, _) = run_trigger("An orc hits you.", None, None);
        assert!(
            styled
                .styled_chars
                .iter()
                .all(|c| c.color == AnsiCode::Red && c.bold)
        );

        let (_output, non_match, _) = run_trigger("n orc hits you.", None, None);
        assert!(
            non_match
                .styled_chars
                .iter()
                .all(|c| c.color == AnsiCode::DefaultColor && !c.bold)
        );
    }

    #[test]
    fn other_hits_avatar_whole_line_magenta_and_twice_highlighted() {
        let text = "Orc hits Nynn twice as hard.";
        let (_output, styled, _automation) = run_trigger(text, None, Some("Nynn"));
        assert!(
            styled
                .styled_chars
                .iter()
                .all(|c| c.color == AnsiCode::Magenta)
        );
        assert!(styled.styled_chars.iter().any(|c| c.bold));
    }
}
