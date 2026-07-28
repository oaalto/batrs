use crate::config::{SettingsTable, is_truthy_setting_value};
use crate::player_profile::registry::{PersistSlot, SettingDefinition};

pub fn read_persist(table: &SettingsTable, definition: &SettingDefinition) -> String {
    match definition.persist {
        PersistSlot::Rig => table.rig.clone(),
        PersistSlot::TzarakkMount => table.tzarakk_mount.clone(),
        PersistSlot::SabreWeapon => table.sabre_weapon.clone(),
        PersistSlot::RiftwalkerEntityFire => table.riftwalker_entity_fire.clone(),
        PersistSlot::RiftwalkerEntityAir => table.riftwalker_entity_air.clone(),
        PersistSlot::RiftwalkerEntityWater => table.riftwalker_entity_water.clone(),
        PersistSlot::RiftwalkerEntityEarth => table.riftwalker_entity_earth.clone(),
        PersistSlot::Extra => table.extra.get(definition.key).cloned().unwrap_or_default(),
    }
}

pub fn write_persist(table: &mut SettingsTable, definition: &SettingDefinition, value: String) {
    match definition.persist {
        PersistSlot::Rig => table.rig = value,
        PersistSlot::TzarakkMount => table.tzarakk_mount = value,
        PersistSlot::SabreWeapon => table.sabre_weapon = value,
        PersistSlot::RiftwalkerEntityFire => table.riftwalker_entity_fire = value,
        PersistSlot::RiftwalkerEntityAir => table.riftwalker_entity_air = value,
        PersistSlot::RiftwalkerEntityWater => table.riftwalker_entity_water = value,
        PersistSlot::RiftwalkerEntityEarth => table.riftwalker_entity_earth = value,
        PersistSlot::Extra => {
            if definition.sparse_when_default && !is_truthy_setting_value(&value) {
                table.extra.remove(definition.key);
            } else {
                table.extra.insert(definition.key.to_string(), value);
            }
        }
    }
}
