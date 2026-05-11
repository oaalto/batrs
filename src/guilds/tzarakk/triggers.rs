use crate::ansi::{AnsiCode, StyledLine};
use crate::automation::Action;
use crate::guilds::TzarakkGuild;
use crate::guilds::tzarakk::{DISMOUNTED_FLAG, MOUNT_SUMMONED_FLAG, TZARAKK_MOUNT_VAR};
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
use regex::Regex;

#[cfg(test)]
use crate::automation::Automation;
#[cfg(test)]
use crate::stats::Stats;

lazy_static! {
    // Mount detection - when mount is already summoned and ridden
    static ref MOUNT_DETECTION_REGEX: Regex =
        Regex::new(r"^'(Vedir|Orthos)', .+ \[Rider: You\]$").unwrap();

    // Round tracking
    static ref ROUND_REGEX: Regex = Regex::new(r"^\*+ Round .+ \*+$").unwrap();

    // Dismount detection
    static ref DISMOUNT_REGEXES: Vec<Regex> = vec![
        Regex::new(r"The ice makes a sound below your mount, scaring it!").unwrap(),
        Regex::new(r"You are knocked off your mount!").unwrap(),
        Regex::new(r"Your mount throws you!").unwrap(),
    ];

    // Mount appears
    static ref MOUNT_APPEARS_REGEX: Regex =
        Regex::new(r"(.+) appears in a violent burst of chaos\.").unwrap();

    // Mount death
    static ref MOUNT_DEATH_REGEX: Regex = Regex::new(r"^(.+) is DEAD, R\.I\.P\.$").unwrap();

    // Riding
    static ref RIDING_REGEX: Regex =
        Regex::new(r"^You get up on (.+) and begin to ride\.$").unwrap();

    // Banish mount
    static ref BANISH_MOUNT_REGEX: Regex =
        Regex::new(r"You pray for Tzarakk to receive his mount\.").unwrap();

    // Charge hit/miss
    static ref CHARGE_MISS_REGEXES: Vec<Regex> = vec![
        Regex::new(r"You fail to hit your foe with (.+)").unwrap(),
        Regex::new(r"You charge towards your enemy, but alas -- a clean miss\.").unwrap(),
        Regex::new(r"Your mount snorts and does not respond\.").unwrap(),
        Regex::new(r"Your mount is too confused to comply\.").unwrap(),
    ];

    static ref CHARGE_HIT_REGEX: Regex =
        Regex::new(r"You manage to hit your foe with (.+) as you pass by\.").unwrap();

    // Steed summoned
    static ref STEED_SUMMONED_REGEX: Regex =
        Regex::new(r"A bizarre mist starts to form itself rapidly, and within moments a dark morbid").unwrap();

    // Mount status
    static ref MOUNT_STATUS_REGEX: Regex =
        Regex::new(r"^(Orthos|Vedir) is (.+) \((\d+)%\)").unwrap();
}

impl TzarakkGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::mount_detection_trigger,
            Self::round_trigger,
            Self::dismount_trigger,
            Self::mount_appears_trigger,
            Self::mount_death_trigger,
            Self::riding_trigger,
            Self::banish_mount_trigger,
            Self::charge_result_trigger,
            Self::steed_summoned_trigger,
            Self::mount_status_trigger,
        ]
    }

    pub fn mount_detection_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if MOUNT_DETECTION_REGEX.is_match(&styled_line.plain_line) {
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true));
        }
        output
    }

    pub fn round_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if ROUND_REGEX.is_match(&styled_line.plain_line)
            && ctx.automation.flag_is_set(MOUNT_SUMMONED_FLAG)
        {
            let mount = ctx
                .automation
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            output.actions.push(Action::Send(format!("@x {}", mount)));
        }
        output
    }

    pub fn dismount_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if DISMOUNT_REGEXES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, true);
            output
                .actions
                .push(Action::SetFlag(DISMOUNTED_FLAG.to_string(), true));
        }
        output
    }

    pub fn mount_appears_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if let Some(captures) = MOUNT_APPEARS_REGEX.captures(&styled_line.plain_line) {
            let mount_name = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let tzarakk_mount = ctx
                .automation
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            if mount_name == tzarakk_mount {
                output
                    .actions
                    .push(Action::Send(format!("@mount {}", mount_name)));
            }
        }
        output
    }

    pub fn mount_death_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if ctx.automation.flag_is_set(DISMOUNTED_FLAG)
            && MOUNT_DEATH_REGEX.is_match(&styled_line.plain_line)
        {
            styled_line.set_line_color(AnsiCode::Red, true);
            let mount = ctx
                .automation
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            output
                .actions
                .push(Action::Send(format!("@mount {}", mount)));
        }
        output
    }

    pub fn riding_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if RIDING_REGEX.is_match(&styled_line.plain_line) {
            output
                .actions
                .push(Action::SetFlag(DISMOUNTED_FLAG.to_string(), false));
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true));
        }
        output
    }

    pub fn banish_mount_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if BANISH_MOUNT_REGEX.is_match(&styled_line.plain_line) {
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), false));
            output.actions.push(Action::Send(
                "@rip_action set get all from corpse;dig grave;drop zinc;drop mowgles".to_string(),
            ));
        }
        output
    }

    pub fn charge_result_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if CHARGE_MISS_REGEXES
            .iter()
            .any(|r| r.is_match(&styled_line.plain_line))
        {
            styled_line.set_line_color(AnsiCode::Red, false);
        } else if CHARGE_HIT_REGEX.is_match(&styled_line.plain_line) {
            styled_line.set_line_color(AnsiCode::Blue, false);
        }
        TriggerOutput::default()
    }

    pub fn steed_summoned_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        if STEED_SUMMONED_REGEX.is_match(&styled_line.plain_line) {
            let mount = ctx
                .automation
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            output
                .actions
                .push(Action::Send(format!("@mount {}", mount)));
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true));
            // Auto-set feed mode after summoning
            output.actions.push(Action::Send(
                "@rip_action set get all from corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()
            ));
        }
        output
    }

    pub fn mount_status_trigger(
        _ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        if MOUNT_STATUS_REGEX.is_match(&styled_line.plain_line) {
            styled_line.gag = true;
        }
        TriggerOutput::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ctx<'a>(stats: &'a mut Stats, automation: &'a mut Automation) -> TriggerContext<'a> {
        TriggerContext {
            stats,
            automation,
            rig: None,
            player_name: None,
        }
    }

    #[test]
    fn mount_detection_sets_flag() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("'Vedir', the black steed [Rider: You]");

        let output = TzarakkGuild::mount_detection_trigger(&mut ctx, &mut line);

        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == MOUNT_SUMMONED_FLAG
        ));
    }

    #[test]
    fn mount_detection_orthos_also_works() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("'Orthos', the black steed [Rider: You]");

        let output = TzarakkGuild::mount_detection_trigger(&mut ctx, &mut line);

        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == MOUNT_SUMMONED_FLAG
        ));
    }

    #[test]
    fn round_trigger_sends_x_when_mounted() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(MOUNT_SUMMONED_FLAG, true);
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("*** Round 1 ***");

        let output = TzarakkGuild::round_trigger(&mut ctx, &mut line);

        assert!(!output.actions.is_empty(), "Expected actions but got none");
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::Send(cmd) if cmd == "@x Vedir"
        )));
    }

    #[test]
    fn round_trigger_does_nothing_when_not_mounted() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        // mount_summoned is false by default
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("*** Round 1 ***");

        let output = TzarakkGuild::round_trigger(&mut ctx, &mut line);

        assert!(output.actions.is_empty());
    }

    #[test]
    fn dismount_detects_ice_sound() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("The ice makes a sound below your mount, scaring it!");

        let output = TzarakkGuild::dismount_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == DISMOUNTED_FLAG
        ));
    }

    #[test]
    fn dismount_detects_knocked_off() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You are knocked off your mount!");

        let output = TzarakkGuild::dismount_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == DISMOUNTED_FLAG
        ));
    }

    #[test]
    fn dismount_detects_thrown() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Your mount throws you!");

        let output = TzarakkGuild::dismount_trigger(&mut ctx, &mut line);

        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == DISMOUNTED_FLAG
        ));
    }

    #[test]
    fn mount_appears_sends_mount_command() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Vedir appears in a violent burst of chaos.");

        let output = TzarakkGuild::mount_appears_trigger(&mut ctx, &mut line);

        assert!(matches!(
            &output.actions[0],
            Action::Send(cmd) if cmd == "@mount Vedir"
        ));
    }

    #[test]
    fn mount_appears_different_mount_ignored() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("SomeOtherMount appears in a violent burst of chaos.");

        let output = TzarakkGuild::mount_appears_trigger(&mut ctx, &mut line);

        assert!(output.actions.is_empty());
    }

    #[test]
    fn riding_clears_dismounted_and_sets_summoned() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(DISMOUNTED_FLAG, true);
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You get up on Vedir and begin to ride.");

        let output = TzarakkGuild::riding_trigger(&mut ctx, &mut line);

        assert!(output.actions.iter().any(|a| matches!(
            a, Action::SetFlag(flag, false) if flag == DISMOUNTED_FLAG
        )));
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::SetFlag(flag, true) if flag == MOUNT_SUMMONED_FLAG
        )));
    }

    #[test]
    fn banish_mount_clears_flag_and_sets_rip_action() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(MOUNT_SUMMONED_FLAG, true);
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You pray for Tzarakk to receive his mount.");

        let output = TzarakkGuild::banish_mount_trigger(&mut ctx, &mut line);

        assert!(output.actions.iter().any(|a| matches!(
            a, Action::SetFlag(flag, false) if flag == MOUNT_SUMMONED_FLAG
        )));
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::Send(cmd) if cmd.contains("dig grave")
        )));
    }

    #[test]
    fn charge_miss_colors_red() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("You charge towards your enemy, but alas -- a clean miss.");

        TzarakkGuild::charge_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
    }

    #[test]
    fn charge_hit_colors_blue() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line =
            StyledLine::new("You manage to hit your foe with a powerful strike as you pass by.");

        TzarakkGuild::charge_result_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
    }

    #[test]
    fn steed_summoned_mounts_and_sets_feed_mode() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new(
            "A bizarre mist starts to form itself rapidly, and within moments a dark morbid",
        );

        let output = TzarakkGuild::steed_summoned_trigger(&mut ctx, &mut line);

        assert!(output.actions.iter().any(|a| matches!(
            a, Action::Send(cmd) if cmd == "@mount Vedir"
        )));
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::SetFlag(flag, true) if flag == MOUNT_SUMMONED_FLAG
        )));
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::Send(cmd) if cmd.contains("chaosfeed")
        )));
    }

    #[test]
    fn mount_status_line_gagged() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Vedir is a black steed (100%)");

        TzarakkGuild::mount_status_trigger(&mut ctx, &mut line);

        assert!(line.gag);
    }

    #[test]
    fn mount_death_when_dismounted_removes_and_sends_mount() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(DISMOUNTED_FLAG, true);
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Vedir is DEAD, R.I.P.");

        let output = TzarakkGuild::mount_death_trigger(&mut ctx, &mut line);

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(matches!(
            &output.actions[0],
            Action::Send(cmd) if cmd == "@mount Vedir"
        ));
    }

    #[test]
    fn mount_death_when_not_dismounted_ignored() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        // dismounted is false by default
        let mut ctx = ctx(&mut stats, &mut automation);
        let mut line = StyledLine::new("Vedir is DEAD, R.I.P.");

        let output = TzarakkGuild::mount_death_trigger(&mut ctx, &mut line);

        assert!(output.actions.is_empty());
    }
}
