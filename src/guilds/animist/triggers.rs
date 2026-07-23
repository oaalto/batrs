use crate::ansi::TextStyle;
use crate::automation::Action;
use crate::guilds::AnimistGuild;
use crate::secondary_status::SecondaryStatusEffect;
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

impl AnimistGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![
            Self::soul_companion_status_trigger,
            Self::spirit_appears_trigger,
            Self::soul_companion_training_trigger,
            Self::soul_companion_sword_hit_trigger,
        ]
    }

    pub fn soul_companion_status_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        let plain = line.plain_line.trim_end_matches('\r').trim();
        if let Some(captures) = SOUL_COMPANION_STATUS.captures(plain) {
            let percent = captures[2].parse::<i32>().unwrap_or_default();
            let description = captures[3].trim().to_string();
            return TriggerEffects::none().gag().secondary_status(
                SecondaryStatusEffect::SetSoulCompanion {
                    percent: percent.clamp(0, 100),
                    description,
                },
            );
        }

        TriggerEffects::none()
    }

    pub fn spirit_appears_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        if SPIRIT_APPEARS.is_match(line.plain_line) {
            output
                .actions
                .push(Action::Send("@lead my spirit".to_string()));
        }
        output
    }

    pub fn soul_companion_training_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        if line.plain_line == "You feel slightly better at fighting with your soul companion." {
            return TriggerEffects::none().style_line(TextStyle::BLUE);
        }
        TriggerEffects::none()
    }

    pub fn soul_companion_sword_hit_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        if SOUL_COMPANION_SWORD_HIT.is_match(line.plain_line) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_BLUE);
        }
        TriggerEffects::none()
    }
}

/// Percent may use non-ASCII digits; trailing status text (e.g. `+`) is optional.
static SOUL_COMPANION_STATUS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^Your\s+soul\s+companion\s*:\s*(.+?)\s+\((\d+)%\)\s*(.*?)\s*$").unwrap()
});
static SPIRIT_APPEARS: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^(.+) spirit slowly appears, answering your call\.$").unwrap());
static SOUL_COMPANION_SWORD_HIT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"^(.+)'s soul companion swings his sword in (.+) arc, and hits.*$").unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::secondary_status::SecondaryStatus;
    use crate::triggers::{Trigger, TriggerFacts, TriggerLine};

    fn run(
        trigger: Trigger,
        line_text: &str,
        status: &mut SecondaryStatus,
    ) -> (TriggerEffects, StyledLine) {
        let output = trigger(&TriggerLine::new(line_text), &TriggerFacts::default());
        for effect in output.secondary_status.clone() {
            status.apply_effect(effect);
        }
        let mut line = StyledLine::new(line_text);
        output.apply_line_effects_to(&mut line);
        (output, line)
    }

    fn stats_line_text(status: &SecondaryStatus) -> String {
        status
            .render_soul_inline()
            .spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    #[test]
    fn soul_companion_status_updates_secondary_status() {
        let mut status = SecondaryStatus::default();

        let (_, line) = run(
            AnimistGuild::soul_companion_status_trigger,
            "Your soul companion: exc (88%) guarding you",
            &mut status,
        );

        assert!(line.gag);
        assert_eq!(stats_line_text(&status), "Soul: 88% guarding you");
    }

    #[test]
    fn soul_companion_status_trims_carriage_return() {
        let mut status = SecondaryStatus::default();

        let (_, line) = run(
            AnimistGuild::soul_companion_status_trigger,
            "Your soul companion: exc (88%) +\r",
            &mut status,
        );

        assert!(line.gag);
        assert_eq!(stats_line_text(&status), "Soul: 88% +");
    }

    #[test]
    fn soul_companion_status_optional_suffix() {
        let mut status = SecondaryStatus::default();

        let (_, line) = run(
            AnimistGuild::soul_companion_status_trigger,
            "your soul companion: wolf (50%)",
            &mut status,
        );

        assert!(line.gag);
        assert_eq!(stats_line_text(&status), "Soul: 50% ");
    }

    #[test]
    fn spirit_appearing_leads_it() {
        let mut status = SecondaryStatus::default();
        let (output, _) = run(
            AnimistGuild::spirit_appears_trigger,
            "A wolf spirit slowly appears, answering your call.",
            &mut status,
        );

        assert!(matches!(
            &output.actions[0],
            Action::Send(command) if command == "@lead my spirit"
        ));
    }

    #[test]
    fn soul_companion_training_is_blue() {
        let mut status = SecondaryStatus::default();
        let (_, line) = run(
            AnimistGuild::soul_companion_training_trigger,
            "You feel slightly better at fighting with your soul companion.",
            &mut status,
        );

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
    }

    #[test]
    fn soul_companion_sword_hit_is_light_blue() {
        let mut status = SecondaryStatus::default();
        let (_, line) = run(
            AnimistGuild::soul_companion_sword_hit_trigger,
            "Oaalto's soul companion swings his sword in wide arc, and hits Orc.",
            &mut status,
        );

        assert_eq!(line.styled_chars[0].color, AnsiCode::Blue);
        assert!(line.styled_chars[0].bold);
    }
}
