use crate::config::{is_truthy_setting_value, GenericCommandsConfig, UserSettings};
use crate::guilds::catalog::{DEFAULT_GUILD_PRIMARY_KEYWORD, GuildSelection};
use crate::player_profile::automation::{automation_flags_for_settings, automation_vars_for_settings};
use crate::player_profile::registry::{SettingDefinition, SettingKind, SettingSlot, SETTINGS_DEFS};
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
            write_known_slot(&mut known, definition, value);
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
            write_guild_dialog_slot(
                &mut defaults,
                definition.slot,
                read_known_slot(settings, definition.slot),
            );
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

fn write_guild_dialog_slot(
    defaults: &mut GuildDialogProfileDefaults,
    slot: SettingSlot,
    value: String,
) {
    match slot {
        SettingSlot::TzarakkMount => defaults.tzarakk_mount = value,
        SettingSlot::SabreWeapon => defaults.sabre_weapon = value,
        SettingSlot::RiftwalkerEntity(index) => defaults.riftwalker_entity_labels[index] = value,
        SettingSlot::Rig | SettingSlot::IsLich => {}
    }
}

pub(crate) fn read_known_slot(settings: &KnownProfileSettings, slot: SettingSlot) -> String {
    match slot {
        SettingSlot::Rig => settings.rig.clone(),
        SettingSlot::TzarakkMount => settings.tzarakk_mount.clone(),
        SettingSlot::SabreWeapon => settings.sabre_weapon.clone(),
        SettingSlot::RiftwalkerEntity(index) => settings.riftwalker_entity_labels[index].clone(),
        SettingSlot::IsLich => settings.is_lich.to_string(),
    }
}

fn write_known_slot(
    settings: &mut KnownProfileSettings,
    definition: &SettingDefinition,
    value: String,
) {
    match definition.slot {
        SettingSlot::Rig => settings.rig = value,
        SettingSlot::TzarakkMount => settings.tzarakk_mount = value,
        SettingSlot::SabreWeapon => settings.sabre_weapon = value,
        SettingSlot::RiftwalkerEntity(index) => settings.riftwalker_entity_labels[index] = value,
        SettingSlot::IsLich => settings.is_lich = is_truthy_setting_value(&value),
    }
}

pub fn runtime_profile_from_parts(
    selected_guild_keys: Vec<String>,
    guild_primary_background: &str,
    settings: UserSettings,
    generic_commands_config: GenericCommandsConfig,
    trigger_config: TriggerConfig,
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
        )
    }
}
