use crate::ansi::{StyledLine, TextStyle};
use crate::automation::{Action, AutomationFlags, AutomationVars};
use crate::guilds::MonkSkillsConfig;
use crate::secondary_status::SecondaryStatusEffect;
use crate::stats::StatsEffect;
use serde::{Deserialize, Serialize};
use std::sync::{LazyLock, OnceLock};

pub use crate::ansi::LineEffect;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TriggerConfig {
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub guild_triggers: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub spell_vocals: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub common_triggers: bool,
    #[serde(default = "default_true", skip_serializing_if = "is_true")]
    pub core_triggers: bool,
}

impl Default for TriggerConfig {
    fn default() -> Self {
        Self {
            guild_triggers: true,
            spell_vocals: true,
            common_triggers: true,
            core_triggers: true,
        }
    }
}

impl TriggerConfig {
    pub fn is_default(&self) -> bool {
        self.guild_triggers && self.spell_vocals && self.common_triggers && self.core_triggers
    }
}

fn is_true(value: &bool) -> bool {
    *value
}

fn default_true() -> bool {
    true
}

mod common;
pub(crate) mod money_summary;
mod player_combat_rules;
mod prompt;
mod recovery_bracket;
pub(crate) mod rule_engine;
mod short_score;
pub use short_score::SC_REGEX;
mod spell_vocal_data;
mod spell_vocals;

static COMMON_TRIGGERS: LazyLock<Vec<Trigger>> = LazyLock::new(|| vec![common::trigger]);
static CORE_TRIGGERS: LazyLock<Vec<Trigger>> = LazyLock::new(|| {
    vec![
        prompt::trigger,
        short_score::trigger,
        recovery_bracket::trigger,
    ]
});

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriggerLine<'a> {
    pub plain_line: &'a str,
}

impl<'a> TriggerLine<'a> {
    pub fn new(plain_line: &'a str) -> Self {
        Self { plain_line }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct TriggerContext<'a> {
    flags: &'a AutomationFlags,
    vars: &'a AutomationVars,
    pub rig: Option<&'a str>,
    pub player_name: Option<&'a str>,
    monk_skills: MonkSkillsConfig,
}

static DEFAULT_AUTOMATION_FLAGS: OnceLock<AutomationFlags> = OnceLock::new();
static DEFAULT_AUTOMATION_VARS: OnceLock<AutomationVars> = OnceLock::new();
static DEFAULT_MONK_SKILLS: OnceLock<MonkSkillsConfig> = OnceLock::new();

impl<'a> TriggerContext<'a> {
    pub fn new(
        flags: &'a AutomationFlags,
        vars: &'a AutomationVars,
        rig: Option<&'a str>,
        player_name: Option<&'a str>,
        monk_skills: MonkSkillsConfig,
    ) -> Self {
        Self {
            flags,
            vars,
            rig,
            player_name,
            monk_skills,
        }
    }

    pub fn monk_skills(&self) -> &MonkSkillsConfig {
        &self.monk_skills
    }

    pub fn flag_is_set(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }

    pub fn get_var(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str)
    }

    pub fn rig(&self) -> Option<&str> {
        self.rig
    }

    pub fn player_name(&self) -> Option<&str> {
        self.player_name
    }
}

impl Default for TriggerContext<'_> {
    fn default() -> Self {
        Self::new(
            DEFAULT_AUTOMATION_FLAGS.get_or_init(AutomationFlags::default),
            DEFAULT_AUTOMATION_VARS.get_or_init(AutomationVars::default),
            None,
            None,
            DEFAULT_MONK_SKILLS
                .get_or_init(MonkSkillsConfig::default)
                .clone(),
        )
    }
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct OriginalLineEffects {
    pub gag: bool,
    pub edits: Vec<LineEffect>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct TriggerEffects {
    pub original: OriginalLineEffects,
    pub lines: Vec<StyledLine>,
    pub actions: Vec<Action>,
    pub stats: Vec<StatsEffect>,
    pub secondary_status: Vec<SecondaryStatusEffect>,
}

impl TriggerEffects {
    pub fn none() -> Self {
        Self::default()
    }

    pub fn gag(mut self) -> Self {
        self.original.gag = true;
        self
    }

    pub fn style_line(mut self, style: TextStyle) -> Self {
        self.original.edits.push(LineEffect::StyleLine(style));
        self
    }

    pub fn style_block(mut self, text: impl Into<String>, style: TextStyle) -> Self {
        self.original.edits.push(LineEffect::StyleBlock {
            text: text.into(),
            style,
        });
        self
    }

    pub fn insert_plain_after_plain_byte_idx(
        mut self,
        byte_idx: usize,
        suffix: impl Into<String>,
    ) -> Self {
        self.original
            .edits
            .push(LineEffect::InsertPlainAfterPlainByteIdx {
                byte_idx,
                suffix: suffix.into(),
            });
        self
    }

    pub fn emit(mut self, line: StyledLine) -> Self {
        self.lines.push(line);
        self
    }

    pub fn stat(mut self, effect: StatsEffect) -> Self {
        self.stats.push(effect);
        self
    }

    pub fn secondary_status(mut self, effect: SecondaryStatusEffect) -> Self {
        self.secondary_status.push(effect);
        self
    }

    pub fn extend(&mut self, other: TriggerEffects) {
        self.original.gag |= other.original.gag;
        self.original.edits.extend(other.original.edits);
        self.lines.extend(other.lines);
        self.actions.extend(other.actions);
        self.stats.extend(other.stats);
        self.secondary_status.extend(other.secondary_status);
    }

    pub fn apply_line_effects_to(&self, line: &mut StyledLine) {
        for edit in &self.original.edits {
            edit.apply_to(line);
        }
        if self.original.gag {
            line.gag = true;
        }
    }
}

pub type Trigger =
    for<'a> fn(line: &TriggerLine<'_>, context: &TriggerContext<'a>) -> TriggerEffects;

pub fn common_trigger_catalog() -> Vec<crate::command::TriggerCatalogEntry> {
    common::trigger_catalog()
}

pub fn process(
    context: &TriggerContext<'_>,
    guild_triggers: &[Trigger],
    line: &str,
    config: &TriggerConfig,
) -> TriggerEffects {
    let mut current_line = StyledLine::new(line);
    let mut output = TriggerEffects::default();

    // Guild triggers first so stats hooks (e.g. Animist soul companion) always run before spell labels and common rules.
    if config.guild_triggers {
        for trigger in guild_triggers {
            let result = trigger(&TriggerLine::new(&current_line.plain_line), context);
            result.apply_line_effects_to(&mut current_line);
            output.extend(result);
        }
    }

    if config.spell_vocals {
        let result = spell_vocals::trigger(&TriggerLine::new(&current_line.plain_line), context);
        result.apply_line_effects_to(&mut current_line);
        output.extend(result);
    }

    if config.common_triggers {
        for trigger in COMMON_TRIGGERS.iter() {
            let result = trigger(&TriggerLine::new(&current_line.plain_line), context);
            result.apply_line_effects_to(&mut current_line);
            output.extend(result);
        }
    }

    if config.core_triggers {
        for trigger in CORE_TRIGGERS.iter() {
            let result = trigger(&TriggerLine::new(&current_line.plain_line), context);
            result.apply_line_effects_to(&mut current_line);
            output.extend(result);
        }
    }

    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ansi::AnsiCode;
    use crate::ansi::StyledLine;
    use crate::guilds::{AnimistGuild, Guild, MonkGuild};
    use std::collections::HashMap;

    fn companion_line_is_blue(output: &TriggerEffects, line: &str) -> bool {
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled
            .styled_chars
            .iter()
            .all(|c| c.color == AnsiCode::Blue)
    }

    fn player_hit_line_is_green(output: &TriggerEffects, line: &str) -> bool {
        let mut styled = StyledLine::new(line);
        output.apply_line_effects_to(&mut styled);
        styled.styled_chars[0].color == AnsiCode::Green
    }

    #[test]
    fn process_without_animist_applies_player_combat_hit_hilite() {
        let text = "Fueryon hits Reaver 5 times causing a nasty laceration.";
        let flags = HashMap::new();
        let vars = HashMap::new();
        let facts = TriggerContext::new(
            &flags,
            &vars,
            None,
            Some("Fueryon"),
            MonkSkillsConfig::default(),
        );
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(MonkGuild::default())];
        let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
        let output = process(&facts, &guild_triggers, text, &TriggerConfig::default());
        assert!(
            player_hit_line_is_green(&output, text),
            "player combat hit hilite should run without Animist active"
        );
    }

    #[test]
    fn process_without_animist_skips_companion_combat_hilite() {
        let text = "A blue-glowing soul companion [Nynn].";
        let flags = HashMap::new();
        let vars = HashMap::new();
        let facts = TriggerContext::new(
            &flags,
            &vars,
            None,
            Some("Nynn"),
            MonkSkillsConfig::default(),
        );
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(MonkGuild::default())];
        let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
        let output = process(&facts, &guild_triggers, text, &TriggerConfig::default());
        assert!(
            !companion_line_is_blue(&output, text),
            "companion hilite should not run without Animist active"
        );
    }

    #[test]
    fn process_with_animist_applies_companion_combat_hilite() {
        let text = "A blue-glowing soul companion [Nynn].";
        let flags = HashMap::new();
        let vars = HashMap::new();
        let facts = TriggerContext::new(
            &flags,
            &vars,
            None,
            Some("Nynn"),
            MonkSkillsConfig::default(),
        );
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(AnimistGuild::default())];
        let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
        let output = process(&facts, &guild_triggers, text, &TriggerConfig::default());
        assert!(
            companion_line_is_blue(&output, text),
            "companion hilite should run when Animist is active"
        );
    }

    #[test]
    fn process_with_guild_triggers_disabled_skips_companion_combat_hilite() {
        let text = "A blue-glowing soul companion [Nynn].";
        let flags = HashMap::new();
        let vars = HashMap::new();
        let facts = TriggerContext::new(
            &flags,
            &vars,
            None,
            Some("Nynn"),
            MonkSkillsConfig::default(),
        );
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(AnimistGuild::default())];
        let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
        let config = TriggerConfig {
            guild_triggers: false,
            ..TriggerConfig::default()
        };
        let output = process(&facts, &guild_triggers, text, &config);
        assert!(
            !companion_line_is_blue(&output, text),
            "companion hilite should not run when guild triggers are disabled"
        );
    }

    #[test]
    fn trigger_config_serde_roundtrip_defaults_omit_section() {
        let config = TriggerConfig::default();
        let text = toml::to_string_pretty(&config).unwrap();
        assert!(!text.contains("guild_triggers"));
        assert!(!text.contains("[triggers]"));

        let partial = TriggerConfig {
            guild_triggers: false,
            core_triggers: false,
            ..TriggerConfig::default()
        };
        let text = toml::to_string_pretty(&partial).unwrap();
        assert!(text.contains("guild_triggers = false"));
        assert!(text.contains("core_triggers = false"));
        assert!(!text.contains("spell_vocals"));
        assert!(!text.contains("common_triggers"));

        let parsed: TriggerConfig = toml::from_str(
            r#"
guild_triggers = false
"#,
        )
        .unwrap();
        assert!(!parsed.guild_triggers);
        assert!(parsed.spell_vocals);
        assert!(parsed.common_triggers);
        assert!(parsed.core_triggers);
    }
}
