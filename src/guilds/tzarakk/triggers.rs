use crate::ansi::TextStyle;
use crate::automation::Action;
use crate::guilds::TzarakkGuild;
use crate::guilds::tzarakk::{DISMOUNTED_FLAG, MOUNT_SUMMONED_FLAG, TZARAKK_MOUNT_VAR};
use crate::stats::StatsEffect;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

static MOUNT_DETECTION_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^'(Vedir|Orthos)', .+ \[Rider: You\]$").unwrap());
static ROUND_REGEX: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^\*+ Round .+ \*+$").unwrap());
static CHAOSFEED_REPLENISH_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^A faint fog-like substance flows from corpse of (.+) to (.+)'s lifeless eyes replenishing it (.+)\.$"
    )
    .unwrap()
});
static DISMOUNT_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"The ice makes a sound below your mount, scaring it!").unwrap(),
        Regex::new(r"You are knocked off your mount!").unwrap(),
        Regex::new(r"Your mount throws you!").unwrap(),
    ]
});
static MOUNT_APPEARS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(.+) appears in a violent burst of chaos\.").unwrap());
static MOUNT_DEATH_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+) is DEAD, R\.I\.P\.$").unwrap());
static RIDING_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^You get up on (.+) and begin to ride\.$").unwrap());
static BANISH_MOUNT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You pray for Tzarakk to receive his mount\.").unwrap());
static CHARGE_MISS_REGEXES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new(r"You fail to hit your foe with (.+)").unwrap(),
        Regex::new(r"You charge towards your enemy, but alas -- a clean miss\.").unwrap(),
        Regex::new(r"Your mount snorts and does not respond\.").unwrap(),
        Regex::new(r"Your mount is too confused to comply\.").unwrap(),
    ]
});
static CHARGE_HIT_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You manage to hit your foe with (.+) as you pass by\.").unwrap());
static STEED_SUMMONED_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"A bizarre mist starts to form itself rapidly, and within moments a dark morbid")
        .unwrap()
});
static MOUNT_STATUS_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(Orthos|Vedir) is (.+) \((\d+)%\)\.?$").unwrap());

impl TzarakkGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::mount_detection_trigger,
            Self::round_trigger,
            Self::chaosfeed_replenish_trigger,
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
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if MOUNT_DETECTION_REGEX.is_match(line.plain_line) {
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true));
        }
        output
    }

    pub fn round_trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if ROUND_REGEX.is_match(line.plain_line) && facts.flag_is_set(MOUNT_SUMMONED_FLAG) {
            let mount = facts
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            output.actions.push(Action::Send(format!("@x {}", mount)));
        }
        output
    }

    pub fn chaosfeed_replenish_trigger(
        line: &TriggerLine<'_>,
        facts: &TriggerFacts,
    ) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if let Some(captures) = CHAOSFEED_REPLENISH_REGEX.captures(line.plain_line) {
            let replenished_mount = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let tzarakk_mount = facts
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());

            if replenished_mount == tzarakk_mount {
                output.actions.push(Action::Send(format!(
                    "@tzarakk chaosfeed corpse;x {}",
                    tzarakk_mount
                )));
            }
        }
        output
    }

    pub fn dismount_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if DISMOUNT_REGEXES.iter().any(|r| r.is_match(line.plain_line)) {
            output = output.style_line(TextStyle::BRIGHT_RED);
            output
                .actions
                .push(Action::SetFlag(DISMOUNTED_FLAG.to_string(), true));
        }
        output
    }

    pub fn mount_appears_trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if let Some(captures) = MOUNT_APPEARS_REGEX.captures(line.plain_line) {
            let mount_name = captures.get(1).map(|m| m.as_str()).unwrap_or("");
            let tzarakk_mount = facts
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

    pub fn mount_death_trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if facts.flag_is_set(DISMOUNTED_FLAG) && MOUNT_DEATH_REGEX.is_match(line.plain_line) {
            output = output.style_line(TextStyle::BRIGHT_RED);
            let mount = facts
                .get_var(TZARAKK_MOUNT_VAR)
                .cloned()
                .unwrap_or_else(|| "Vedir".to_string());
            output
                .actions
                .push(Action::Send(format!("@mount {}", mount)));
        }
        output
    }

    pub fn riding_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if RIDING_REGEX.is_match(line.plain_line) {
            output
                .actions
                .push(Action::SetFlag(DISMOUNTED_FLAG.to_string(), false));
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true));
        }
        output
    }

    pub fn banish_mount_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if BANISH_MOUNT_REGEX.is_match(line.plain_line) {
            output.stats.push(StatsEffect::ClearTzarakkMountStatus);
            output
                .actions
                .push(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), false));
            output.actions.push(Action::Send(
                "@rip_action set get all from corpse;dig grave;drop zinc;drop mowgles".to_string(),
            ));
        }
        output
    }

    pub fn charge_result_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if CHARGE_MISS_REGEXES
            .iter()
            .any(|r| r.is_match(line.plain_line))
        {
            TriggerEffects::none().style_line(TextStyle::RED)
        } else if CHARGE_HIT_REGEX.is_match(line.plain_line) {
            TriggerEffects::none().style_line(TextStyle::BLUE)
        } else {
            TriggerEffects::none()
        }
    }

    pub fn steed_summoned_trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if STEED_SUMMONED_REGEX.is_match(line.plain_line) {
            let mount = facts
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
                "@rip_action set get all from corpse;tzarakk chaosfeed corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()
            ));
        }
        output
    }

    pub fn mount_status_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let plain = line.plain_line.trim_end_matches('\r').trim();
        if let Some(captures) = MOUNT_STATUS_REGEX.captures(plain) {
            let name = captures[1].to_string();
            let description = captures[2].trim().to_string();
            let percent = captures[3].parse::<i32>().unwrap_or_default();
            return TriggerEffects::none()
                .gag()
                .stat(StatsEffect::SetTzarakkMountStatus {
                    name,
                    percent: percent.clamp(0, 100),
                    description,
                });
        }
        TriggerEffects::none()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::automation::Automation;
    use crate::stats::Stats;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn facts(automation: &Automation) -> TriggerFacts {
        TriggerFacts::new(
            automation.snapshot_flags(),
            automation.snapshot_vars(),
            None,
            None,
        )
    }

    fn run(
        trigger: Trigger,
        line_text: &str,
        automation: &Automation,
        stats: &mut Stats,
    ) -> (TriggerEffects, StyledLine) {
        let output = trigger(&TriggerLine::new(line_text), &facts(automation));
        for effect in output.stats.clone() {
            stats.apply_effect(effect);
        }
        let mut line = StyledLine::new(line_text);
        output.apply_line_effects_to(&mut line);
        (output, line)
    }

    fn tzarakk_mount_line_text(stats: &Stats) -> String {
        stats
            .render_tzarakk_mount_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn mount_detection_sets_flag() {
        let mut stats = Stats::default();
        let automation = Automation::new();

        let (output, _) = run(
            TzarakkGuild::mount_detection_trigger,
            "'Vedir', the black steed [Rider: You]",
            &automation,
            &mut stats,
        );

        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == MOUNT_SUMMONED_FLAG
        ));
    }

    #[test]
    fn mount_detection_orthos_also_works() {
        let mut stats = Stats::default();
        let automation = Automation::new();

        let (output, _) = run(
            TzarakkGuild::mount_detection_trigger,
            "'Orthos', the black steed [Rider: You]",
            &automation,
            &mut stats,
        );

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

        let (output, _) = run(
            TzarakkGuild::round_trigger,
            "*** Round 1 ***",
            &automation,
            &mut stats,
        );

        assert!(!output.actions.is_empty(), "Expected actions but got none");
        assert!(output.actions.iter().any(|a| matches!(
            a, Action::Send(cmd) if cmd == "@x Vedir"
        )));
    }

    #[test]
    fn round_trigger_does_nothing_when_not_mounted() {
        let mut stats = Stats::default();
        let automation = Automation::new();
        // mount_summoned is false by default

        let (output, _) = run(
            TzarakkGuild::round_trigger,
            "*** Round 1 ***",
            &automation,
            &mut stats,
        );

        assert!(output.actions.is_empty());
    }

    #[test]
    fn chaosfeed_replenish_feeds_and_mount_checks_tracked_mount() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let (output, _) = run(
            TzarakkGuild::chaosfeed_replenish_trigger,
            "A faint fog-like substance flows from corpse of Vedir to goblin's lifeless eyes replenishing it fully.",
            &automation,
            &mut stats,
        );

        assert!(matches!(
            &output.actions[0],
            Action::Send(cmd) if cmd == "@tzarakk chaosfeed corpse;x Vedir"
        ));
    }

    #[test]
    fn chaosfeed_replenish_ignores_other_corpses() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());
        let (output, _) = run(
            TzarakkGuild::chaosfeed_replenish_trigger,
            "A faint fog-like substance flows from corpse of Orthos to goblin's lifeless eyes replenishing it fully.",
            &automation,
            &mut stats,
        );

        assert!(output.actions.is_empty());
    }

    #[test]
    fn dismount_detects_ice_sound() {
        let mut stats = Stats::default();
        let automation = Automation::new();

        let (output, line) = run(
            TzarakkGuild::dismount_trigger,
            "The ice makes a sound below your mount, scaring it!",
            &automation,
            &mut stats,
        );

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(line.styled_chars[0].bold);
        assert!(matches!(
            &output.actions[0],
            Action::SetFlag(flag, true) if flag == DISMOUNTED_FLAG
        ));
    }

    #[test]
    fn mount_status_line_gagged_and_updates_stats() {
        let mut stats = Stats::default();
        let automation = Automation::new();

        let (_output, line) = run(
            TzarakkGuild::mount_status_trigger,
            "Vedir is in excellent shape (100%).",
            &automation,
            &mut stats,
        );

        assert!(line.gag);
        assert_eq!(tzarakk_mount_line_text(&stats), "Vedir: 100%");
    }

    #[test]
    fn mount_death_when_dismounted_sends_mount() {
        let mut stats = Stats::default();
        let mut automation = Automation::new();
        automation.set_flag(DISMOUNTED_FLAG, true);
        automation.set_var(TZARAKK_MOUNT_VAR, "Vedir".to_string());

        let (output, line) = run(
            TzarakkGuild::mount_death_trigger,
            "Vedir is DEAD, R.I.P.",
            &automation,
            &mut stats,
        );

        assert_eq!(line.styled_chars[0].color, AnsiCode::Red);
        assert!(matches!(
            &output.actions[0],
            Action::Send(cmd) if cmd == "@mount Vedir"
        ));
    }
}
