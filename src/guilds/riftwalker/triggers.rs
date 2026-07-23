use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::RiftwalkerGuild;
use crate::guilds::riftwalker::{
    AIR_SKILL, EARTH_SKILL, ENTITY_LABEL_AIR, ENTITY_LABEL_EARTH, ENTITY_LABEL_FIRE,
    ENTITY_LABEL_WATER, FIRE_SKILL, RIFTWALKER_ELEMENT_VAR, RIFTWALKER_HAS_ENTITY_FLAG,
    RIFTWALKER_SKILL_VAR, WATER_SKILL,
};
use crate::stats::StatsEffect;
use crate::triggers::{LineEffect, Trigger, TriggerEffects, TriggerFacts, TriggerLine};
use regex::Regex;
use std::sync::LazyLock;

/// Battle listen entity HP (TinyFugue `riftwalker.tf`); requires `battle listen` on the MUD.
/// Lines may end with extra fields and `  =--`; optional three bracket segments after `HP:n(max)`.
static RIFTWALKER_BATTLE_HP: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)^--=\s+(.+?)\s+HP:([0-9]+)\(([^)]+)\)(?:\s+(\[[^\]]*\]))?(?:\s+(\[[^\]]*\]))?(?:\s+(\[[^\]]*\]))?").unwrap()
});
static RIFTWALKER_BATTLE_LABEL: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?i)^--=\s+(.+?)\s+=--\s*$").unwrap());

impl RiftwalkerGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::primary_trigger]
    }

    pub fn primary_trigger(line: &TriggerLine<'_>, facts: &TriggerFacts) -> TriggerEffects {
        let mut output = TriggerEffects::default();
        battle_listen_entity_status(facts, line.plain_line, &mut output);
        if output.original.gag {
            return output;
        }

        let line = line.plain_line;
        clears_and_keeps(facts, line, &mut output);
        line_sync_entity_skill(facts, line, &mut output);
        skill_state_echoes(facts, line, &mut output);

        elemental_line_paint(facts, line, &mut output);
        summon_entity_line_paint(facts, line, &mut output);
        elemental_tokens_paint(facts, &mut output);
        aura_paint(facts, line, &mut output);
        misc_paint(facts, line, &mut output);
        entity_sense_paint(&mut output);

        output
    }
}

fn automation_label(facts: &TriggerFacts, key: &str) -> String {
    facts
        .get_var(key)
        .cloned()
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "entity".to_string())
}

/// BatMUD often prints the literal word `entity` in `A/An … <element> <noun> <aura> with power [yours]`
/// even when the player uses a custom label in other messages. Accept either configured noun or `entity`.
fn status_line_noun_regex_chunk(configured: &str) -> String {
    let trimmed = configured.trim();
    let esc = regex::escape(trimmed);
    if trimmed.eq_ignore_ascii_case("entity") {
        esc
    } else {
        format!("(?:{esc}|entity)")
    }
}

fn aura_noun_alternation(facts: &TriggerFacts) -> String {
    let base = noun_alt_pattern(facts);
    if base.is_empty() {
        "entity".to_string()
    } else {
        format!("{base}|entity")
    }
}

/// Sorted, deduped, regex-escaped alternation of per-element nouns.
fn noun_alt_pattern(facts: &TriggerFacts) -> String {
    let mut parts = vec![
        automation_label(facts, ENTITY_LABEL_FIRE),
        automation_label(facts, ENTITY_LABEL_AIR),
        automation_label(facts, ENTITY_LABEL_WATER),
        automation_label(facts, ENTITY_LABEL_EARTH),
    ];
    parts.sort();
    parts.dedup();
    parts
        .into_iter()
        .map(|s| regex::escape(&s))
        .collect::<Vec<_>>()
        .join("|")
}

fn battle_listen_entity_status(
    facts: &TriggerFacts,
    plain_line: &str,
    output: &mut TriggerEffects,
) {
    let text = plain_line.trim_end_matches('\r');
    if let Some(caps) = RIFTWALKER_BATTLE_HP.captures(text) {
        if facts.flag_is_set(RIFTWALKER_HAS_ENTITY_FLAG) {
            if let Some(label_seg) = caps.get(1) {
                let lbl = label_seg.as_str().trim();
                if !lbl.is_empty() {
                    output
                        .stats
                        .push(StatsEffect::MergeRiftwalkerBattleLabel(lbl.to_string()));
                }
            }
            let hp = caps
                .get(2)
                .map(|segment| segment.as_str().parse::<i32>().unwrap_or(0))
                .unwrap_or(0);
            let paren_inside = caps.get(3).map(|g| g.as_str()).unwrap_or("");
            let b1 = caps.get(4).map(|g| g.as_str());
            let b2 = caps.get(5).map(|g| g.as_str());
            let b3 = caps.get(6).map(|g| g.as_str());
            output
                .stats
                .push(StatsEffect::MergeRiftwalkerBattleHpFromListen {
                    hp,
                    paren_inside: paren_inside.to_string(),
                    brackets: [
                        b1.map(str::to_string),
                        b2.map(str::to_string),
                        b3.map(str::to_string),
                    ],
                });
            push_entity_hp_notices(hp, output);
        }
        output.original.gag = true;
        return;
    }
    if !facts.flag_is_set(RIFTWALKER_HAS_ENTITY_FLAG) {
        return;
    }
    if let Some(caps) = RIFTWALKER_BATTLE_LABEL.captures(text) {
        let label = caps
            .get(1)
            .map(|segment| segment.as_str().to_string())
            .unwrap_or_default();
        output
            .stats
            .push(StatsEffect::MergeRiftwalkerBattleLabel(label));
        output.original.gag = true;
    }
}

fn push_entity_hp_notices(hp: i32, output: &mut TriggerEffects) {
    let (message, style) = if hp < 100 {
        (
            "*********** !!! ENTITY UNDER 100 HP !!! ***********",
            TextStyle::BRIGHT_RED,
        )
    } else if hp < 150 {
        (
            "____________ENTITY UNDER 150hp______________",
            TextStyle::BRIGHT_MAGENTA,
        )
    } else if hp < 200 {
        ("ENTITY UNDER 200hp!!", TextStyle::MAGENTA)
    } else if hp < 250 {
        ("Entity under 250hp!!", TextStyle::BRIGHT_YELLOW)
    } else {
        return;
    };
    let mut notice = StyledLine::new(message);
    notice.set_line_style(style);
    output.lines.push(notice);
}

fn clears_and_keeps(facts: &TriggerFacts, line: &str, output: &mut TriggerEffects) {
    let n_alt = noun_alt_pattern(facts);
    let lost_entity = format!(
        r"(?i)^Your\s+(?:{n_alt})\s+begins to warp, seeming to become unstable\. It folds in on itself and vanishes!$"
    );
    let lost_soul = r"(?i)^Your soul cries out in anguish as your faithful companion is slain!$";
    if Regex::new(&lost_entity).is_ok_and(|re| re.is_match(line.trim()))
        || Regex::new(lost_soul).is_ok_and(|re| re.is_match(line.trim()))
    {
        output.stats.push(StatsEffect::ClearRiftwalkerEntityStatus);
        output.actions.push(Action::SetFlag(
            RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
            false,
        ));
    }

    if line.contains("crumpled piece of paper flies through the air") {
        output.actions.push(Action::Send("@keep paper".to_string()));
    }

    if line.starts_with("A dazzling spark races along the stream of green light between you") {
        output
            .actions
            .push(Action::Send("@keep add all spark,collection".to_string()));
    }
}

fn line_sync_entity_skill(facts: &TriggerFacts, line: &str, output: &mut TriggerEffects) {
    let trimmed = line.trim();
    let mapping = [
        ("fire", FIRE_SKILL, "fire", ENTITY_LABEL_FIRE),
        ("air", AIR_SKILL, "air", ENTITY_LABEL_AIR),
        ("water", WATER_SKILL, "water", ENTITY_LABEL_WATER),
        ("earth", EARTH_SKILL, "earth", ENTITY_LABEL_EARTH),
    ];

    for &(glyph, mastery, bucket, key) in &mapping {
        let noun = regex::escape(&automation_label(facts, key));
        let Ok(re) = Regex::new(&format!(
            r"(?i)^.+\s+{glyph}\s+{noun}\s+.+with power \[yours\]$"
        )) else {
            continue;
        };
        if re.is_match(trimmed) {
            entity_skill_actions(mastery, bucket)
                .into_iter()
                .for_each(|action| output.actions.push(action));
            return;
        }
    }
}

fn entity_skill_actions(skill: &str, element: &str) -> Vec<Action> {
    vec![
        Action::SetVar(RIFTWALKER_SKILL_VAR.to_string(), skill.to_string()),
        Action::SetVar(RIFTWALKER_ELEMENT_VAR.to_string(), element.to_string()),
        Action::SetFlag(RIFTWALKER_HAS_ENTITY_FLAG.to_string(), true),
    ]
}

fn is_prepared_line(facts: &TriggerFacts, line: &str) -> bool {
    let n_alt = noun_alt_pattern(facts);
    let Ok(re) = Regex::new(&format!(
        r"(?i)^Your\s+(?:{n_alt})\s+is prepared to do the skill\.?$"
    )) else {
        return false;
    };
    re.is_match(line.trim())
}

fn is_concentration_lost_line(facts: &TriggerFacts, line: &str) -> bool {
    let n_alt = noun_alt_pattern(facts);
    let Ok(re) = Regex::new(&format!(
        r"(?i)^Your\s+(?:{n_alt})\s+loses its concentration and cannot do the skill\.$"
    )) else {
        return false;
    };
    re.is_match(line.trim())
}

fn skill_state_echoes(facts: &TriggerFacts, line: &str, output: &mut TriggerEffects) {
    if is_prepared_line(facts, line) {
        let mut banner = StyledLine::new("!!!!!!!!!! Entity Skill !!!!!!!!!!");
        banner.set_line_style(TextStyle::BRIGHT_BLUE);
        output.lines.push(banner);
        return;
    }

    if is_concentration_lost_line(facts, line) {
        if current_skill_equals(facts, AIR_SKILL) {
            output
                .lines
                .push(down_notice("SUFFOCATING EMBRACE IS DOWN!"));
        } else if current_skill_equals(facts, EARTH_SKILL) {
            output.lines.push(down_notice("EARTHEN COVER IS DOWN!"));
        } else if current_skill_equals(facts, WATER_SKILL) {
            output
                .lines
                .push(down_notice("SUBJUGATING BACKWASH IS DOWN!"));
        }
        return;
    }

    let air_lab = automation_label(facts, ENTITY_LABEL_AIR);
    if matches_line_insensitive(
        line,
        format!("Your air {air_lab} falters and its wispy tendrils fall to its sides."),
    ) && current_skill_equals(facts, AIR_SKILL)
    {
        output
            .lines
            .push(down_notice("SUFFOCATING EMBRACE IS DOWN!"));
        return;
    }

    let earth_lab = automation_label(facts, ENTITY_LABEL_EARTH);
    if matches_line_insensitive(
        line,
        format!("Your earth {earth_lab} hunches down looking much less solid than a second ago."),
    ) && current_skill_equals(facts, EARTH_SKILL)
    {
        output.lines.push(down_notice("EARTHEN COVER IS DOWN!"));
        return;
    }

    let water_lab = automation_label(facts, ENTITY_LABEL_WATER);
    if matches_line_insensitive(
        line,
        format!("Your water {water_lab} stops glowing and its skin becomes still."),
    ) && current_skill_equals(facts, WATER_SKILL)
    {
        output
            .lines
            .push(down_notice("SUBJUGATING BACKWASH IS DOWN!"));
    }
}

fn matches_line_insensitive(got: &str, expected_ascii: String) -> bool {
    got.eq_ignore_ascii_case(expected_ascii.as_str())
}

fn current_skill_equals(facts: &TriggerFacts, needle: &str) -> bool {
    facts
        .get_var(RIFTWALKER_SKILL_VAR)
        .map(|skill| skill == needle)
        .unwrap_or(false)
}

fn down_notice(message: &'static str) -> StyledLine {
    let mut line = StyledLine::new(message);
    line.set_line_style(TextStyle::BRIGHT_RED);
    line
}

fn elemental_line_paint(facts: &TriggerFacts, text: &str, output: &mut TriggerEffects) {
    let f = regex::escape(&automation_label(facts, ENTITY_LABEL_FIRE));
    let a = regex::escape(&automation_label(facts, ENTITY_LABEL_AIR));
    let w = regex::escape(&automation_label(facts, ENTITY_LABEL_WATER));
    let e = regex::escape(&automation_label(facts, ENTITY_LABEL_EARTH));

    let Ok(hit) = Regex::new(&format!(
        r"^(?:Fire\s+{f}|Air\s+{a}|Water\s+{w}|Earth\s+{e})\s+hits .+ (once|twice|thrice) .+\.$"
    )) else {
        return;
    };
    let Ok(stun) = Regex::new(&format!(
        r"^(?:Fire\s+{f}|Air\s+{a}|Water\s+{w}|Earth\s+{e}) is stunned\.$"
    )) else {
        return;
    };

    let n_alt = noun_alt_pattern(facts);
    let Ok(miss) = Regex::new(&format!(
        r"^Your (.+)\s+(?:{n_alt})\s+does some strange combat maneuver but doesn't hit anything\.$"
    )) else {
        return;
    };

    if hit.is_match(text) {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::GREEN));
    } else if stun.is_match(text) {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::RED));
    } else if miss.is_match(text) {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_RED));
    }
}

fn summon_entity_line_paint(facts: &TriggerFacts, text: &str, output: &mut TriggerEffects) {
    let rows = [
        ("air", ENTITY_LABEL_AIR, TextStyle::BRIGHT_CYAN),
        ("fire", ENTITY_LABEL_FIRE, TextStyle::RED),
        ("water", ENTITY_LABEL_WATER, TextStyle::BLUE),
        ("earth", ENTITY_LABEL_EARTH, TextStyle::YELLOW),
    ];
    for &(elem, key, style) in &rows {
        let n = status_line_noun_regex_chunk(&automation_label(facts, key));
        let Ok(re) = Regex::new(&format!(
            r"(?i)^(?:A|An)\s+(.+)\s+{elem}\s+{n}\s+(.+?)\s+with power \[yours\]$"
        )) else {
            continue;
        };
        if re.is_match(text) {
            output.original.edits.push(LineEffect::StyleLine(style));
            return;
        }
    }
}

fn elemental_tokens_paint(facts: &TriggerFacts, output: &mut TriggerEffects) {
    let pairs = [
        ("Fire entity", ENTITY_LABEL_FIRE, TextStyle::RED),
        ("Air entity", ENTITY_LABEL_AIR, TextStyle::BRIGHT_CYAN),
        ("Water entity", ENTITY_LABEL_WATER, TextStyle::BLUE),
        ("Earth entity", ENTITY_LABEL_EARTH, TextStyle::YELLOW),
    ];
    for (stock_slice, configuration_key, style) in pairs {
        let customized = stock_slice.replace(
            "entity",
            automation_label(facts, configuration_key).as_str(),
        );
        for fragment in [stock_slice, &customized] {
            output.original.edits.push(LineEffect::StyleBlock {
                text: fragment.to_string(),
                style,
            });
            let lower = fragment.to_ascii_lowercase();
            if lower != *fragment {
                output
                    .original
                    .edits
                    .push(LineEffect::StyleBlock { text: lower, style });
            }
        }
    }
}

const AURA_WORDS: &[&str] = &[
    "glowing",
    "shimmering",
    "gleaming",
    "sizzling",
    "sparkling",
    "glittering",
    "radiating",
    "throbbing",
    "pulsating",
    "blazing",
];

fn aura_style_for(word: &str) -> Option<TextStyle> {
    let table: &[(&str, TextStyle)] = &[
        ("glowing", TextStyle::BLUE),
        ("shimmering", TextStyle::BRIGHT_BLUE),
        ("gleaming", TextStyle::CYAN),
        ("sizzling", TextStyle::BRIGHT_CYAN),
        ("sparkling", TextStyle::YELLOW),
        ("glittering", TextStyle::BRIGHT_YELLOW),
        ("radiating", TextStyle::MAGENTA),
        ("throbbing", TextStyle::BRIGHT_MAGENTA),
        ("pulsating", TextStyle::RED),
        ("blazing", TextStyle::BRIGHT_RED),
    ];
    for &(name, style) in table {
        if name.eq_ignore_ascii_case(word) {
            return Some(style);
        }
    }
    None
}

fn aura_paint(facts: &TriggerFacts, text: &str, output: &mut TriggerEffects) {
    let n_alt = aura_noun_alternation(facts);
    let aura_alt = AURA_WORDS.join("|");
    let Ok(re) = Regex::new(&format!(
        r"(?i)^(?:A|An)\s+(?P<pre>.+)\s+(?P<nn>(?:{n_alt}))\s+(?P<aur>{aura_alt})\s+with power \[yours\]$"
    )) else {
        return;
    };
    let Some(caps) = re.captures(text) else {
        return;
    };
    let Some(aur_m) = caps.name("aur") else {
        return;
    };
    let Some(style) = aura_style_for(aur_m.as_str()) else {
        return;
    };
    output.original.edits.push(LineEffect::StylePlainByteRange {
        range: aur_m.range(),
        style,
    });
}

fn misc_paint(facts: &TriggerFacts, text: &str, output: &mut TriggerEffects) {
    let a = regex::escape(&automation_label(facts, ENTITY_LABEL_AIR));
    if let Ok(air_embrace) = Regex::new(&format!(
        r"(?i)^Air\s+{a}\s+embraces .+ with its wispy tendrils\.$"
    )) && air_embrace.is_match(text)
    {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_BLUE));
        return;
    }

    let n_alt = noun_alt_pattern(facts);
    if let Ok(wave) = Regex::new(&format!(
        r"(?i)^A wave of blue light bursts forth from your\s+(?:{n_alt})\s+and hits you in the chest\.$"
    )) && wave.is_match(text)
    {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_BLUE));
        return;
    }

    let w = regex::escape(&automation_label(facts, ENTITY_LABEL_WATER));
    if let Ok(water_glow) = Regex::new(&format!(r"(?i)^Water\s+{w}\s+starts to glow,.+shore\.$"))
        && water_glow.is_match(text)
    {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_BLUE));
        return;
    }

    if let Ok(skill_focus) = Regex::new(&format!(
        r"(?i)^.+\s+(?:{n_alt})\s+starts concentrating on a new offensive skill\.$"
    )) && skill_focus.is_match(text)
    {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_WHITE));
        return;
    }

    if is_prepared_line(facts, text) {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::BRIGHT_BLUE));
        return;
    }
    if text.contains("crumpled piece of paper flies through the air") {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::GREEN));
        return;
    }
    if let Ok(regain) = Regex::new(&format!(
        r"(?i)^You manage to regain control of your\s+(?:{n_alt})\s+before the connection is completely broken!$"
    )) && regain.is_match(text)
    {
        output
            .original
            .edits
            .push(LineEffect::StyleLine(TextStyle::GREEN));
    }
}

fn entity_sense_paint(output: &mut TriggerEffects) {
    output.original.edits.push(LineEffect::StyleBlock {
        text: "Entity sense:".to_string(),
        style: TextStyle::BRIGHT_BLUE,
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;
    use crate::triggers::{TriggerFacts, TriggerLine};
    use ratatui::text::Line;
    use unicode_segmentation::UnicodeSegmentation;

    fn line_plain(line: &Line<'_>) -> String {
        line.spans
            .iter()
            .map(|span| span.content.as_ref())
            .collect()
    }

    fn facts(automation: &Automation) -> TriggerFacts {
        TriggerFacts::new(
            automation.snapshot_flags(),
            automation.snapshot_vars(),
            None,
            None,
        )
    }

    fn run(
        line_text: &str,
        automation: &Automation,
        stats: &mut Stats,
    ) -> (TriggerEffects, StyledLine) {
        let output =
            RiftwalkerGuild::primary_trigger(&TriggerLine::new(line_text), &facts(automation));
        for effect in output.stats.clone() {
            stats.apply_effect(effect);
        }
        let mut line = StyledLine::new(line_text);
        output.apply_line_effects_to(&mut line);
        (output, line)
    }

    #[test]
    fn aura_only_on_summon_line_not_random_sparkling() {
        let mut stats = Stats::default();
        let auto = Automation::new();
        let (_, noise) = run("The river is sparkling in the sun.", &auto, &mut stats);
        let beta = noise.plain_line.find("sparkling").unwrap();
        assert_eq!(
            noise.styled_chars[noise.plain_line[..beta].graphemes(true).count()].color,
            crate::ansi::TextColor::Default
        );

        let (_, summon) = run(
            "A huge fire entity sparkling with power [yours]",
            &auto,
            &mut stats,
        );
        let sp = summon.plain_line.find("sparkling").unwrap();
        let idx = summon.plain_line[..sp].graphemes(true).count();
        assert_eq!(summon.styled_chars[idx].color, TextStyle::YELLOW.color);
    }

    #[test]
    fn aura_respects_custom_entity_noun() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_var(ENTITY_LABEL_FIRE, "golem".to_string());
        let (_, line) = run(
            "A huge fire golem gleaming with power [yours]",
            &auto,
            &mut stats,
        );
        let g = line.plain_line.find("gleaming").unwrap();
        let idx = line.plain_line[..g].graphemes(true).count();
        assert_eq!(line.styled_chars[idx].color, TextStyle::CYAN.color);
    }

    #[test]
    fn skill_sync_requires_element_and_configured_noun() {
        let mut auto = Automation::new();
        auto.set_var(ENTITY_LABEL_FIRE, "golem".to_string());
        let mut out = TriggerEffects::default();
        let facts = facts(&auto);
        line_sync_entity_skill(
            &facts,
            "Some prefix fire entity with power [yours]",
            &mut out,
        );
        assert!(out.actions.is_empty());

        line_sync_entity_skill(
            &facts,
            "Some prefix fire golem trailing with power [yours]",
            &mut out,
        );
        assert!(!out.actions.is_empty());
    }

    #[test]
    fn battle_listen_hp_updates_stats_gags_and_warns_when_flag_set() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, true);
        let (out, line) = run("--=  fire thing  HP:95(30%) more", &auto, &mut stats);
        assert!(line.gag);
        assert!(stats.has_riftwalker_entity_status());
        assert!(line_plain(&stats.render_riftwalker_entity_inline()).contains("HP:95(30%)"));
        assert_eq!(
            out.lines[0].plain_line,
            "*********** !!! ENTITY UNDER 100 HP !!! ***********"
        );
    }

    #[test]
    fn battle_listen_hp_gags_even_without_has_entity_flag() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, false);
        let (_, line) = run("--=  fire thing  HP:200(50%) more", &auto, &mut stats);
        assert!(line.gag);
        assert!(!stats.has_riftwalker_entity_status());
    }

    #[test]
    fn battle_listen_combined_line_with_trailing_status_is_gagged() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, true);
        let (_, line) = run(
            "--=  Fire entity  HP:511(629) [+28] [] []  =--",
            &auto,
            &mut stats,
        );
        assert!(line.gag);
        assert_eq!(
            line_plain(&stats.render_riftwalker_entity_inline()),
            "Fire entity  HP:511(629) [+28] [] []"
        );
    }

    #[test]
    fn battle_listen_second_and_third_bracket_slots_use_mud_text() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, true);
        let (_, line) = run(
            "--=  Water entity  HP:100(100) [+1] [foo] [bar]  =--",
            &auto,
            &mut stats,
        );
        assert!(line.gag);
        assert_eq!(
            line_plain(&stats.render_riftwalker_entity_inline()),
            "Water entity  HP:100(100) [+1] [foo] [bar]",
        );
    }

    #[test]
    fn battle_listen_label_merges_and_gags() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, true);
        let (out, line) = run("--=  My pet wisp  =--", &auto, &mut stats);
        assert!(line.gag);
        assert!(line_plain(&stats.render_riftwalker_entity_inline()).contains("My pet wisp"));
        assert!(out.lines.is_empty());
    }

    #[test]
    fn entity_death_clears_battle_listen_status() {
        let mut stats = Stats::default();
        stats.merge_riftwalker_battle_hp(400);
        let mut auto = Automation::new();
        auto.set_flag(RIFTWALKER_HAS_ENTITY_FLAG, true);
        let line = "Your entity begins to warp, seeming to become unstable. It folds in on itself and vanishes!";
        let _ = run(line, &auto, &mut stats);
        assert!(!stats.has_riftwalker_entity_status());
    }
}
