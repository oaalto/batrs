use crate::player_profile::registry::{AutomationExport, SETTINGS_DEFS};
use crate::player_profile::runtime::{read_known_slot, KnownProfileSettings};

pub fn automation_flags_for_settings(settings: &KnownProfileSettings) -> Vec<(String, bool)> {
    SETTINGS_DEFS
        .iter()
        .filter(|definition| matches!(definition.automation_export, AutomationExport::Flag))
        .map(|definition| {
            (
                definition.key.to_string(),
                crate::config::is_truthy_setting_value(&read_known_slot(settings, definition.slot)),
            )
        })
        .collect()
}

pub fn automation_vars_for_settings(settings: &KnownProfileSettings) -> Vec<(String, String)> {
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
