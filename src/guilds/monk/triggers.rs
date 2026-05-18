use crate::ansi::TextStyle;
use crate::automation::Action;
use crate::guilds::MonkGuild;
use crate::guilds::monk::commands::reset_current_skill_actions;
use crate::guilds::monk::{
    AREA_SKILL_1, AREA_SKILL_2, AREA_SKILL_3, ARMOUR_SKILL_1, ARMOUR_SKILL_2, ARMOUR_SKILL_3,
    AVOID_SKILL_1, AVOID_SKILL_2, AVOID_SKILL_3, CURRENT_AREA_SKILL_VAR, CURRENT_ARMOUR_SKILL_VAR,
    CURRENT_AVOID_SKILL_VAR, CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1, DISRUPT_SKILL_2,
    DISRUPT_SKILL_3, DOING_MEDITATION_FLAG, KATA_DONE_FLAG,
};
use crate::guilds::sects_triggers;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

struct MonkRule {
    pattern: Regex,
    color: Option<TextStyle>,
    set_var: Option<(&'static str, &'static str)>,
}

impl MonkGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::state_trigger,
            Self::skill_result_trigger,
            sects_triggers::sect_cultivation_hilite_trigger,
        ]
    }

    pub fn state_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        let line = line.plain_line;

        if KATA_DONE.iter().any(|regex| regex.is_match(line)) {
            output
                .actions
                .push(Action::SetFlag(KATA_DONE_FLAG.to_string(), true));
            output.actions.push(Action::IfFlag {
                flag: DOING_MEDITATION_FLAG.to_string(),
                actions: vec![Action::Send(crate::abilities::client_send_line(
                    "use 'meditation'",
                ))],
            });
        }

        if line == "You start concentrating on the skill." {
            output
                .actions
                .push(Action::SetFlag(KATA_DONE_FLAG.to_string(), false));
        }

        if line == "You sit down and start meditating." {
            output
                .actions
                .push(Action::SetFlag(DOING_MEDITATION_FLAG.to_string(), false));
        }

        if INTERRUPTS.iter().any(|regex| regex.is_match(line)) {
            output.actions.extend(reset_current_skill_actions());
        }

        output
    }

    pub fn skill_result_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        let line = line.plain_line.to_string();

        for rule in MONK_RULES
            .iter()
            .filter(|rule| rule.pattern.is_match(&line))
        {
            if let Some(style) = rule.color {
                output = output.style_line(style);
            }
            if let Some((key, value)) = rule.set_var {
                output
                    .actions
                    .push(Action::SetVar(key.to_string(), value.to_string()));
            }
        }

        output
    }
}

lazy_static! {
    static ref KATA_DONE: Vec<Regex> = vec![
        Regex::new(r"^You perform the kata\.$").unwrap(),
        Regex::new(r"^You perform the peaceful (.+) kata\.$").unwrap(),
    ];
    static ref INTERRUPTS: Vec<Regex> = vec![
        Regex::new(r"^You are not in combat right now\.$").unwrap(),
        Regex::new(r"^Your movement prevents you from doing the skill\.$").unwrap(),
        Regex::new(r"^GgrTF:  ---- SKILL STOPPED ----$").unwrap(),
        Regex::new(r"^You lose your concentration and cannot do the skill\.$").unwrap(),
        Regex::new(r"^You break your skill attempt\.$").unwrap(),
        Regex::new(
            r"^You stop concentrating on the skill and begin searching for a proper place to rest\.$"
        )
        .unwrap(),
        Regex::new(r"^You start chanting\.$").unwrap(),
    ];
    static ref MONK_RULES: Vec<MonkRule> = vec![
        rule(r"You do a complex attack maneuver but miss\.", Some(TextStyle::BRIGHT_RED), None),
        rule(r"You fail to reach the state of inner harmony\.", Some(TextStyle::RED), None),
        rule(r"Your training is starting to pay off!", Some(TextStyle::BLUE), None),
        rule(
            r"You feel like you have mastered the art of (.+)\. It might be time to find another advanced technique\.",
            Some(TextStyle::BLUE),
            None,
        ),
        rule(
            r"Because you mastered the skill before, it comes back to you much faster this time\.",
            Some(TextStyle::BLUE),
            None,
        ),
        rule(
            r"You feel like imaginary food is done digesting\.",
            Some(TextStyle::RED),
            None,
        ),
        rule(
            r"^The (blow|thrashing) knocks some of (its|her|his) defenses loose,\s+leaving (it|him|her) temporarily vulnerable!$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"^As (she|he|it) lands, some of (his|her|its) protection shifts out of place, leaving (him|her|it) temporarily vulnerable!$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"but only score a glancing blow\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"but only bruise the muscle\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"scoring a solid hit!$",
            Some(TextStyle::BLUE),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something pop!$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something snap!$",
            Some(TextStyle::GREEN),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something shatter!$",
            Some(TextStyle::MAGENTA),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"but don't get any solid hits\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"preventing you from hitting with the others\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"getting two hits in!$",
            Some(TextStyle::BLUE),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"shaking (his|her|its) whole body!$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"outstretched limbs, but miss\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"(manage|manages|managed) to land on (his|her|its) butt\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"but (he|she|it) twists to (landon|land on) (his|her|its) side\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"and throw (him|it|her) down onto (his|her|its) back!$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"shakes (his|her|its) head back and forth, clearly disoriented\.$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"blinks distractedly, looking somewhat blind!$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"hacks and wheezes, looking disoriented\.$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"takes a moment too long to regain (his|her|its) footing\.$",
            Some(TextStyle::GREEN),
            None,
        ),
        rule(
            r"but can't make flesh contact\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"on the side of the head\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"a harsh slap across the jaw\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"but miss the veins you were aiming for\.$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"hitting one of the arteries and disrupting (his|her|its) blood flow!$",
            Some(TextStyle::GREEN),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"hitting both arteries and temporarily halting (his|her|its) blood to the brain!$",
            Some(TextStyle::BRIGHT_GREEN),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"but slip and fall down\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"You jump up and kick (.+) in the ribcage, but don't get enough contact to backflip\.",
            Some(TextStyle::CYAN),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"and have to settle for a dropkick to the stomach\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"You land a single kick in the middle of (.+)'s chest, backflip, and land on your feet\.",
            Some(TextStyle::YELLOW),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"but (he|she|it) deflects your hands\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"but barely manage to move (her|it|him) at all\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"forcing (him|her|it) to take a few steps back\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"backs off and you can't even get started\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^Most of your attacks are partially deflected or blocked\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"shoulders and sides, but nothing deadly\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"^You get some hits to the belly, getting some penetration\.$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r#"getting your fingers between the ribs like you"d hoped\.$"#,
            Some(TextStyle::GREEN),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"your knuckles between the ribs!$",
            Some(TextStyle::MAGENTA),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"^You send (.+) crashing into (.+)!$",
            Some(TextStyle::BRIGHT_MAGENTA),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"blocks it and knocks you to the ground\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^Your kick is true, but not forceful enough to knock anyone around\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"The impact is less than satisfying\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"^You kick it stumbling backwards!$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"^You knock (.+) into (.+)!$",
            Some(TextStyle::GREEN),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"braces and blocks it\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^You drop down and sweep your leg low along the ground\.$",
            Some(TextStyle::MAGENTA),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"but are pushed back\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"but can't get a decent claw in\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"but can't push hard enough to get into a flip\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"clawing (.+) in the back with curved fingers!$",
            Some(TextStyle::YELLOW),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"leaving you flat on your back!$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"and you end up merely slamming your back against (.+)\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_3)),
        ),
        rule(
            r"over the shoulder with the heel of your foot\.$",
            Some(TextStyle::BLUE),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_3)),
        ),
        rule(
            r"but fall short and land on your side\.$",
            Some(TextStyle::BRIGHT_RED),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"and end up merely kicking (.+) in the face with one foot\.$",
            Some(TextStyle::CYAN),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
    ];
}

fn rule(
    pattern: &str,
    color: Option<TextStyle>,
    set_var: Option<(&'static str, &'static str)>,
) -> MonkRule {
    MonkRule {
        pattern: Regex::new(pattern).unwrap(),
        color,
        set_var,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::automation::Automation;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn facts(automation: &Automation) -> TriggerFacts {
        TriggerFacts::new(
            automation.snapshot_flags(),
            automation.snapshot_vars(),
            None,
            None,
        )
    }

    fn run_skill(line_text: &str) -> (TriggerEffects, StyledLine) {
        let output =
            MonkGuild::skill_result_trigger(&TriggerLine::new(line_text), &TriggerFacts::default());
        let mut line = StyledLine::new(line_text);
        output.apply_line_effects_to(&mut line);
        (output, line)
    }

    #[test]
    fn kata_done_sends_meditation_when_requested() {
        let mut automation = Automation::new();
        automation.set_flag(DOING_MEDITATION_FLAG, true);

        let output = MonkGuild::state_trigger(
            &TriggerLine::new("You perform the kata."),
            &facts(&automation),
        );
        let sends = automation.apply_actions(output.actions);

        assert_eq!(sends, vec!["@use 'meditation'"]);
        assert!(automation.flag_is_set(KATA_DONE_FLAG));
    }

    #[test]
    fn interrupt_resets_current_skills() {
        let output = MonkGuild::state_trigger(
            &TriggerLine::new("You break your skill attempt."),
            &TriggerFacts::default(),
        );

        assert_eq!(output.actions.len(), 4);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_ARMOUR_SKILL_VAR && value == ARMOUR_SKILL_1
        ));
    }

    #[test]
    fn armour_result_colors_and_updates_current_skill() {
        let (output, line) = run_skill("You kick hard, scoring a solid hit!");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_ARMOUR_SKILL_VAR && value == ARMOUR_SKILL_2
        ));
    }

    #[test]
    fn disrupt_result_colors_and_updates_current_skill() {
        let (output, line) = run_skill(
            "You jump up and kick foe in the ribcage, but don't get enough contact to backflip.",
        );

        assert_eq!(line.styled_chars[0].color, AnsiCode::Cyan);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_DISRUPT_SKILL_VAR && value == DISRUPT_SKILL_3
        ));
    }

    #[test]
    fn area_result_colors_and_updates_current_skill() {
        let (output, line) = run_skill("You knock orc into troll!");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_AREA_SKILL_VAR && value == AREA_SKILL_3
        ));
    }

    #[test]
    fn avoid_result_colors_and_updates_current_skill() {
        let (output, line) =
            run_skill("You claw hard, clawing goblin in the back with curved fingers!");

        assert_eq!(line.styled_chars[0].color, AnsiCode::Yellow);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_AVOID_SKILL_VAR && value == AVOID_SKILL_2
        ));
    }
}
