use crate::player_profile::registry::{AutomationExport, SETTINGS_DEFS};
use crate::player_profile::runtime::KnownProfileSettings;

pub fn automation_flags_for_settings(settings: &KnownProfileSettings) -> Vec<(String, bool)> {
    SETTINGS_DEFS
        .iter()
        .filter(|definition| matches!(definition.automation_export, AutomationExport::Flag))
        .map(|definition| {
            (
                definition.key.to_string(),
                crate::config::is_truthy_setting_value(&definition.slot.read(settings)),
            )
        })
        .collect()
}

pub fn automation_vars_for_settings(settings: &KnownProfileSettings) -> Vec<(String, String)> {
    SETTINGS_DEFS
        .iter()
        .filter(|definition| matches!(definition.automation_export, AutomationExport::Var))
        .map(|definition| (definition.key.to_string(), definition.slot.read(settings)))
        .collect()
}
