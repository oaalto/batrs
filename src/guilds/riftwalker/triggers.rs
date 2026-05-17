use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::automation::Automation;
use crate::guilds::RiftwalkerGuild;
use crate::guilds::riftwalker::{
    AIR_SKILL, EARTH_SKILL, ENTITY_LABEL_AIR, ENTITY_LABEL_EARTH, ENTITY_LABEL_FIRE,
    ENTITY_LABEL_WATER, FIRE_SKILL, RIFTWALKER_ELEMENT_VAR, RIFTWALKER_HAS_ENTITY_FLAG,
    RIFTWALKER_SKILL_VAR, WATER_SKILL,
};
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use regex::Regex;

impl RiftwalkerGuild {
    pub fn get_triggers(&self) -> Vec<Trigger> {
        vec![Self::primary_trigger]
    }

    pub fn primary_trigger(
        ctx: &mut TriggerContext<'_>,
        styled_line: &mut StyledLine,
    ) -> TriggerOutput {
        let mut output = TriggerOutput::default();
        let line = styled_line.plain_line.as_str();
        clears_and_keeps(line, ctx.automation, &mut output);
        line_sync_entity_skill(ctx, line, &mut output);
        skill_state_echoes(ctx, line, &mut output);

        elemental_line_paint(ctx, styled_line);
        summon_entity_line_paint(ctx, styled_line);
        elemental_tokens_paint(ctx, styled_line);
        aura_paint(ctx, styled_line);
        misc_paint(ctx, styled_line);
        entity_sense_paint(styled_line);

        output
    }
}

fn automation_label(automation: &Automation, key: &str) -> String {
    automation
        .get_var(key)
        .cloned()
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "entity".to_string())
}

/// Sorted, deduped, regex-escaped alternation of per-element nouns.
fn noun_alt_pattern(automation: &Automation) -> String {
    let mut parts = vec![
        automation_label(automation, ENTITY_LABEL_FIRE),
        automation_label(automation, ENTITY_LABEL_AIR),
        automation_label(automation, ENTITY_LABEL_WATER),
        automation_label(automation, ENTITY_LABEL_EARTH),
    ];
    parts.sort();
    parts.dedup();
    parts
        .into_iter()
        .map(|s| regex::escape(&s))
        .collect::<Vec<_>>()
        .join("|")
}

fn clears_and_keeps(line: &str, automation: &Automation, output: &mut TriggerOutput) {
    let n_alt = noun_alt_pattern(automation);
    let lost_entity = format!(
        r"(?i)^Your\s+(?:{n_alt})\s+begins to warp, seeming to become unstable\. It folds in on itself and vanishes!$"
    );
    let lost_soul = r"(?i)^Your soul cries out in anguish as your faithful companion is slain!$";
    if Regex::new(&lost_entity).is_ok_and(|re| re.is_match(line.trim()))
        || Regex::new(lost_soul).is_ok_and(|re| re.is_match(line.trim()))
    {
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

fn line_sync_entity_skill(ctx: &TriggerContext<'_>, line: &str, output: &mut TriggerOutput) {
    let trimmed = line.trim();
    let mapping = [
        ("fire", FIRE_SKILL, "fire", ENTITY_LABEL_FIRE),
        ("air", AIR_SKILL, "air", ENTITY_LABEL_AIR),
        ("water", WATER_SKILL, "water", ENTITY_LABEL_WATER),
        ("earth", EARTH_SKILL, "earth", ENTITY_LABEL_EARTH),
    ];

    for &(glyph, mastery, bucket, key) in &mapping {
        let noun = regex::escape(&automation_label(ctx.automation, key));
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

fn is_prepared_line(automation: &Automation, line: &str) -> bool {
    let n_alt = noun_alt_pattern(automation);
    let Ok(re) = Regex::new(&format!(
        r"(?i)^Your\s+(?:{n_alt})\s+is prepared to do the skill\.?$"
    )) else {
        return false;
    };
    re.is_match(line.trim())
}

fn is_concentration_lost_line(automation: &Automation, line: &str) -> bool {
    let n_alt = noun_alt_pattern(automation);
    let Ok(re) = Regex::new(&format!(
        r"(?i)^Your\s+(?:{n_alt})\s+loses its concentration and cannot do the skill\.$"
    )) else {
        return false;
    };
    re.is_match(line.trim())
}

fn skill_state_echoes(ctx: &TriggerContext<'_>, line: &str, output: &mut TriggerOutput) {
    if is_prepared_line(ctx.automation, line) {
        let mut banner = StyledLine::new("!!!!!!!!!! Entity Skill !!!!!!!!!!");
        banner.set_line_style(TextStyle::BRIGHT_BLUE);
        output.lines.push(banner);
        return;
    }

    if is_concentration_lost_line(ctx.automation, line) {
        if current_skill_equals(ctx, AIR_SKILL) {
            output
                .lines
                .push(down_notice("SUFFOCATING EMBRACE IS DOWN!"));
        } else if current_skill_equals(ctx, EARTH_SKILL) {
            output.lines.push(down_notice("EARTHEN COVER IS DOWN!"));
        } else if current_skill_equals(ctx, WATER_SKILL) {
            output
                .lines
                .push(down_notice("SUBJUGATING BACKWASH IS DOWN!"));
        }
        return;
    }

    let air_lab = automation_label(ctx.automation, ENTITY_LABEL_AIR);
    if matches_line_insensitive(
        line,
        format!("Your air {air_lab} falters and its wispy tendrils fall to its sides."),
    ) && current_skill_equals(ctx, AIR_SKILL)
    {
        output
            .lines
            .push(down_notice("SUFFOCATING EMBRACE IS DOWN!"));
        return;
    }

    let earth_lab = automation_label(ctx.automation, ENTITY_LABEL_EARTH);
    if matches_line_insensitive(
        line,
        format!("Your earth {earth_lab} hunches down looking much less solid than a second ago."),
    ) && current_skill_equals(ctx, EARTH_SKILL)
    {
        output.lines.push(down_notice("EARTHEN COVER IS DOWN!"));
        return;
    }

    let water_lab = automation_label(ctx.automation, ENTITY_LABEL_WATER);
    if matches_line_insensitive(
        line,
        format!("Your water {water_lab} stops glowing and its skin becomes still."),
    ) && current_skill_equals(ctx, WATER_SKILL)
    {
        output
            .lines
            .push(down_notice("SUBJUGATING BACKWASH IS DOWN!"));
    }
}

fn matches_line_insensitive(got: &str, expected_ascii: String) -> bool {
    got.eq_ignore_ascii_case(expected_ascii.as_str())
}

fn current_skill_equals(ctx: &TriggerContext<'_>, needle: &str) -> bool {
    ctx.automation
        .get_var(RIFTWALKER_SKILL_VAR)
        .map(|skill| skill == needle)
        .unwrap_or(false)
}

fn down_notice(message: &'static str) -> StyledLine {
    let mut line = StyledLine::new(message);
    line.set_line_style(TextStyle::BRIGHT_RED);
    line
}

fn elemental_line_paint(ctx: &TriggerContext<'_>, styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    let f = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_FIRE));
    let a = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_AIR));
    let w = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_WATER));
    let e = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_EARTH));

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

    let n_alt = noun_alt_pattern(ctx.automation);
    let Ok(miss) = Regex::new(&format!(
        r"^Your (.+)\s+(?:{n_alt})\s+does some strange combat maneuver but doesn't hit anything\.$"
    )) else {
        return;
    };

    if hit.is_match(text) {
        styled_line.set_line_style(TextStyle::GREEN);
    } else if stun.is_match(text) {
        styled_line.set_line_style(TextStyle::RED);
    } else if miss.is_match(text) {
        styled_line.set_line_style(TextStyle::BRIGHT_RED);
    }
}

fn summon_entity_line_paint(ctx: &TriggerContext<'_>, styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    let rows = [
        ("air", ENTITY_LABEL_AIR, TextStyle::BRIGHT_CYAN),
        ("fire", ENTITY_LABEL_FIRE, TextStyle::RED),
        ("water", ENTITY_LABEL_WATER, TextStyle::BLUE),
        ("earth", ENTITY_LABEL_EARTH, TextStyle::YELLOW),
    ];
    for &(elem, key, style) in &rows {
        let n = regex::escape(&automation_label(ctx.automation, key));
        let Ok(re) = Regex::new(&format!(
            r"(?i)^(?:A|An)\s+(.+)\s+{elem}\s+{n}\s+(.+?)\s+with power \[yours\]$"
        )) else {
            continue;
        };
        if re.is_match(text) {
            styled_line.set_line_style(style);
            return;
        }
    }
}

fn elemental_tokens_paint(ctx: &TriggerContext<'_>, styled_line: &mut StyledLine) {
    let pairs = [
        ("Fire entity", ENTITY_LABEL_FIRE, TextStyle::RED),
        ("Air entity", ENTITY_LABEL_AIR, TextStyle::BRIGHT_CYAN),
        ("Water entity", ENTITY_LABEL_WATER, TextStyle::BLUE),
        ("Earth entity", ENTITY_LABEL_EARTH, TextStyle::YELLOW),
    ];
    for (stock_slice, configuration_key, style) in pairs {
        styled_line.set_block_style(stock_slice, style);
        let customized = stock_slice.replace(
            "entity",
            automation_label(ctx.automation, configuration_key).as_str(),
        );
        if customized != stock_slice {
            styled_line.set_block_style(customized.as_str(), style);
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

fn aura_paint(ctx: &TriggerContext<'_>, styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    let n_alt = noun_alt_pattern(ctx.automation);
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
    styled_line.set_plain_byte_range_style(aur_m.range(), style);
}

fn misc_paint(ctx: &TriggerContext<'_>, styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    let a = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_AIR));
    if let Ok(air_embrace) = Regex::new(&format!(
        r"(?i)^Air\s+{a}\s+embraces .+ with its wispy tendrils\.$"
    )) && air_embrace.is_match(text)
    {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }

    let n_alt = noun_alt_pattern(ctx.automation);
    if let Ok(wave) = Regex::new(&format!(
        r"(?i)^A wave of blue light bursts forth from your\s+(?:{n_alt})\s+and hits you in the chest\.$"
    )) && wave.is_match(text)
    {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }

    let w = regex::escape(&automation_label(ctx.automation, ENTITY_LABEL_WATER));
    if let Ok(water_glow) = Regex::new(&format!(r"(?i)^Water\s+{w}\s+starts to glow,.+shore\.$"))
        && water_glow.is_match(text)
    {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }

    if let Ok(skill_focus) = Regex::new(&format!(
        r"(?i)^.+\s+(?:{n_alt})\s+starts concentrating on a new offensive skill\.$"
    )) && skill_focus.is_match(text)
    {
        styled_line.set_line_style(TextStyle::BRIGHT_WHITE);
        return;
    }

    if is_prepared_line(ctx.automation, text) {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }
    if text.contains("crumpled piece of paper flies through the air") {
        styled_line.set_line_style(TextStyle::GREEN);
        return;
    }
    if let Ok(regain) = Regex::new(&format!(
        r"(?i)^You manage to regain control of your\s+(?:{n_alt})\s+before the connection is completely broken!$"
    )) && regain.is_match(text)
    {
        styled_line.set_line_style(TextStyle::GREEN);
    }
}

fn entity_sense_paint(styled_line: &mut StyledLine) {
    styled_line.set_block_style("Entity sense:", TextStyle::BRIGHT_BLUE);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::stats::Stats;
    use crate::triggers::TriggerContext;
    use unicode_segmentation::UnicodeSegmentation;

    #[test]
    fn aura_only_on_summon_line_not_random_sparkling() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut auto,
            rig: None,
            player_name: None,
        };
        let mut noise = StyledLine::new("The river is sparkling in the sun.");
        RiftwalkerGuild::primary_trigger(&mut ctx, &mut noise);
        let beta = noise.plain_line.find("sparkling").unwrap();
        assert_eq!(
            noise.styled_chars[noise.plain_line[..beta].graphemes(true).count()].color,
            crate::ansi::TextColor::Default
        );

        let mut summon = StyledLine::new("A huge fire entity sparkling with power [yours]");
        RiftwalkerGuild::primary_trigger(&mut ctx, &mut summon);
        let sp = summon.plain_line.find("sparkling").unwrap();
        let idx = summon.plain_line[..sp].graphemes(true).count();
        assert_eq!(summon.styled_chars[idx].color, TextStyle::YELLOW.color);
    }

    #[test]
    fn aura_respects_custom_entity_noun() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_var(ENTITY_LABEL_FIRE, "golem".to_string());
        let mut ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut auto,
            rig: None,
            player_name: None,
        };
        let mut line = StyledLine::new("A huge fire golem gleaming with power [yours]");
        RiftwalkerGuild::primary_trigger(&mut ctx, &mut line);
        let g = line.plain_line.find("gleaming").unwrap();
        let idx = line.plain_line[..g].graphemes(true).count();
        assert_eq!(line.styled_chars[idx].color, TextStyle::CYAN.color);
    }

    #[test]
    fn skill_sync_requires_element_and_configured_noun() {
        let mut stats = Stats::default();
        let mut auto = Automation::new();
        auto.set_var(ENTITY_LABEL_FIRE, "golem".to_string());
        let ctx = TriggerContext {
            stats: &mut stats,
            automation: &mut auto,
            rig: None,
            player_name: None,
        };
        let mut out = TriggerOutput::default();
        line_sync_entity_skill(&ctx, "Some prefix fire entity with power [yours]", &mut out);
        assert!(out.actions.is_empty());

        line_sync_entity_skill(
            &ctx,
            "Some prefix fire golem trailing with power [yours]",
            &mut out,
        );
        assert!(!out.actions.is_empty());
    }
}
