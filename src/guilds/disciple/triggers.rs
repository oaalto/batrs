use crate::ansi::{StyledLine, TextStyle};
use crate::guilds::DiscipleGuild;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

static SPAWN_GOING_DOWN: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new("You feel like the pulse of chaos inside you is slowing down!").unwrap()
});
static RED_HILITES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new("You feel exhausted, being here in the light.").unwrap(),
        Regex::new("You try to attack your enemy but fall over your own feet.").unwrap(),
    ]
});
static GREEN_HILITES: LazyLock<Vec<Regex>> = LazyLock::new(|| {
    vec![
        Regex::new("You feel the chaos pulse inside you!").unwrap(),
        Regex::new("Your (.+) tentacle strikes (.+).").unwrap(),
        Regex::new("You force yourself deeper into the chaos frenzy!").unwrap(),
    ]
});

impl DiscipleGuild {
    pub fn get_triggers(&self) -> Vec<crate::triggers::Trigger> {
        vec![
            Self::spawn_going_down_trigger,
            Self::red_hilites_trigger,
            Self::green_hilites_trigger,
        ]
    }

    pub fn spawn_going_down_trigger(
        line: &TriggerLine<'_>,
        _facts: &TriggerFacts,
    ) -> TriggerEffects {
        if SPAWN_GOING_DOWN.is_match(line.plain_line) {
            let mut alert = StyledLine::new("*************** SPAWN GOING DOWN!! ***************");
            alert.set_line_style(TextStyle::BRIGHT_RED);
            return TriggerEffects::none()
                .style_line(TextStyle::BRIGHT_RED)
                .emit(alert);
        }
        TriggerEffects::none()
    }

    pub fn red_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if RED_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::BRIGHT_RED);
        }
        TriggerEffects::none()
    }

    pub fn green_hilites_trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
        if GREEN_HILITES.iter().any(|r| r.is_match(line.plain_line)) {
            return TriggerEffects::none().style_line(TextStyle::GREEN);
        }
        TriggerEffects::none()
    }
}
