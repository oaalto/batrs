use crate::config::{GenericCommandsConfig, UserSettings, is_truthy_setting_value};
use crate::guilds::MonkSkillsConfig;
use crate::guilds::catalog::{DEFAULT_GUILD_PRIMARY_KEYWORD, GuildSelection};
use crate::player_profile::automation::{
    automation_flags_for_settings, automation_vars_for_settings,
};
use crate::player_profile::registry::{SETTINGS_DEFS, SettingDefinition, SettingKind, SettingSlot};
use crate::triggers::TriggerConfig;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct KnownProfileSettings {
    pub rig: String,
    pub tzarakk_mount: String,
    pub sabre_weapon: String,
    pub riftwalker_entity_labels: [String; 4],
    pub is_lich: bool,
}

impl KnownProfileSettings {
    pub fn rig_for_triggers(&self) -> Option<&str> {
        non_empty(&self.rig)
    }
}

impl KnownProfileSettings {
    fn from_user_settings(settings: &UserSettings) -> Self {
        let mut known = Self {
            rig: String::new(),
            tzarakk_mount: String::new(),
            sabre_weapon: String::new(),
            riftwalker_entity_labels: default_riftwalker_entity_labels(),
            is_lich: false,
        };
        for definition in SETTINGS_DEFS {
            let raw = setting_value(settings, definition.key);
            let value = normalized_setting_value(definition, raw);
            definition.slot.write(&mut known, value);
        }
        known
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildDialogProfileDefaults {
    pub primary_background: String,
    pub tzarakk_mount: String,
    pub sabre_weapon: String,
    pub riftwalker_entity_labels: [String; 4],
}

impl GuildDialogProfileDefaults {
    fn from_settings(primary_background: &str, settings: &KnownProfileSettings) -> Self {
        let mut defaults = Self {
            primary_background: primary_background.to_string(),
            tzarakk_mount: String::new(),
            sabre_weapon: String::new(),
            riftwalker_entity_labels: default_riftwalker_entity_labels(),
        };
        for definition in SETTINGS_DEFS
            .iter()
            .filter(|definition| definition.guild_dialog)
        {
            definition
                .slot
                .write_guild_dialog(&mut defaults, definition.slot.read(settings));
        }
        defaults
    }
}

pub(crate) fn default_riftwalker_entity_labels() -> [String; 4] {
    std::array::from_fn(|_| "entity".to_string())
}

fn setting_value(settings: &UserSettings, key: &str) -> String {
    settings.get(key).unwrap_or_default().to_string()
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

fn normalized_setting_value(definition: &SettingDefinition, raw: String) -> String {
    match definition.kind {
        SettingKind::String => match definition.slot {
            SettingSlot::RiftwalkerEntity(_) if raw.is_empty() => definition.default.to_string(),
            _ => raw,
        },
        SettingKind::Bool => {
            if is_truthy_setting_value(&raw) {
                raw
            } else {
                definition.default.to_string()
            }
        }
    }
}

pub fn runtime_profile_from_parts(
    selected_guild_keys: Vec<String>,
    guild_primary_background: &str,
    settings: UserSettings,
    generic_commands_config: GenericCommandsConfig,
    trigger_config: TriggerConfig,
    monk_skills_config: MonkSkillsConfig,
) -> PlayerRuntimeProfile {
    let known_settings = KnownProfileSettings::from_user_settings(&settings);
    let guild_selection =
        GuildSelection::from_persisted_keys(&selected_guild_keys, Some(guild_primary_background));
    let guild_primary_background = guild_selection.primary_background_keyword().to_string();
    let automation_vars = automation_vars_for_settings(&known_settings);
    let automation_flags = automation_flags_for_settings(&known_settings);
    let guild_dialog_defaults =
        GuildDialogProfileDefaults::from_settings(&guild_primary_background, &known_settings);

    PlayerRuntimeProfile {
        guild_selection,
        guild_primary_background,
        generic_commands_config,
        trigger_config,
        monk_skills_config,
        settings: known_settings,
        automation_vars,
        automation_flags,
        guild_dialog_defaults,
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerRuntimeProfile {
    pub guild_selection: GuildSelection,
    pub guild_primary_background: String,
    pub generic_commands_config: GenericCommandsConfig,
    pub trigger_config: TriggerConfig,
    pub monk_skills_config: MonkSkillsConfig,
    pub settings: KnownProfileSettings,
    pub automation_vars: Vec<(String, String)>,
    pub automation_flags: Vec<(String, bool)>,
    pub guild_dialog_defaults: GuildDialogProfileDefaults,
}

pub struct InterpretedPlayerProfile {
    pub normalized_player: crate::config::PlayerToml,
    pub changed: bool,
    pub runtime: PlayerRuntimeProfile,
}

impl Default for PlayerRuntimeProfile {
    fn default() -> Self {
        runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            UserSettings::default(),
            GenericCommandsConfig::default(),
            TriggerConfig::default(),
            MonkSkillsConfig::default(),
        )
    }
}
