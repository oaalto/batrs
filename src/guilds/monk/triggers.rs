use crate::ansi::{AnsiCode, StyledLine};
use crate::automation::Action;
use crate::guilds::MonkGuild;
use crate::guilds::monk::commands::reset_current_skill_actions;
use crate::guilds::monk::{
    AREA_SKILL_1, AREA_SKILL_2, AREA_SKILL_3, ARMOUR_SKILL_1, ARMOUR_SKILL_2, ARMOUR_SKILL_3,
    AVOID_SKILL_1, AVOID_SKILL_2, AVOID_SKILL_3, CURRENT_AREA_SKILL_VAR, CURRENT_ARMOUR_SKILL_VAR,
    CURRENT_AVOID_SKILL_VAR, CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1, DISRUPT_SKILL_2,
    DISRUPT_SKILL_3, DOING_MEDITATION_FLAG, KATA_DONE_FLAG,
};
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

struct MonkRule {
    pattern: Regex,
    color: Option<(AnsiCode, bool)>,
    set_var: Option<(&'static str, &'static str)>,
}

impl MonkGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::state_trigger, Self::skill_result_trigger]
    }

    pub fn state_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = &styled_line.plain_line;

        if KATA_DONE.iter().any(|regex| regex.is_match(line)) {
            output
                .actions
                .push(Action::SetFlag(KATA_DONE_FLAG.to_string(), true));
            output.actions.push(Action::IfFlag {
                flag: DOING_MEDITATION_FLAG.to_string(),
                actions: vec![Action::Send("@use meditation".to_string())],
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

    pub fn skill_result_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line.plain_line.clone();

        for rule in MONK_RULES
            .iter()
            .filter(|rule| rule.pattern.is_match(&line))
        {
            if let Some((color, bold)) = rule.color {
                styled_line.set_line_color(color, bold);
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
        rule(r"You do a complex attack maneuver but miss\.", Some((AnsiCode::Red, true)), None),
        rule(r"You fail to reach the state of inner harmony\.", Some((AnsiCode::Red, false)), None),
        rule(r"Your training is starting to pay off!", Some((AnsiCode::Blue, false)), None),
        rule(
            r"You feel like you have mastered the art of (.+)\. It might be time to find another advanced technique\.",
            Some((AnsiCode::Blue, false)),
            None,
        ),
        rule(
            r"Because you mastered the skill before, it comes back to you much faster this time\.",
            Some((AnsiCode::Blue, false)),
            None,
        ),
        rule(
            r"You feel like imaginary food is done digesting\.",
            Some((AnsiCode::Red, false)),
            None,
        ),
        rule(
            r"^The (blow|thrashing) knocks some of (its|her|his) defenses loose,\s+leaving (it|him|her) temporarily vulnerable!$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"^As (she|he|it) lands, some of (his|her|its) protection shifts out of place, leaving (him|her|it) temporarily vulnerable!$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"but only score a glancing blow\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"but only bruise the muscle\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"scoring a solid hit!$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something pop!$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something snap!$",
            Some((AnsiCode::Green, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"and you feel something shatter!$",
            Some((AnsiCode::Magenta, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_2)),
        ),
        rule(
            r"but don't get any solid hits\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"preventing you from hitting with the others\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"getting two hits in!$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"shaking (his|her|its) whole body!$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_3)),
        ),
        rule(
            r"outstretched limbs, but miss\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"(manage|manages|managed) to land on (his|her|its) butt\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"but (he|she|it) twists to (landon|land on) (his|her|its) side\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"and throw (him|it|her) down onto (his|her|its) back!$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1)),
        ),
        rule(
            r"shakes (his|her|its) head back and forth, clearly disoriented\.$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"blinks distractedly, looking somewhat blind!$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"hacks and wheezes, looking disoriented\.$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"takes a moment too long to regain (his|her|its) footing\.$",
            Some((AnsiCode::Green, false)),
            None,
        ),
        rule(
            r"but can't make flesh contact\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"on the side of the head\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"a harsh slap across the jaw\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"but miss the veins you were aiming for\.$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"hitting one of the arteries and disrupting (his|her|its) blood flow!$",
            Some((AnsiCode::Green, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"hitting both arteries and temporarily halting (his|her|its) blood to the brain!$",
            Some((AnsiCode::Green, true)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_2)),
        ),
        rule(
            r"but slip and fall down\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"You jump up and kick (.+) in the ribcage, but don't get enough contact to backflip\.",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"and have to settle for a dropkick to the stomach\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"You land a single kick in the middle of (.+)'s chest, backflip, and land on your feet\.",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_3)),
        ),
        rule(
            r"but (he|she|it) deflects your hands\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"but barely manage to move (her|it|him) at all\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"forcing (him|her|it) to take a few steps back\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1)),
        ),
        rule(
            r"backs off and you can't even get started\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^Most of your attacks are partially deflected or blocked\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"shoulders and sides, but nothing deadly\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"^You get some hits to the belly, getting some penetration\.$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r#"getting your fingers between the ribs like you"d hoped\.$"#,
            Some((AnsiCode::Green, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"your knuckles between the ribs!$",
            Some((AnsiCode::Magenta, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"^You send (.+) crashing into (.+)!$",
            Some((AnsiCode::Magenta, true)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_2)),
        ),
        rule(
            r"blocks it and knocks you to the ground\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^Your kick is true, but not forceful enough to knock anyone around\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"The impact is less than satisfying\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"^You kick it stumbling backwards!$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"^You knock (.+) into (.+)!$",
            Some((AnsiCode::Green, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_3)),
        ),
        rule(
            r"braces and blocks it\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"^You drop down and sweep your leg low along the ground\.$",
            Some((AnsiCode::Magenta, false)),
            Some((CURRENT_AREA_SKILL_VAR, AREA_SKILL_1)),
        ),
        rule(
            r"but are pushed back\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"but can't get a decent claw in\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"but can't push hard enough to get into a flip\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"clawing (.+) in the back with curved fingers!$",
            Some((AnsiCode::Yellow, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_2)),
        ),
        rule(
            r"leaving you flat on your back!$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"and you end up merely slamming your back against (.+)\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_3)),
        ),
        rule(
            r"over the shoulder with the heel of your foot\.$",
            Some((AnsiCode::Blue, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_3)),
        ),
        rule(
            r"but fall short and land on your side\.$",
            Some((AnsiCode::Red, true)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
        rule(
            r"and end up merely kicking (.+) in the face with one foot\.$",
            Some((AnsiCode::Cyan, false)),
            Some((CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1)),
        ),
    ];
}

fn rule(
    pattern: &str,
    color: Option<(AnsiCode, bool)>,
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
    use crate::automation::Automation;
    use crate::stats::Stats;

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    #[test]
    fn kata_done_sends_meditation_when_requested() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(DOING_MEDITATION_FLAG, true);
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You perform the kata.");

        let output = MonkGuild::state_trigger(&mut ctx, &mut line);
        let sends = ctx.automation.apply_actions(output.actions);

        assert_eq!(sends, vec!["@use meditation"]);
        assert!(ctx.automation.flag_is_set(KATA_DONE_FLAG));
    }

    #[test]
    fn interrupt_resets_current_skills() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You break your skill attempt.");

        let output = MonkGuild::state_trigger(&mut ctx, &mut line);

        assert_eq!(output.actions.len(), 4);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_ARMOUR_SKILL_VAR && value == ARMOUR_SKILL_1
        ));
    }

    #[test]
    fn armour_result_colors_and_updates_current_skill() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You kick hard, scoring a solid hit!");

        let output = MonkGuild::skill_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_ARMOUR_SKILL_VAR && value == ARMOUR_SKILL_2
        ));
    }

    #[test]
    fn disrupt_result_colors_and_updates_current_skill() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new(
            "You jump up and kick foe in the ribcage, but don't get enough contact to backflip.",
        );

        let output = MonkGuild::skill_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Cyan);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_DISRUPT_SKILL_VAR && value == DISRUPT_SKILL_3
        ));
    }

    #[test]
    fn area_result_colors_and_updates_current_skill() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You knock orc into troll!");

        let output = MonkGuild::skill_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Green);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_AREA_SKILL_VAR && value == AREA_SKILL_3
        ));
    }

    #[test]
    fn avoid_result_colors_and_updates_current_skill() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line =
            StyledLine::new("You claw hard, clawing goblin in the back with curved fingers!");

        let output = MonkGuild::skill_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Yellow);
        assert!(matches!(
            &output.actions[0],
            Action::SetVar(key, value) if key == CURRENT_AVOID_SKILL_VAR && value == AVOID_SKILL_2
        ));
    }
}
