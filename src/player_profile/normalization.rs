use crate::config::{PlayerToml, SettingEntry};
use crate::guilds::catalog::GuildSelection;
use crate::player_profile::registry::{definition_for_key, SettingDefinition, SettingKind, SettingSlot, SETTINGS_DEFS};
use std::collections::HashMap;

pub fn normalize_settings_entries(entries: Vec<SettingEntry>) -> (Vec<SettingEntry>, bool) {
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
            let value = normalized_setting_value(definition, raw.clone());
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

pub fn normalize_player_guilds(player: &mut PlayerToml) -> bool {
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

fn normalized_setting_value(definition: &SettingDefinition, raw: String) -> String {
    match definition.kind {
        SettingKind::String => match definition.slot {
            SettingSlot::RiftwalkerEntity(_) if raw.is_empty() => definition.default.to_string(),
            _ => raw,
        },
        SettingKind::Bool => {
            if crate::config::is_truthy_setting_value(&raw) {
                raw
            } else {
                definition.default.to_string()
            }
        }
    }
}
