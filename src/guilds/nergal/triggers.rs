use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::NergalGuild;
use crate::secondary_status::{NergalMinion, NergalResourceStatus, SecondaryStatusEffect};
use crate::triggers::{Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

impl NergalGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::nergal_trigger]
    }

    pub fn nergal_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        let line = line.plain_line.trim_end_matches('\r').trim();

        if let Some(captures) = MINION_STATUS.captures(line) {
            let minion = NergalMinion {
                name: captures[1].to_string(),
                hp: captures[2].parse().unwrap_or(0),
                max_hp: captures[3].parse().unwrap_or(0),
                sp: captures[4].parse().unwrap_or(0),
                max_sp: captures[5].parse().unwrap_or(0),
                ep: captures[6].parse().unwrap_or(0),
                max_ep: captures[7].parse().unwrap_or(0),
            };
            return output
                .gag()
                .secondary_status(SecondaryStatusEffect::UpsertNergalMinion(minion));
        }

        if let Some(captures) = RESOURCE_STATUS.captures(line) {
            let status = NergalResourceStatus {
                vitae: captures[1].parse().unwrap_or(0),
                max_vitae: captures[2].parse().unwrap_or(0),
                potentia: captures[3].parse().unwrap_or(0),
                max_potentia: captures[4].parse().unwrap_or(0),
                evolution_points: captures[5].parse().unwrap_or(0),
            };
            return output
                .gag()
                .secondary_status(SecondaryStatusEffect::SetNergalResourceStatus(status));
        }

        if unsummon_clears_minions(line) {
            return output.secondary_status(SecondaryStatusEffect::ClearNergalMinions);
        }

        if line.contains("DEAD, R.I.P.") {
            output.actions.push(Action::Send("@nergal sc".to_string()));
            return output;
        }

        if line.contains("Potentia: 1000/1000") {
            output
                .lines
                .push(echo_notice("***** POTENTIA IS FULL! *****", true));
            return output;
        }

        if line == "Your body can't handle any more of potentia!" {
            output
                .lines
                .push(echo_notice("***** POTENTIA IS FULL! *****", false));
            return output.style_line(TextStyle::RED);
        }

        if line.contains("Vitae: 1000/1000") {
            output
                .lines
                .push(echo_notice("***** VITAE IS FULL! *****", true));
            return output;
        }

        if line == "Your body can't handle any more of vitae!" {
            output
                .lines
                .push(echo_notice("***** VITAE IS FULL! *****", false));
            return output.style_line(TextStyle::RED);
        }

        if line.contains("looks a lot less in pain as colonies start to disappear")
            || HARVEST_VITAE.is_match(line)
            || HARVEST_POTENTIA.is_match(line)
            || line.contains("You feel your insight of evolution expanding")
        {
            output = output.style_line(TextStyle::CYAN);
        } else if line
            .contains("You hear deep inside your head the parasite whispers more secrets of")
            || line.contains(
                "You hear deep inside your head the parasite whispering to you secrets of",
            )
        {
            output = output.style_line(TextStyle::GREEN);
        } else if line.contains("looks relieved as the aether line fades away") {
            output = output.style_line(TextStyle::BLUE);
        } else if AURA_SCRATCH.is_match(line)
            || AURA_PLUNGES.is_match(line)
            || AURA_ESSENCE.is_match(line)
        {
            output = output.style_line(TextStyle::GREEN);
        }

        output
    }
}

fn unsummon_clears_minions(line: &str) -> bool {
    UNSUMMON_CONNECTION.is_match(line)
        || UNSUMMON_END.is_match(line)
        || UNSUMMON_ORDER_DORMANT.is_match(line)
        || UNSUMMON_RELEASE_HOST.is_match(line)
}

fn echo_notice(text: &str, green: bool) -> StyledLine {
    let mut line = StyledLine::new(text);
    line.set_line_style(if green {
        TextStyle::BRIGHT_GREEN
    } else {
        TextStyle::BRIGHT_RED
    });
    line
}

static MINION_STATUS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^::\.\.:\. (.+) \[Hp: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*, Sp: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*, Ep: (-?[0-9]+) \(([0-9]+)\)[ \-+()0-9]*\]$",
    )
    .unwrap()
});
static RESOURCE_STATUS: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^::\.\.:\. \[Vitae: ([0-9]+)/([0-9]+)  Potentia: ([0-9]+)/([0-9]+), Evolution points: ([0-9]+)\]$",
    )
    .unwrap()
});
static UNSUMMON_CONNECTION: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"Your connection to your parasite is severed completely\. .+ jerks violently couple of times and collapses",
    )
    .unwrap()
});
static UNSUMMON_END: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"You end the connection to your parasite, making the host jerk couple of times violently\. After couple of seconds .+ collapses and stops moving at all",
    )
    .unwrap()
});
static UNSUMMON_ORDER_DORMANT: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"You order the parasite to return .+ and lay dormant there until you have use for it again\.",
    )
    .unwrap()
});
static UNSUMMON_RELEASE_HOST: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"You 'release' the host from the parasites influence\. The host jerks violently couple of times",
    )
    .unwrap()
});
static HARVEST_VITAE: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You feel you harvest (.+) amount of vitae\.\.").unwrap());
static HARVEST_POTENTIA: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"You feel you harvest (.+) amount of potentia\.\.").unwrap());
static AURA_SCRATCH: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(.*) manages to scratch (.*) skin infecting the tissue under the skin with nasty disease",
    )
    .unwrap()
});
static AURA_PLUNGES: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"(.*) plunges towards (.*) and manages to sink its disease infecting nails into (.*) flesh!",
    )
    .unwrap()
});
static AURA_ESSENCE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"You can feel essence flowing into you from (.*) as (.*) sinks its nails into its victim!",
    )
    .unwrap()
});

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::secondary_status::SecondaryStatus;
    use crate::triggers::{TriggerFacts, TriggerLine};

    fn run(line_text: &str, status: &mut SecondaryStatus) -> (TriggerEffects, StyledLine) {
        let output =
            NergalGuild::nergal_trigger(&TriggerLine::new(line_text), &TriggerFacts::default());
        for effect in output.secondary_status.clone() {
            status.apply_effect(effect);
        }
        let mut styled = StyledLine::new(line_text);
        output.apply_line_effects_to(&mut styled);
        (output, styled)
    }

    #[test]
    fn minion_status_gags_and_upserts() {
        let mut status = SecondaryStatus::default();
        let line = "::..:. Tick [Hp: 44 (55) (+3), Sp: 1 (1), Ep: 200 (200)]";

        let (out, styled) = run(line, &mut status);

        assert!(styled.gag);
        assert!(out.actions.is_empty());
        assert!(out.lines.is_empty());
        let first = status.nergal_minions()[0].as_ref().unwrap();
        assert_eq!(first.name, "Tick");
        assert_eq!(first.hp, 44);
        assert_eq!(first.max_hp, 55);
        assert_eq!(first.sp, 1);
        assert_eq!(first.max_sp, 1);
        assert_eq!(first.ep, 200);
        assert_eq!(first.max_ep, 200);
    }

    #[test]
    fn resource_status_gags_and_updates_secondary_status() {
        let mut status = SecondaryStatus::default();
        let line = "::..:. [Vitae: 22/1000  Potentia: 752/1000, Evolution points: 0]";

        let (out, styled) = run(line, &mut status);

        assert!(styled.gag);
        assert!(out.actions.is_empty());
        assert!(out.lines.is_empty());
        assert_eq!(
            out.secondary_status,
            vec![SecondaryStatusEffect::SetNergalResourceStatus(
                NergalResourceStatus {
                    vitae: 22,
                    max_vitae: 1000,
                    potentia: 752,
                    max_potentia: 1000,
                    evolution_points: 0,
                }
            )]
        );
        assert!(status.has_nergal_resource_status());
    }

    #[test]
    fn resource_status_requires_strict_field_order() {
        let mut status = SecondaryStatus::default();
        let line = "::..:. [Potentia: 752/1000 Vitae: 22/1000, Evolution points: 0]";

        let (out, styled) = run(line, &mut status);

        assert!(!styled.gag);
        assert!(out.secondary_status.is_empty());
        assert!(!status.has_nergal_resource_status());
    }

    #[test]
    fn unsummon_connection_clears_minions() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(NergalMinion {
            name: "a".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        let line = "Your connection to your parasite is severed completely. Host jerks violently couple of times and collapses.";

        let _ = run(line, &mut status);

        assert!(!status.has_nergal_minions());
    }

    #[test]
    fn unsummon_end_clears_minions() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(NergalMinion {
            name: "Tick".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        let line = "You end the connection to your parasite, making the host jerk couple of times violently. After couple of seconds Tick collapses and stops moving at all.";

        let _ = run(line, &mut status);

        assert!(!status.has_nergal_minions());
    }

    #[test]
    fn unsummon_order_dormant_clears_minions() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(NergalMinion {
            name: "Balrog".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        let line = "You order the parasite to return the corrupted lands of blue flamed Tree and lay dormant there until you have use for it again.";

        let _ = run(line, &mut status);

        assert!(!status.has_nergal_minions());
    }

    #[test]
    fn unsummon_release_host_clears_minions() {
        let mut status = SecondaryStatus::default();
        status.upsert_nergal_minion(NergalMinion {
            name: "Weeping pixie".into(),
            hp: 1,
            max_hp: 1,
            sp: 1,
            max_sp: 1,
            ep: 1,
            max_ep: 1,
        });
        let line = "More thoughts infiltrate your mind. As you are evaluating your minions, one of them seems sub optimal for the servitude of the lord Nergal. You 'release' the host from the parasites influence. The host jerks violently couple of times as if regaining its free will but without the parasite the host is too weak to survive and collapses.";

        let _ = run(line, &mut status);

        assert!(!status.has_nergal_minions());
    }

    #[test]
    fn death_sends_nergal_score_alias() {
        let mut status = SecondaryStatus::default();

        let (out, _) = run("DEAD, R.I.P.", &mut status);

        assert_eq!(out.actions.len(), 1);
        assert!(matches!(&out.actions[0], Action::Send(command) if command == "@nergal sc"));
    }

    #[test]
    fn potentia_full_echoes_green() {
        let mut status = SecondaryStatus::default();

        let (out, _) = run("Potentia: 1000/1000", &mut status);

        assert_eq!(out.lines.len(), 1);
        assert_eq!(out.lines[0].plain_line, "***** POTENTIA IS FULL! *****");
        assert_eq!(out.lines[0].styled_chars[0].color, AnsiCode::Green);
    }

    #[test]
    fn scratch_aura_line_is_green() {
        let mut status = SecondaryStatus::default();
        let line = "Foo manages to scratch Bar skin infecting the tissue under the skin with nasty disease";

        let (_, styled) = run(line, &mut status);

        assert_eq!(styled.styled_chars[0].color, AnsiCode::Green);
    }
}
