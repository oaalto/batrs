pub mod automation;
pub mod normalization;
pub mod persistence;
pub mod registry;
pub mod runtime;

pub use registry::{RIFTWALKER_ENTITY_LABEL_KEYS, SABRE_WEAPON_KEY, TZARAKK_MOUNT_KEY};
pub use runtime::{InterpretedPlayerProfile, PlayerRuntimeProfile};

use crate::config::{PlayerToml, SettingEntry, SettingsTable, UserSettings};
use crate::player_profile::normalization::{normalize_player_guilds, normalize_settings_entries};
use crate::player_profile::registry::{SETTINGS_DEFS, definition_for_key};

pub fn interpret_player_toml(player: PlayerToml) -> InterpretedPlayerProfile {
    let mut normalized_player = player;
    let changed = normalize_player_toml(&mut normalized_player);
    let settings = user_settings_from_player(&normalized_player);
    let guild_primary_background = normalized_player
        .guild_primary_background
        .as_deref()
        .unwrap_or(crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD);
    let runtime = runtime::runtime_profile_from_parts(
        normalized_player.guilds.clone().unwrap_or_default(),
        guild_primary_background,
        settings,
        normalized_player.generic_commands.clone(),
        normalized_player.triggers.clone(),
    );

    InterpretedPlayerProfile {
        normalized_player,
        changed,
        runtime,
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
            value: definition.persist.read(&player.settings, definition.key),
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

fn settings_table_from_normalized_entries(entries: &[SettingEntry]) -> SettingsTable {
    let mut table = SettingsTable::default();
    for entry in entries {
        if let Some(definition) = definition_for_key(&entry.key) {
            definition.persist.write(
                &mut table,
                definition.key,
                entry.value.clone(),
                definition.sparse_when_default,
            );
        } else {
            table.extra.insert(entry.key.clone(), entry.value.clone());
        }
    }
    table
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SettingEntry;
    use crate::player_profile::registry::{
        IS_LICH_KEY, RIFTWALKER_ENTITY_AIR_KEY, RIFTWALKER_ENTITY_EARTH_KEY,
        RIFTWALKER_ENTITY_FIRE_KEY, RIFTWALKER_ENTITY_WATER_KEY, RIG_KEY, SettingKind, SettingSlot,
        TZARAKK_MOUNT_KEY,
    };

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
        let profile = runtime::runtime_profile_from_parts(
            Vec::new(),
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
            UserSettings::default(),
            crate::config::GenericCommandsConfig::default(),
            crate::triggers::TriggerConfig::default(),
        );

        assert_eq!(
            profile.guild_selection.persisted_keys(),
            Vec::<String>::new()
        );
        assert_eq!(
            profile.guild_primary_background,
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD
        );
        assert_eq!(
            profile.settings.riftwalker_entity_labels,
            runtime::default_riftwalker_entity_labels()
        );
        assert_eq!(
            profile.guild_dialog_defaults.riftwalker_entity_labels,
            runtime::default_riftwalker_entity_labels()
        );
        assert_eq!(
            profile.automation_flags,
            vec![(IS_LICH_KEY.to_string(), false)]
        );
    }

    #[test]
    fn profile_extracts_known_settings() {
        let profile = runtime::runtime_profile_from_parts(
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
            crate::config::GenericCommandsConfig::default(),
            crate::triggers::TriggerConfig::default(),
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
        let profile = runtime::runtime_profile_from_parts(
            Vec::new(),
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
            settings(&[
                (RIFTWALKER_ENTITY_FIRE_KEY, ""),
                (RIFTWALKER_ENTITY_AIR_KEY, "air"),
                (RIFTWALKER_ENTITY_WATER_KEY, ""),
                (RIFTWALKER_ENTITY_EARTH_KEY, "earth"),
            ]),
            crate::config::GenericCommandsConfig::default(),
            crate::triggers::TriggerConfig::default(),
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
        let generic_commands_config = crate::config::GenericCommandsConfig {
            enabled_groups: vec!["common_spells".to_string()],
            disabled_commands: vec!["cinv".to_string()],
        };

        let profile = runtime::runtime_profile_from_parts(
            Vec::new(),
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
            UserSettings::default(),
            generic_commands_config.clone(),
            crate::triggers::TriggerConfig::default(),
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
            slots.push(definition.slot);
            assert!(definition_for_key(definition.key).is_some());
        }
        slots.sort_by_key(|slot| match slot {
            SettingSlot::Rig => 0,
            SettingSlot::TzarakkMount => 1,
            SettingSlot::SabreWeapon => 2,
            SettingSlot::RiftwalkerEntity(index) => 3 + index,
            SettingSlot::IsLich => 7,
        });
        assert_eq!(slots.len(), 8);
        assert_eq!(slots, {
            let mut expected = vec![
                SettingSlot::Rig,
                SettingSlot::TzarakkMount,
                SettingSlot::SabreWeapon,
            ];
            expected.extend((0..4).map(SettingSlot::RiftwalkerEntity));
            expected.push(SettingSlot::IsLich);
            expected
        });

        let entries = normalize_settings_entries(Vec::new()).0;
        assert_eq!(entries.len(), 8);
        for definition in SETTINGS_DEFS {
            assert!(
                entries.iter().any(|entry| entry.key == definition.key),
                "missing normalized entry for {}",
                definition.key
            );
        }
    }

    #[test]
    fn registry_dispatch_round_trips_persist_and_slot() {
        for definition in SETTINGS_DEFS {
            let (persist_sentinel, expected_persist) = if definition.sparse_when_default {
                ("yes".to_string(), "yes".to_string())
            } else {
                let value = format!("rt-{}", definition.key);
                (value.clone(), value)
            };
            let mut table = SettingsTable::default();
            definition.persist.write(
                &mut table,
                definition.key,
                persist_sentinel,
                definition.sparse_when_default,
            );
            assert_eq!(
                definition.persist.read(&table, definition.key),
                expected_persist,
                "persist round-trip failed for {}",
                definition.key
            );

            let (write_value, expected_read) = match definition.kind {
                SettingKind::Bool => ("yes".to_string(), "true".to_string()),
                SettingKind::String => {
                    let value = format!("slot-{}", definition.key);
                    (value.clone(), value)
                }
            };
            let mut known = runtime::runtime_profile_from_parts(
                Vec::new(),
                crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
                UserSettings::default(),
                crate::config::GenericCommandsConfig::default(),
                crate::triggers::TriggerConfig::default(),
            )
            .settings;
            definition.slot.write(&mut known, write_value);
            assert_eq!(
                definition.slot.read(&known),
                expected_read,
                "slot round-trip failed for {}",
                definition.key
            );
        }
    }

    #[test]
    fn is_lich_absent_is_false_and_omitted_from_extra() {
        let player = PlayerToml::default();
        let interpreted = interpret_player_toml(player);

        assert!(!interpreted.runtime.settings.is_lich);
        assert!(
            !interpreted
                .normalized_player
                .settings
                .extra
                .contains_key(IS_LICH_KEY)
        );
        assert!(
            settings_entries_for_editor(&interpreted.normalized_player)
                .iter()
                .any(|entry| entry.key == IS_LICH_KEY && entry.value.is_empty())
        );
    }

    #[test]
    fn is_lich_truthy_values_persist_in_extra() {
        for value in ["yes", "true", "1", "TRUE", "Yes"] {
            let player = PlayerToml {
                settings: crate::config::SettingsTable {
                    extra: std::collections::HashMap::from([(
                        IS_LICH_KEY.to_string(),
                        value.to_string(),
                    )]),
                    ..Default::default()
                },
                ..Default::default()
            };

            let interpreted = interpret_player_toml(player);

            assert!(
                interpreted.runtime.settings.is_lich,
                "expected truthy for {value}"
            );
            assert_eq!(
                interpreted
                    .normalized_player
                    .settings
                    .extra
                    .get(IS_LICH_KEY)
                    .map(String::as_str),
                Some(value)
            );
        }
    }

    #[test]
    fn is_lich_explicit_false_dropped_from_extra_on_normalize() {
        let player = PlayerToml {
            settings: crate::config::SettingsTable {
                extra: std::collections::HashMap::from([(
                    IS_LICH_KEY.to_string(),
                    "false".to_string(),
                )]),
                ..Default::default()
            },
            ..Default::default()
        };

        let interpreted = interpret_player_toml(player);

        assert!(interpreted.changed);
        assert!(!interpreted.runtime.settings.is_lich);
        assert!(
            !interpreted
                .normalized_player
                .settings
                .extra
                .contains_key(IS_LICH_KEY)
        );
    }

    #[test]
    fn guild_dialog_defaults_follow_registry_flags() {
        let profile = runtime::runtime_profile_from_parts(
            Vec::new(),
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
            settings(&[(RIG_KEY, "bag")]),
            crate::config::GenericCommandsConfig::default(),
            crate::triggers::TriggerConfig::default(),
        );

        assert_eq!(profile.settings.rig, "bag");
        assert_eq!(profile.guild_dialog_defaults.tzarakk_mount, "");
        assert_eq!(profile.guild_dialog_defaults.sabre_weapon, "");
    }

    #[test]
    fn unknown_settings_preserved_in_extra_round_trip() {
        let player = PlayerToml {
            settings: crate::config::SettingsTable {
                extra: std::collections::HashMap::from([(
                    "custom_flag".to_string(),
                    "on".to_string(),
                )]),
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
    fn automation_exports_built_from_registry() {
        let profile = runtime::runtime_profile_from_parts(
            Vec::new(),
            crate::guilds::catalog::DEFAULT_GUILD_PRIMARY_KEYWORD,
            settings(&[
                (RIG_KEY, "bag"),
                (TZARAKK_MOUNT_KEY, "Vedir"),
                (SABRE_WEAPON_KEY, "sabre"),
                (RIFTWALKER_ENTITY_FIRE_KEY, "flame"),
                (IS_LICH_KEY, "yes"),
            ]),
            crate::config::GenericCommandsConfig::default(),
            crate::triggers::TriggerConfig::default(),
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
        assert_eq!(
            profile.automation_flags,
            vec![(IS_LICH_KEY.to_string(), true)]
        );
    }
}
