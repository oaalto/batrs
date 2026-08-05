use crate::automation::Action;
use crate::combat_awareness::NOT_IN_COMBAT_LINE;
use crate::triggers::player_combat_rules::player_combat_rules_arc;
use crate::triggers::rule_engine::{
    HiliteTarget, Rule, RuleAction, RuleCondition, RuleMatcher, apply_rules, push_rule, sort_rules,
    tf_echo, tf_hilite,
};
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::{Arc, LazyLock};

static RULES: LazyLock<Vec<Rule>> = LazyLock::new(|| {
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
        RuleMatcher::Simple("You feel in harmony with yourself, the universe and life in general."),
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
        RuleMatcher::Simple("You feel like you managed to channel additional POWER to your spell."),
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
        ("Your wings glow as they absorb more magic!", "BCyellow"),
        ("Whee, your neat fur is dry again!", "Cgreen"),
        ("You gain insight to warhorse philosophy!", "BCyellow"),
        (
            "You learn more about the praying mantis tactics!",
            "BCyellow",
        ),
        ("The water BURNS your skin.", "BCred"),
        ("You feel exhausted, being here in the dark.", "BCred"),
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

    sort_rules(&mut rules);
    rules
});

pub fn trigger_catalog() -> Vec<crate::command::TriggerCatalogEntry> {
    let mut entries = crate::triggers::rule_engine::rule_catalog_entries(&RULES);
    entries.push(crate::command::TriggerCatalogEntry::new(
        "^<player> hits ...",
        "Highlight player combat lines (pattern uses configured player name).",
    ));
    entries
}

pub fn trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
    let mut output = TriggerEffects::default();
    if let Some(rig) = facts.rig()
        && !rig.is_empty()
    {
        output
            .actions
            .push(Action::SetVar("rig".to_string(), rig.to_string()));
    }

    let player_combat_rules = facts
        .player_name()
        .map(player_combat_rules_arc)
        .unwrap_or_else(|| Arc::new(Vec::new()));

    apply_rules(
        RULES.iter().chain(player_combat_rules.iter()),
        line.plain_line,
        facts,
        &mut output,
    );
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::{AnsiCode, StyledLine};
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
            crate::guilds::MonkSkillsConfig::default(),
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
    fn money_summary_through_trigger_emits_summary_lines() {
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
        let (output, _, _) = run_trigger_with_setup(NOT_IN_COMBAT_LINE, None, None, |auto| {
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
        let (output, _, _) = run_trigger(NOT_IN_COMBAT_LINE, None, None);
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
    fn avatar_hits_other_highlights_once_in_blue() {
        let text = "Nynn hits orc once with force.";
        let (_output, styled, _) = run_trigger(text, None, Some("Nynn"));
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
        let (_output, styled, _) = run_trigger(text, None, Some("odefu"));
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
        let (_output, styled, _) = run_trigger(text, None, Some("odefu"));
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
    fn other_hits_avatar_whole_line_magenta_and_twice_highlighted() {
        let text = "Orc hits Nynn twice as hard.";
        let (_output, styled, _) = run_trigger(text, None, Some("Nynn"));
        assert!(
            styled
                .styled_chars
                .iter()
                .all(|c| c.color == AnsiCode::Magenta)
        );
        assert!(styled.styled_chars.iter().any(|c| c.bold));
    }
}
