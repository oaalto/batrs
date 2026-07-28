pub const RIG_KEY: &str = "rig";
pub const TZARAKK_MOUNT_KEY: &str = "tzarakk_mount";
pub const SABRE_WEAPON_KEY: &str = "sabre_weapon";
pub(crate) const RIFTWALKER_ENTITY_FIRE_KEY: &str = "riftwalker_entity_fire";
pub(crate) const RIFTWALKER_ENTITY_AIR_KEY: &str = "riftwalker_entity_air";
pub(crate) const RIFTWALKER_ENTITY_WATER_KEY: &str = "riftwalker_entity_water";
pub(crate) const RIFTWALKER_ENTITY_EARTH_KEY: &str = "riftwalker_entity_earth";
pub(crate) const IS_LICH_KEY: &str = "is_lich";
pub(crate) const DEFAULT_RIFTWALKER_ENTITY_LABEL: &str = "entity";

pub const RIFTWALKER_ENTITY_LABEL_KEYS: [&str; 4] = [
    RIFTWALKER_ENTITY_FIRE_KEY,
    RIFTWALKER_ENTITY_AIR_KEY,
    RIFTWALKER_ENTITY_WATER_KEY,
    RIFTWALKER_ENTITY_EARTH_KEY,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingKind {
    String,
    Bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SettingSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntity(usize),
    IsLich,
}

#[derive(Clone, Copy)]
pub enum PersistSlot {
    Rig,
    TzarakkMount,
    SabreWeapon,
    RiftwalkerEntityFire,
    RiftwalkerEntityAir,
    RiftwalkerEntityWater,
    RiftwalkerEntityEarth,
    Extra,
}

#[derive(Clone, Copy)]
pub enum AutomationExport {
    Var,
    Flag,
}

#[derive(Clone, Copy)]
pub struct SettingDefinition {
    pub key: &'static str,
    pub default: &'static str,
    pub kind: SettingKind,
    pub slot: SettingSlot,
    pub persist: PersistSlot,
    pub sparse_when_default: bool,
    pub guild_dialog: bool,
    pub automation_export: AutomationExport,
}

pub const SETTINGS_DEFS: &[SettingDefinition] = &[
    SettingDefinition {
        key: RIG_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::Rig,
        persist: PersistSlot::Rig,
        sparse_when_default: false,
        guild_dialog: false,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: TZARAKK_MOUNT_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::TzarakkMount,
        persist: PersistSlot::TzarakkMount,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: SABRE_WEAPON_KEY,
        default: "",
        kind: SettingKind::String,
        slot: SettingSlot::SabreWeapon,
        persist: PersistSlot::SabreWeapon,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_FIRE_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(0),
        persist: PersistSlot::RiftwalkerEntityFire,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_AIR_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(1),
        persist: PersistSlot::RiftwalkerEntityAir,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_WATER_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(2),
        persist: PersistSlot::RiftwalkerEntityWater,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: RIFTWALKER_ENTITY_EARTH_KEY,
        default: DEFAULT_RIFTWALKER_ENTITY_LABEL,
        kind: SettingKind::String,
        slot: SettingSlot::RiftwalkerEntity(3),
        persist: PersistSlot::RiftwalkerEntityEarth,
        sparse_when_default: false,
        guild_dialog: true,
        automation_export: AutomationExport::Var,
    },
    SettingDefinition {
        key: IS_LICH_KEY,
        default: "",
        kind: SettingKind::Bool,
        slot: SettingSlot::IsLich,
        persist: PersistSlot::Extra,
        sparse_when_default: true,
        guild_dialog: false,
        automation_export: AutomationExport::Flag,
    },
];

pub fn definition_for_key(key: &str) -> Option<&'static SettingDefinition> {
    SETTINGS_DEFS
        .iter()
        .find(|definition| definition.key == key)
}
