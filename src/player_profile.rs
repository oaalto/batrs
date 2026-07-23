use crate::config::{GenericCommandsConfig, PlayerToml, SettingEntry, SettingsTable, UserSettings};
use crate::guilds::catalog::{DEFAULT_GUILD_PRIMARY_KEYWORD, GuildSelection};
use std::collections::HashMap;

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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingKind {
    String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SettingSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntity(usize),
}

#[derive(Clone, Copy)]
enum PersistSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntityFire,
    RiftwalkerEntityAir,
    RiftwalkerEntityWater,
    RiftwalkerEntityEarth,
}

#[derive(Clone, Copy)]
enum AutomationExport {
    Var,
}

struct SettingDefinition {
    key: &'static str,
    default: &'static str,
    kind: SettingKind,
    slot: SettingSlot,
    persist: PersistSlot,
    guild_dialog: bool,
    automation_export: AutomationExport,
}

const SETTINGS_DEFS: &[SettingDefinition] = &[
    SettingDefinition {
        key: RIG_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::Rig,
        persist: PersistSlot::Rig,
        guild_dialog: false,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: TZARAKK_MOUNT_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::TzarakkMount,
        persist: PersistSlot::TzarakkMount,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: SABRE_WEAPON_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::SabreWeapon,
        persist: PersistSlot::SabreWeapon,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_FIRE_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(0),
        persist: PersistSlot::RiftwalkerEntityFire,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_AIR_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(1),
        persist: PersistSlot::RiftwalkerEntityAir,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_WATER_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(2),
        persist: PersistSlot::RiftwalkerEntityWater,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_EARTH_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(3),
        persist: PersistSlot::RiftwalkerEntityEarth,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
];

fn definition_for_key(key: &str) -> Option<&'static SettingDefinition> {
    SETTINGS_DEFS
        .iter()
        .find(|definition| definition.key == key)
}

fn read_persist(table: &SettingsTable, slot: PersistSlot) -> &str {
    match slot {
        PersistSlot::Rig => &table.rig,
        PersistSlot::TzarakkMount => &table.tzarakk_mount,
        PersistSlot::SabreWeapon => &table.sabre_weapon,
        PersistSlot::RiftwalkerEntityFire => &table.riftwalker_entity_fire,
        PersistSlot::RiftwalkerEntityAir => &table.riftwalker_entity_air,
        PersistSlot::RiftwalkerEntityWater => &table.riftwalker_entity_water,
        PersistSlot::RiftwalkerEntityEarth => &table.riftwalker_entity_earth,
    }
}

fn write_persist(table: &mut SettingsTable, slot: PersistSlot, value: String) {
    match slot {
        PersistSlot::Rig => table.rig = value,
        PersistSlot::TzarakkMount => table.tzarakk_mount = value,
        PersistSlot::SabreWeapon => table.sabre_weapon = value,
        PersistSlot::RiftwalkerEntityFire => table.riftwalker_entity_fire = value,
        PersistSlot::RiftwalkerEntityAir => table.riftwalker_entity_air = value,
        PersistSlot::RiftwalkerEntityWater => table.riftwalker_entity_water = value,
        PersistSlot::RiftwalkerEntityEarth => table.riftwalker_entity_earth = value,
    }
}

fn read_known_slot(settings: &KnownProfileSettings, slot: SettingSlot) -> String {
    match slot {
        SettingSlot::Rig => settings.rig.clone(),
        SettingSlot::TzarakkMount => settings.tzarakk_mount.clone(),
        SettingSlot::SabreWeapon => settings.sabre_weapon.clone(),
        SettingSlot::RiftwalkerEntity(index) => settings.riftwalker_entity_labels[index].clone(),
    }
}

fn write_known_slot(settings: &mut KnownProfileSettings, slot: SettingSlot, value: String) {
    match slot {
        SettingSlot::Rig => settings.rig = value,
        SettingSlot::TzarakkMount => settings.tzarakk_mount = value,
        SettingSlot::SabreWeapon => settings.sabre_weapon = value,
        SettingSlot::RiftwalkerEntity(index) => settings.riftwalker_entity_labels[index] = value,
    }
}

fn write_guild_dialog_slot(
    defaults: &mut GuildDialogProfileDefaults,
    slot: SettingSlot,
    value: String,
) {
    match slot {
        SettingSlot::TzarakkMount => defaults.tzarakk_mount = value,
        SettingSlot::SabreWeapon => defaults.sabre_weapon = value,
        SettingSlot::RiftwalkerEntity(index) => defaults.riftwalker_entity_labels[index] = value,
        SettingSlot::Rig => {}
    }
}

fn normalized_string_value(definition: &SettingDefinition, raw: String) -> String {
    match definition.kind {
        SettingKind::String => match definition.slot {
            SettingSlot::RiftwalkerEntity(_) if raw.is_empty() => definition.default.to_string(),
            _ => raw,
        },
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlayerRuntimeProfile {
    pub guild_selection: GuildSelection,
    pub guild_primary_background: String,
    pub generic_commands_config: GenericCommandsConfig,
    pub settings: KnownProfileSettings,
    pub automation_vars: Vec<(String, String)>,
    pub automation_flags: Vec<(String, bool)>,
    pub guild_dialog_defaults: GuildDialogProfileDefaults,
}

pub struct InterpretedPlayerProfile {
    pub normalized_player: PlayerToml,
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

fn default_riftwalker_entity_labels() -> [String; 4] {
    std::array::from_fn(|_| DEFAULT_RIFTWALKER_ENTITY_LABEL.to_string())
}

pub fn interpret_player_toml(player: PlayerToml) -> InterpretedPlayerProfile {
    let mut normalized_player = player;
    let changed = normalize_player_toml(&mut normalized_player);
    let settings = user_settings_from_player(&normalized_player);
    let guild_primary_background = normalized_player
        .guild_primary_background
        .as_deref()
        .unwrap_or(DEFAULT_GUILD_PRIMARY_KEYWORD);
    let runtime = runtime_profile_from_parts(
        normalized_player.guilds.clone().unwrap_or_default(),
        guild_primary_background,
        settings,
        normalized_player.generic_commands.clone(),
    );

    InterpretedPlayerProfile {
        normalized_player,
        changed,
        runtime,
    }
}

fn runtime_profile_from_parts(
    selected_guild_keys: Vec<String>,
    guild_primary_background: &str,
    settings: UserSettings,
    generic_commands_config: GenericCommandsConfig,
) -> PlayerRuntimeProfile {
    let known_settings = KnownProfileSettings::from_user_settings(&settings);
    let guild_selection =
        GuildSelection::from_persisted_keys(&selected_guild_keys, Some(guild_primary_background));
    let guild_primary_background = guild_selection.primary_background_keyword().to_string();
    let automation_vars = automation_vars_for_settings(&known_settings);
    let automation_flags = vec![(IS_LICH_KEY.to_string(), known_settings.is_lich)];
    let guild_dialog_defaults =
        GuildDialogProfileDefaults::from_settings(&guild_primary_background, &known_settings);

    PlayerRuntimeProfile {
        guild_selection,
        guild_primary_background,
        generic_commands_config,
        settings: known_settings,
        automation_vars,
        automation_flags,
        guild_dialog_defaults,
    }
}

pub fn settings_entries_for_editor(player: &PlayerToml) -> Vec<SettingEntry> {
    user_settings_from_player(player).entries
}

pub fn settings_table_from_entries(entries: &[SettingEntry]) -> SettingsTable {
    let (normalized, _) = normalize_settings_entries(entries.to_vec());
    settings_table_from_normalized_entries(&normalized)
}

pub fn user_settings_from_player(player: &PlayerToml) -> UserSettings {
    let mut entries = SETTINGS_DEFS
        .iter()
        .map(|definition| SettingEntry {
            key: definition.key.to_string(),
            value: read_persist(&player.settings, definition.persist).to_string(),
        })
        .collect::<Vec<_>>();
    let mut keys: Vec<String> = player.settings.extra.keys().cloned().collect();
    keys.sort();
    for key in keys {
        if definition_for_key(&key).is_none()
            && let Some(value) = player.settings.extra.get(&key)
        {
            entries.push(SettingEntry {
                key,
                value: value.clone(),
            });
        }
    }
    UserSettings { entries }
}

fn normalize_player_toml(player: &mut PlayerToml) -> bool {
    let entries = user_settings_from_player(player).entries;
    let (normalized, settings_changed) = normalize_settings_entries(entries);
    let guild_changed = normalize_player_guilds(player);
    if settings_changed {
        player.settings = settings_table_from_normalized_entries(&normalized);
    }
    settings_changed || guild_changed
}

fn normalize_settings_entries(entries: Vec<SettingEntry>) -> (Vec<SettingEntry>, bool) {
    let mut known = HashMap::new();
    let mut extras = Vec::new();
    for entry in entries {
        if definition_for_key(&entry.key).is_some() {
            known.insert(entry.key, entry.value);
        } else {
            extras.push(entry);
        }
    }

    let mut changed = false;
    let mut normalized = Vec::new();
    for definition in SETTINGS_DEFS {
        if let Some(raw) = known.remove(definition.key) {
            let value = normalized_string_value(definition, raw.clone());
            if value != raw {
                changed = true;
            }
            normalized.push(SettingEntry {
                key: definition.key.to_string(),
                value,
            });
        } else {
            normalized.push(SettingEntry {
                key: definition.key.to_string(),
                value: definition.default.to_string(),
            });
            changed = true;
        }
    }
    normalized.extend(extras);
    (normalized, changed)
}

fn settings_table_from_normalized_entries(entries: &[SettingEntry]) -> SettingsTable {
    let mut table = SettingsTable::default();
    for entry in entries {
        if let Some(definition) = definition_for_key(&entry.key) {
            write_persist(&mut table, definition.persist, entry.value.clone());
        } else {
            table.extra.insert(entry.key.clone(), entry.value.clone());
        }
    }
    table
}

fn normalize_player_guilds(player: &mut PlayerToml) -> bool {
    let selection = GuildSelection::from_persisted_keys(
        &player.guilds.clone().unwrap_or_default(),
        player.guild_primary_background.as_deref(),
    );
    let normalized_guilds = selection.persisted_keys_option();
    let normalized_primary = selection.primary_background_keyword().to_string();
    let changed = player.guilds != normalized_guilds
        || player.guild_primary_background.as_deref() != Some(normalized_primary.as_str());

    player.guilds = normalized_guilds;
    player.guild_primary_background = Some(normalized_primary);

    changed
}

impl KnownProfileSettings {
    fn from_user_settings(settings: &UserSettings) -> Self {
        let mut known = Self {
            rig: String::new(),
            tzarakk_mount: String::new(),
            sabre_weapon: String::new(),
            riftwalker_entity_labels: default_riftwalker_entity_labels(),
            is_lich: settings.is_lich_enabled(),
        };
        for definition in SETTINGS_DEFS {
            let raw = setting_value(settings, definition.key);
            let value = normalized_string_value(definition, raw);
            write_known_slot(&mut known, definition.slot, value);
        }
        known
    }
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
            write_guild_dialog_slot(
                &mut defaults,
                definition.slot,
                read_known_slot(settings, definition.slot),
            );
        }
        defaults
    }
}

fn automation_vars_for_settings(settings: &KnownProfileSettings) -> Vec<(String, String)> {
    SETTINGS_DEFS
        .iter()
        .filter(|definition| matches!(definition.automation_export, AutomationExport::Var))
        .map(|definition| {
            (
                definition.key.to_string(),
                read_known_slot(settings, definition.slot),
            )
        })
        .collect()
}

fn setting_value(settings: &UserSettings, key: &str) -> String {
    settings.get(key).unwrap_or_default().to_string()
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
        let profile = runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            UserSettings::default(),
            GenericCommandsConfig::default(),
        );

        assert_eq!(
            profile.guild_selection.persisted_keys(),
            Vec::<String>::new()
        );
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
        let profile = runtime_profile_from_parts(
            vec!["animist".to_string(), "missing".to_string()],
            "magical",
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
            profile.guild_selection.persisted_keys(),
            vec!["animist".to_string()]
        );
        assert_eq!(profile.guild_primary_background, "good_religious");
        assert_eq!(profile.settings.rig, "bag");
        assert_eq!(profile.settings.tzarakk_mount, "Vedir");
        assert_eq!(profile.settings.sabre_weapon, "sabre");
        assert_eq!(
            profile.guild_dialog_defaults.primary_background,
            "good_religious"
        );
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
        let profile = runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
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

        let profile = runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            UserSettings::default(),
            generic_commands_config.clone(),
        );

        assert_eq!(profile.generic_commands_config, generic_commands_config);
    }

    #[test]
    fn interpret_player_toml_filters_unimplemented_and_unknown_guilds() {
        let player = PlayerToml {
            guilds: Some(vec![
                "animist".to_string(),
                "alchemists".to_string(),
                "missing".to_string(),
            ]),
            ..Default::default()
        };

        let interpreted = interpret_player_toml(player);

        assert!(interpreted.changed);
        assert_eq!(
            interpreted.normalized_player.guilds,
            Some(vec!["animist".to_string()])
        );
        assert_eq!(
            interpreted.runtime.guild_selection.persisted_keys(),
            vec!["animist".to_string()]
        );
    }

    #[test]
    fn interpret_player_toml_normalizes_settings_without_runtime_editor_entries() {
        let player = PlayerToml {
            settings: settings_table_from_normalized_entries(
                &settings(&[
                    (RIG_KEY, "bag"),
                    (RIFTWALKER_ENTITY_FIRE_KEY, ""),
                    (IS_LICH_KEY, "yes"),
                ])
                .entries,
            ),
            ..Default::default()
        };

        let interpreted = interpret_player_toml(player);

        assert!(interpreted.changed);
        assert_eq!(
            interpreted
                .normalized_player
                .settings
                .riftwalker_entity_fire,
            "entity"
        );
        assert_eq!(interpreted.runtime.settings.rig, "bag");
        assert!(interpreted.runtime.settings.is_lich);
    }

    #[test]
    fn registry_rows_are_complete_and_unique() {
        let mut slots = Vec::new();
        for definition in SETTINGS_DEFS {
            assert_eq!(definition.kind, SettingKind::String);
            slots.push(definition.slot);
            assert!(definition_for_key(definition.key).is_some());
        }
        slots.sort_by_key(|slot| match slot {
            SettingSlot::Rig => 0,
            SettingSlot::TzarakkMount => 1,
            SettingSlot::SabreWeapon => 2,
            SettingSlot::RiftwalkerEntity(index) => 3 + index,
        });
        assert_eq!(slots.len(), 7);
        assert_eq!(slots, {
            let mut expected = vec![
                SettingSlot::Rig,
                SettingSlot::TzarakkMount,
                SettingSlot::SabreWeapon,
            ];
            expected.extend((0..4).map(SettingSlot::RiftwalkerEntity));
            expected
        });
    }

    #[test]
    fn guild_dialog_defaults_follow_registry_flags() {
        let profile = runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            settings(&[(RIG_KEY, "bag")]),
            GenericCommandsConfig::default(),
        );

        assert_eq!(profile.settings.rig, "bag");
        assert_eq!(profile.guild_dialog_defaults.tzarakk_mount, "");
        assert_eq!(profile.guild_dialog_defaults.sabre_weapon, "");
    }

    #[test]
    fn unknown_settings_preserved_in_extra_round_trip() {
        let player = PlayerToml {
            settings: SettingsTable {
                extra: HashMap::from([("custom_flag".to_string(), "on".to_string())]),
                ..Default::default()
            },
            ..Default::default()
        };

        let interpreted = interpret_player_toml(player);
        assert_eq!(
            interpreted
                .normalized_player
                .settings
                .extra
                .get("custom_flag"),
            Some(&"on".to_string())
        );
    }

    #[test]
    fn automation_vars_built_from_registry() {
        let profile = runtime_profile_from_parts(
            Vec::new(),
            DEFAULT_GUILD_PRIMARY_KEYWORD,
            settings(&[
                (RIG_KEY, "bag"),
                (TZARAKK_MOUNT_KEY, "Vedir"),
                (SABRE_WEAPON_KEY, "sabre"),
                (RIFTWALKER_ENTITY_FIRE_KEY, "flame"),
            ]),
            GenericCommandsConfig::default(),
        );

        assert_eq!(
            profile.automation_vars,
            vec![
                (RIG_KEY.to_string(), "bag".to_string()),
                (TZARAKK_MOUNT_KEY.to_string(), "Vedir".to_string()),
                (SABRE_WEAPON_KEY.to_string(), "sabre".to_string()),
                (RIFTWALKER_ENTITY_FIRE_KEY.to_string(), "flame".to_string()),
                (RIFTWALKER_ENTITY_AIR_KEY.to_string(), "entity".to_string()),
                (
                    RIFTWALKER_ENTITY_WATER_KEY.to_string(),
                    "entity".to_string()
                ),
                (
                    RIFTWALKER_ENTITY_EARTH_KEY.to_string(),
                    "entity".to_string()
                ),
            ]
        );
    }
}
