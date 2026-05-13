use crate::ansi::{StyledLine, TextStyle};
use crate::automation::Action;
use crate::guilds::RiftwalkerGuild;
use crate::guilds::riftwalker::{
    AIR_SKILL, EARTH_SKILL, ENTITY_LABEL_AIR, ENTITY_LABEL_EARTH, ENTITY_LABEL_FIRE,
    ENTITY_LABEL_WATER, FIRE_SKILL, RIFTWALKER_ELEMENT_VAR, RIFTWALKER_HAS_ENTITY_FLAG,
    RIFTWALKER_SKILL_VAR, WATER_SKILL,
};
use crate::triggers::{Trigger, TriggerContext, TriggerOutput};
use lazy_static::lazy_static;
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
        clears_and_keeps(line, &mut output);
        line_sync_entity_skill(ctx, line, &mut output);
        skill_state_echoes(ctx, line, &mut output);

        elemental_line_paint(styled_line);
        elemental_tokens_paint(ctx, styled_line);
        aura_paint(styled_line);
        misc_paint(styled_line);
        entity_sense_paint(styled_line);

        output
    }
}

fn automation_label(automation: &crate::automation::Automation, key: &str) -> String {
    automation
        .get_var(key)
        .cloned()
        .filter(|segment| !segment.is_empty())
        .unwrap_or_else(|| "entity".to_string())
}

lazy_static! {
    static ref ENTITY_LOST_RE: Regex = Regex::new(
        "(?i)^(Your entity begins to warp.*?vanishes!|Your soul cries out in anguish as your faithful companion is slain!)$",
    ).unwrap();
    static ref SKILL_SYNC_ROW: Regex = Regex::new(
        r"(?i).*(?P<glyph>Fire|Air|Water|Earth)\s+(?P<label>[^\s]+).*\[yours\]",
    )
    .unwrap();
    static ref SKILL_READY_TAIL: Regex = Regex::new(
        "(?i)is prepared to do the skill\\.?$",
    )
    .unwrap();
    static ref ELEMENTAL_HIT: Regex =
        Regex::new(r"^(Fire|Air|Water|Earth) entity hits .+ (once|twice|thrice) .+\.$").unwrap();
    static ref ELEMENTAL_STUN: Regex =
        Regex::new(r"^(Fire|Air|Water|Earth) entity is stunned\.$").unwrap();
    static ref ENTITY_MISS: Regex = Regex::new(
        r"^Your (.+) entity does some strange combat maneuver but doesn't hit anything\.$",
    )
    .unwrap();
    static ref AIR_EMBRACE: Regex =
        Regex::new(r"(?i)Air entity embraces .+ with its wispy tendrils\.").unwrap();
    static ref WATER_GLOW: Regex =
        Regex::new(r"(?i)Water entity starts to glow,.+shore\.").unwrap();
    static ref SKILL_FOCUS: Regex =
        Regex::new(r"(?i).+ entity starts concentrating on a new offensive skill\.").unwrap();
}

fn clears_and_keeps(line: &str, output: &mut TriggerOutput) {
    if ENTITY_LOST_RE.is_match(line.trim()) {
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
    let Some(captures) = SKILL_SYNC_ROW.captures(trimmed) else {
        return;
    };
    let keyword = captures
        .name("glyph")
        .map(|found| found.as_str().to_ascii_lowercase())
        .unwrap_or_default();

    let label_token = captures
        .name("label")
        .map(|found| found.as_str().to_ascii_lowercase())
        .unwrap_or_default();

    let mapping = [
        ("fire", FIRE_SKILL, "fire", ENTITY_LABEL_FIRE),
        ("air", AIR_SKILL, "air", ENTITY_LABEL_AIR),
        ("water", WATER_SKILL, "water", ENTITY_LABEL_WATER),
        ("earth", EARTH_SKILL, "earth", ENTITY_LABEL_EARTH),
    ];

    let mut matched_skill = None;
    for &(glyph_piece, mastery, bucket, settings_key) in mapping.iter() {
        if glyph_piece != keyword {
            continue;
        }
        let configured_label = automation_label(ctx.automation, settings_key).to_ascii_lowercase();
        if configured_label == label_token {
            matched_skill = Some((mastery, bucket));
            break;
        }
    }

    let Some((discipline, column)) = matched_skill else {
        return;
    };
    entity_skill_actions(discipline, column)
        .into_iter()
        .for_each(|action| output.actions.push(action));
}

fn entity_skill_actions(skill: &str, element: &str) -> Vec<Action> {
    vec![
        Action::SetVar(RIFTWALKER_SKILL_VAR.to_string(), skill.to_string()),
        Action::SetVar(RIFTWALKER_ELEMENT_VAR.to_string(), element.to_string()),
        Action::SetFlag(RIFTWALKER_HAS_ENTITY_FLAG.to_string(), true),
    ]
}

fn skill_state_echoes(ctx: &TriggerContext<'_>, line: &str, output: &mut TriggerOutput) {
    if SKILL_READY_TAIL.is_match(line) {
        output
            .lines
            .push(StyledLine::new("!!!!!!!!!! Entity Skill !!!!!!!!!!"));
        return;
    }

    if line == "Your entity loses its concentration and cannot do the skill." {
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
        format!("Your earth {earth_lab} hunches down looking much less solid than a second ago.",),
    ) && current_skill_equals(ctx, EARTH_SKILL)
    {
        output.lines.push(down_notice("EARTHEN COVER IS DOWN!"));
        return;
    }

    let water_lab = automation_label(ctx.automation, ENTITY_LABEL_WATER);
    if matches_line_insensitive(
        line,
        format!("Your water {water_lab} stops glowing and its skin becomes still.",),
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

fn elemental_line_paint(styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    if ELEMENTAL_HIT.is_match(text) {
        styled_line.set_line_style(TextStyle::GREEN);
    } else if ELEMENTAL_STUN.is_match(text) {
        styled_line.set_line_style(TextStyle::RED);
    } else if ENTITY_MISS.is_match(text) {
        styled_line.set_line_style(TextStyle::BRIGHT_RED);
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

fn aura_paint(styled_line: &mut StyledLine) {
    const TABLE: &[(&str, TextStyle)] = &[
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
    for &(token, style) in TABLE {
        styled_line.set_block_style(token, style);
    }
}

fn misc_paint(styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.as_str();
    if AIR_EMBRACE.is_match(text) {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }
    if text == "A wave of blue light bursts forth from your entity and hits you in the chest." {
        styled_line.set_line_style(TextStyle::BLUE);
        return;
    }
    if WATER_GLOW.is_match(text) {
        styled_line.set_line_style(TextStyle::BRIGHT_BLUE);
        return;
    }
    if SKILL_FOCUS.is_match(text) {
        styled_line.set_line_style(TextStyle::BRIGHT_WHITE);
    }
}

lazy_static! {
    static ref ENTITY_SENSE_SEGMENT: Regex = Regex::new(r"\(Entity sense:[^\)]*\)").unwrap();
}

fn entity_sense_paint(styled_line: &mut StyledLine) {
    let text = styled_line.plain_line.clone();
    if let Some(found) = ENTITY_SENSE_SEGMENT.find(text.as_str()) {
        styled_line.set_block_style(found.as_str(), TextStyle::BRIGHT_BLUE);
    }
}
