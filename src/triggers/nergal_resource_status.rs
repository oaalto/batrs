use crate::stats::{NergalResourceStatus, StatsEffect};
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
use lazy_static::lazy_static;
use regex::Regex;

pub(crate) fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    let line = line.plain_line.trim_end_matches('\r').trim();
    let Some(captures) = RESOURCE_STATUS.captures(line) else {
        return TriggerEffects::default();
    };

    let status = NergalResourceStatus {
        vitae: captures[1].parse().unwrap_or(0),
        max_vitae: captures[2].parse().unwrap_or(0),
        potentia: captures[3].parse().unwrap_or(0),
        max_potentia: captures[4].parse().unwrap_or(0),
        evolution_points: captures[5].parse().unwrap_or(0),
    };

    TriggerEffects::default()
        .gag()
        .stat(StatsEffect::SetNergalResourceStatus(status))
}

lazy_static! {
    static ref RESOURCE_STATUS: Regex = Regex::new(
        r"^::\.\.:\. \[Vitae: ([0-9]+)/([0-9]+)  Potentia: ([0-9]+)/([0-9]+), Evolution points: ([0-9]+)\]$",
    )
    .unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_status_gags_and_updates_stats() {
        let line =
            TriggerLine::new("::..:. [Vitae: 22/1000  Potentia: 752/1000, Evolution points: 0]");

        let out = trigger(&line, &TriggerFacts::default());

        assert!(out.original.gag);
        assert_eq!(
            out.stats,
            vec![StatsEffect::SetNergalResourceStatus(NergalResourceStatus {
                vitae: 22,
                max_vitae: 1000,
                potentia: 752,
                max_potentia: 1000,
                evolution_points: 0,
            })]
        );
    }

    #[test]
    fn resource_status_requires_strict_field_order() {
        let line =
            TriggerLine::new("::..:. [Potentia: 752/1000 Vitae: 22/1000, Evolution points: 0]");

        let out = trigger(&line, &TriggerFacts::default());

        assert!(!out.original.gag);
        assert!(out.stats.is_empty());
    }
}
