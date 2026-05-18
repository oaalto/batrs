use crate::config::{GenericCommandsConfig, SettingEntry, UserSettings};
use crate::guilds::grouping::DEFAULT_GUILD_PRIMARY_KEYWORD;
use crate::guilds::guild_definitions;

const RIG_KEY: &str = "rig";
pub const TZARAKK_MOUNT_KEY: &str = "tzarakk_mount";
pub const SABRE_WEAPON_KEY: &str = "sabre_weapon";
const RIFTWALKER_ENTITY_FIRE_KEY: &str = "riftwalker_entity_fire";
const RIFTWALKER_ENTITY_AIR_KEY: &str = "riftwalker_entity_air";
const RIFTWALKER_ENTITY_WATER_KEY: &str = "riftwalker_entity_water";
const RIFTWALKER_ENTITY_EARTH_KEY: &str = "riftwalker_entity_earth";
const IS_LICH_KEY: &str = "is_lich";
const DEFAULT_RIFTWALKER_ENTITY_LABEL: &str = "entity";

pub const RIFTWALKER_ENTITY_LABEL_KEYS: [&str; 4] = [
    RIFTWALKER_ENTITY_FIRE_KEY,
    RIFTWALKER_ENTITY_AIR_KEY,
    RIFTWALKER_ENTITY_WATER_KEY,
    RIFTWALKER_ENTITY_EARTH_KEY,
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerRuntimeProfile {
    pub selected_guild_keys: Option<Vec<String>>,
    pub guild_primary_background: String,
    pub settings_entries: Vec<SettingEntry>,
    pub generic_commands_config: GenericCommandsConfig,
    pub settings: KnownProfileSettings,
    pub automation_vars: Vec<(String, String)>,
    pub automation_flags: Vec<(String, bool)>,
    pub guild_dialog_defaults: GuildDialogProfileDefaults,
}

impl Default for PlayerRuntimeProfile {
    fn default() -> Self {
        runtime_profile(
            None,
            None,
            UserSettings::default(),
            GenericCommandsConfig::default(),
        )
    }
}

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

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildDialogProfileDefaults {
    pub primary_background: String,
    pub tzarakk_mount: String,
    pub sabre_weapon: String,
    pub riftwalker_entity_labels: [String; 4],
}

#[cfg(test)]
fn default_riftwalker_entity_labels() -> [String; 4] {
    std::array::from_fn(|_| DEFAULT_RIFTWALKER_ENTITY_LABEL.to_string())
}

pub fn runtime_profile(
    selected_guild_keys: Option<Vec<String>>,
    guild_primary_background: Option<&str>,
    settings: UserSettings,
    generic_commands_config: GenericCommandsConfig,
) -> PlayerRuntimeProfile {
    let known_settings = KnownProfileSettings::from_user_settings(&settings);
    let selected_guild_keys = selected_guild_keys.map(filter_known_guilds);
    let guild_primary_background = guild_primary_background
        .filter(|background| !background.is_empty())
        .unwrap_or(DEFAULT_GUILD_PRIMARY_KEYWORD)
        .to_string();
    let automation_vars = automation_vars_for_settings(&known_settings);
    let automation_flags = vec![(IS_LICH_KEY.to_string(), known_settings.is_lich)];
    let guild_dialog_defaults =
        GuildDialogProfileDefaults::from_settings(&guild_primary_background, &known_settings);

    PlayerRuntimeProfile {
        selected_guild_keys,
        guild_primary_background,
        settings_entries: settings.entries,
        generic_commands_config,
        settings: known_settings,
        automation_vars,
        automation_flags,
        guild_dialog_defaults,
    }
}

impl KnownProfileSettings {
    fn from_user_settings(settings: &UserSettings) -> Self {
        Self {
            rig: setting_value(settings, RIG_KEY),
            tzarakk_mount: setting_value(settings, TZARAKK_MOUNT_KEY),
            sabre_weapon: setting_value(settings, SABRE_WEAPON_KEY),
            riftwalker_entity_labels: [
                riftwalker_entity_label(settings, RIFTWALKER_ENTITY_FIRE_KEY),
                riftwalker_entity_label(settings, RIFTWALKER_ENTITY_AIR_KEY),
                riftwalker_entity_label(settings, RIFTWALKER_ENTITY_WATER_KEY),
                riftwalker_entity_label(settings, RIFTWALKER_ENTITY_EARTH_KEY),
            ],
            is_lich: settings.is_lich_enabled(),
        }
    }
}

impl GuildDialogProfileDefaults {
    fn from_settings(primary_background: &str, settings: &KnownProfileSettings) -> Self {
        Self {
            primary_background: primary_background.to_string(),
            tzarakk_mount: settings.tzarakk_mount.clone(),
            sabre_weapon: settings.sabre_weapon.clone(),
            riftwalker_entity_labels: settings.riftwalker_entity_labels.clone(),
        }
    }
}

fn automation_vars_for_settings(settings: &KnownProfileSettings) -> Vec<(String, String)> {
    let mut vars = vec![
        (RIG_KEY.to_string(), settings.rig.clone()),
        (
            TZARAKK_MOUNT_KEY.to_string(),
            settings.tzarakk_mount.clone(),
        ),
    ];
    vars.push((SABRE_WEAPON_KEY.to_string(), settings.sabre_weapon.clone()));
    vars.extend(
        RIFTWALKER_ENTITY_LABEL_KEYS
            .into_iter()
            .zip(settings.riftwalker_entity_labels.iter())
            .map(|(key, value)| (key.to_string(), value.clone())),
    );
    vars
}

fn setting_value(settings: &UserSettings, key: &str) -> String {
    settings.get(key).unwrap_or_default().to_string()
}

fn riftwalker_entity_label(settings: &UserSettings, key: &str) -> String {
    let raw = settings.get(key).unwrap_or_default();
    if raw.is_empty() {
        DEFAULT_RIFTWALKER_ENTITY_LABEL.to_string()
    } else {
        raw.to_string()
    }
}

fn filter_known_guilds(keys: Vec<String>) -> Vec<String> {
    let definitions = guild_definitions();
    keys.into_iter()
        .filter(|key| definitions.iter().any(|definition| definition.key == key))
        .collect()
}

fn non_empty(value: &str) -> Option<&str> {
    (!value.is_empty()).then_some(value)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SettingEntry;

    fn settings(entries: &[(&str, &str)]) -> UserSettings {
        UserSettings {
            entries: entries
                .iter()
                .map(|(key, value)| SettingEntry {
                    key: (*key).to_string(),
                    value: (*value).to_string(),
                })
                .collect(),
        }
    }

    #[test]
    fn profile_uses_defaults_for_missing_settings() {
        let profile = runtime_profile(
            None,
            None,
            UserSettings::default(),
            GenericCommandsConfig::default(),
        );

        assert_eq!(profile.selected_guild_keys, None);
        assert_eq!(
            profile.guild_primary_background,
            DEFAULT_GUILD_PRIMARY_KEYWORD
        );
        assert_eq!(
            profile.settings.riftwalker_entity_labels,
            default_riftwalker_entity_labels()
        );
        assert_eq!(
            profile.guild_dialog_defaults.riftwalker_entity_labels,
            default_riftwalker_entity_labels()
        );
        assert_eq!(
            profile.automation_flags,
            vec![(IS_LICH_KEY.to_string(), false)]
        );
    }

    #[test]
    fn profile_extracts_known_settings() {
        let profile = runtime_profile(
            Some(vec!["animist".to_string(), "missing".to_string()]),
            Some("magical"),
            settings(&[
                (RIG_KEY, "bag"),
                (TZARAKK_MOUNT_KEY, "Vedir"),
                (SABRE_WEAPON_KEY, "sabre"),
                (RIFTWALKER_ENTITY_FIRE_KEY, "flame"),
                (RIFTWALKER_ENTITY_AIR_KEY, "wind"),
                (RIFTWALKER_ENTITY_WATER_KEY, "wave"),
                (RIFTWALKER_ENTITY_EARTH_KEY, "stone"),
                (IS_LICH_KEY, "true"),
            ]),
            GenericCommandsConfig::default(),
        );

        assert_eq!(
            profile.selected_guild_keys,
            Some(vec!["animist".to_string()])
        );
        assert_eq!(profile.guild_primary_background, "magical");
        assert_eq!(profile.settings.rig, "bag");
        assert_eq!(profile.settings.tzarakk_mount, "Vedir");
        assert_eq!(profile.settings.sabre_weapon, "sabre");
        assert_eq!(profile.guild_dialog_defaults.primary_background, "magical");
        assert_eq!(profile.guild_dialog_defaults.tzarakk_mount, "Vedir");
        assert_eq!(profile.guild_dialog_defaults.sabre_weapon, "sabre");
        assert_eq!(
            profile.settings.riftwalker_entity_labels,
            [
                "flame".to_string(),
                "wind".to_string(),
                "wave".to_string(),
                "stone".to_string()
            ]
        );
        assert!(profile.settings.is_lich);
    }

    #[test]
    fn empty_riftwalker_labels_become_entity() {
        let profile = runtime_profile(
            None,
            None,
            settings(&[
                (RIFTWALKER_ENTITY_FIRE_KEY, ""),
                (RIFTWALKER_ENTITY_AIR_KEY, "air"),
                (RIFTWALKER_ENTITY_WATER_KEY, ""),
                (RIFTWALKER_ENTITY_EARTH_KEY, "earth"),
            ]),
            GenericCommandsConfig::default(),
        );

        assert_eq!(
            profile.settings.riftwalker_entity_labels,
            [
                "entity".to_string(),
                "air".to_string(),
                "entity".to_string(),
                "earth".to_string()
            ]
        );
    }

    #[test]
    fn profile_preserves_generic_command_config() {
        let generic_commands_config = GenericCommandsConfig {
            enabled_groups: vec!["common_spells".to_string()],
            disabled_commands: vec!["cinv".to_string()],
        };

        let profile = runtime_profile(
            None,
            None,
            UserSettings::default(),
            generic_commands_config.clone(),
        );

        assert_eq!(profile.generic_commands_config, generic_commands_config);
    }
}
