use crate::config::{SettingsTable, is_truthy_setting_value};
use crate::player_profile::registry::{PersistSlot, SettingSlot};
use crate::player_profile::runtime::{GuildDialogProfileDefaults, KnownProfileSettings};

impl PersistSlot {
    pub(crate) fn read(&self, table: &SettingsTable, key: &str) -> String {
        match self {
            PersistSlot::Rig => table.rig.clone(),
            PersistSlot::TzarakkMount => table.tzarakk_mount.clone(),
            PersistSlot::SabreWeapon => table.sabre_weapon.clone(),
            PersistSlot::RiftwalkerEntityFire => table.riftwalker_entity_fire.clone(),
            PersistSlot::RiftwalkerEntityAir => table.riftwalker_entity_air.clone(),
            PersistSlot::RiftwalkerEntityWater => table.riftwalker_entity_water.clone(),
            PersistSlot::RiftwalkerEntityEarth => table.riftwalker_entity_earth.clone(),
            PersistSlot::Extra => table.extra.get(key).cloned().unwrap_or_default(),
        }
    }

    pub(crate) fn write(
        &self,
        table: &mut SettingsTable,
        key: &str,
        value: String,
        sparse_when_default: bool,
    ) {
        match self {
            PersistSlot::Rig => table.rig = value,
            PersistSlot::TzarakkMount => table.tzarakk_mount = value,
            PersistSlot::SabreWeapon => table.sabre_weapon = value,
            PersistSlot::RiftwalkerEntityFire => table.riftwalker_entity_fire = value,
            PersistSlot::RiftwalkerEntityAir => table.riftwalker_entity_air = value,
            PersistSlot::RiftwalkerEntityWater => table.riftwalker_entity_water = value,
            PersistSlot::RiftwalkerEntityEarth => table.riftwalker_entity_earth = value,
            PersistSlot::Extra => {
                if sparse_when_default && !is_truthy_setting_value(&value) {
                    table.extra.remove(key);
                } else {
                    table.extra.insert(key.to_string(), value);
                }
            }
        }
    }
}

impl SettingSlot {
    pub(crate) fn read(&self, settings: &KnownProfileSettings) -> String {
        match self {
            SettingSlot::Rig => settings.rig.clone(),
            SettingSlot::TzarakkMount => settings.tzarakk_mount.clone(),
            SettingSlot::SabreWeapon => settings.sabre_weapon.clone(),
            SettingSlot::RiftwalkerEntity(index) => {
                settings.riftwalker_entity_labels[*index].clone()
            }
            SettingSlot::IsLich => settings.is_lich.to_string(),
        }
    }

    pub(crate) fn write(&self, settings: &mut KnownProfileSettings, value: String) {
        match self {
            SettingSlot::Rig => settings.rig = value,
            SettingSlot::TzarakkMount => settings.tzarakk_mount = value,
            SettingSlot::SabreWeapon => settings.sabre_weapon = value,
            SettingSlot::RiftwalkerEntity(index) => {
                settings.riftwalker_entity_labels[*index] = value
            }
            SettingSlot::IsLich => settings.is_lich = is_truthy_setting_value(&value),
        }
    }

    pub(crate) fn write_guild_dialog(
        &self,
        defaults: &mut GuildDialogProfileDefaults,
        value: String,
    ) {
        match self {
            SettingSlot::TzarakkMount => defaults.tzarakk_mount = value,
            SettingSlot::SabreWeapon => defaults.sabre_weapon = value,
            SettingSlot::RiftwalkerEntity(index) => {
                defaults.riftwalker_entity_labels[*index] = value
            }
            SettingSlot::Rig | SettingSlot::IsLich => {}
        }
    }
}
