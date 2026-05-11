use crate::command::{Command, CommandContext, Data};
use std::collections::HashMap;

/// An individual generic command with an alias and command template
#[derive(Debug, Clone)]
pub struct GenericCommand {
    pub alias: String,
    pub command: String,
    pub enabled: bool,
}

impl GenericCommand {
    pub fn new(alias: &str, command: &str) -> Self {
        Self {
            alias: alias.to_string(),
            command: command.to_string(),
            enabled: true,
        }
    }
}

/// A group of related generic commands
#[derive(Debug, Clone)]
pub struct GenericCommandGroup {
    pub name: String,
    pub display_name: String,
    pub commands: Vec<GenericCommand>,
}

impl GenericCommandGroup {
    pub fn new(name: &str, display_name: &str, commands: Vec<GenericCommand>) -> Self {
        Self {
            name: name.to_string(),
            display_name: display_name.to_string(),
            commands,
        }
    }

    /// Returns true if all commands are enabled, false if none are, None if mixed
    pub fn selection_state(&self) -> Option<bool> {
        let all_enabled = self.commands.iter().all(|cmd| cmd.enabled);
        let all_disabled = self.commands.iter().all(|cmd| !cmd.enabled);
        if all_enabled {
            Some(true)
        } else if all_disabled {
            Some(false)
        } else {
            None
        }
    }
}

/// Container for all generic command groups
#[derive(Debug, Clone)]
pub struct GenericCommands {
    pub groups: Vec<GenericCommandGroup>,
}

impl Default for GenericCommands {
    fn default() -> Self {
        Self::with_predefined_groups()
    }
}

impl GenericCommands {
    /// Create with all predefined groups enabled
    pub fn with_predefined_groups() -> Self {
        Self {
            groups: vec![
                Self::cure_spells_group(),
                Self::common_spells_group(),
                Self::navigator_group(),
            ],
        }
    }

    /// Returns only enabled commands as a HashMap for command processing
    pub fn commands(&self) -> HashMap<String, Command> {
        let mut result = HashMap::new();
        for group in &self.groups {
            for cmd in &group.commands {
                if cmd.enabled {
                    let handler = Self::get_handler(&cmd.alias);
                    result.insert(cmd.alias.clone(), handler);
                }
            }
        }
        result
    }

    /// Get the handler function for a given command alias
    fn get_handler(alias: &str) -> Command {
        match alias {
            // cure_spells
            "clw" => cast_cure_light_wounds,
            "csw" => cast_cure_serious_wounds,
            "clwf" => cast_cure_light_wounds_full,
            "cswf" => cast_cure_serious_wounds_full,
            // common_spells
            "cww" => cast_water_walking,
            "cinv" => cast_invisibility,
            "cinf" => cast_infravision,
            // navigator
            "ctwe" => cast_teleport_with_error,
            "ctw" => cast_teleport_without_error,
            "cr" => cast_relocate,
            "chw" => cast_heavy_weight,
            _ => unknown_command,
        }
    }

    /// Apply configuration to enable/disable groups and specific commands
    pub fn apply_config(&mut self, enabled_groups: &[String], disabled_commands: &[String]) {
        // If enabled_groups is empty, all groups are enabled by default
        let filter_groups = !enabled_groups.is_empty();

        for group in &mut self.groups {
            // Check if group should be enabled
            let group_enabled = if filter_groups {
                enabled_groups.iter().any(|g| g == &group.name)
            } else {
                true
            };

            for cmd in &mut group.commands {
                // Command is enabled if its group is enabled AND it's not in disabled_commands
                cmd.enabled = group_enabled && !disabled_commands.iter().any(|d| d == &cmd.alias);
            }
        }
    }

    // Predefined command groups based on .tf files

    fn cure_spells_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "cure_spells",
            "Cure Spells",
            vec![
                GenericCommand::new("clw", "@cast 'cure light wounds' {args}"),
                GenericCommand::new("csw", "@cast 'cure serious wounds' {args}"),
                GenericCommand::new("clwf", "@repeat inf cast 'cure light wounds' {args}"),
                GenericCommand::new("cswf", "@repeat inf cast 'cure serious wounds' {args}"),
            ],
        )
    }

    fn common_spells_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "common_spells",
            "Common Spells",
            vec![
                GenericCommand::new("cww", "@cast 'water walking' {args}"),
                GenericCommand::new("cinv", "@cast 'invisibility' {args}"),
                GenericCommand::new("cinf", "@cast 'infravision' {args}"),
            ],
        )
    }

    fn navigator_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "navigator",
            "Navigator",
            vec![
                GenericCommand::new("ctwe", "@cast 'teleport with error'"),
                GenericCommand::new("ctw", "@cast 'teleport without error'"),
                GenericCommand::new("cr", "@cast 'relocate' {args}"),
                GenericCommand::new("chw", "@cast 'heavy weight' {args}"),
            ],
        )
    }
}

// Command handler functions

fn cast_cure_light_wounds(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'cure light wounds' me".to_string())
    } else {
        Some(format!("@cast 'cure light wounds' {}", data.args))
    }
}

fn cast_cure_serious_wounds(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'cure serious wounds' me".to_string())
    } else {
        Some(format!("@cast 'cure serious wounds' {}", data.args))
    }
}

fn cast_cure_light_wounds_full(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@repeat inf cast 'cure light wounds' me".to_string())
    } else {
        Some(format!(
            "@repeat inf cast 'cure light wounds' {}",
            data.args
        ))
    }
}

fn cast_cure_serious_wounds_full(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@repeat inf cast 'cure serious wounds' me".to_string())
    } else {
        Some(format!(
            "@repeat inf cast 'cure serious wounds' {}",
            data.args
        ))
    }
}

fn cast_water_walking(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'water walking' me".to_string())
    } else {
        Some(format!("@cast 'water walking' {}", data.args))
    }
}

fn cast_invisibility(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'invisibility' me".to_string())
    } else {
        Some(format!("@cast 'invisibility' {}", data.args))
    }
}

fn cast_infravision(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'infravision' me".to_string())
    } else {
        Some(format!("@cast 'infravision' {}", data.args))
    }
}

fn cast_teleport_with_error(_data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    Some("@cast 'teleport with error'".to_string())
}

fn cast_teleport_without_error(_data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    Some("@cast 'teleport without error'".to_string())
}

fn cast_relocate(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    Some(format!("@cast 'relocate' {}", data.args))
}

fn cast_heavy_weight(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        Some("@cast 'heavy weight' me".to_string())
    } else {
        Some(format!("@cast 'heavy weight' {}", data.args))
    }
}

fn unknown_command(_data: &Data, _ctx: &mut CommandContext) -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_predefined_groups() {
        let generic = GenericCommands::default();
        assert_eq!(generic.groups.len(), 3);
        assert!(generic.groups.iter().any(|g| g.name == "cure_spells"));
        assert!(generic.groups.iter().any(|g| g.name == "common_spells"));
        assert!(generic.groups.iter().any(|g| g.name == "navigator"));
    }

    #[test]
    fn commands_returns_only_enabled() {
        let mut generic = GenericCommands::default();
        generic.groups[0].commands[0].enabled = false; // Disable clw

        let commands = generic.commands();
        assert!(!commands.contains_key("clw"));
        assert!(commands.contains_key("csw"));
    }

    #[test]
    fn apply_config_filters_groups() {
        let mut generic = GenericCommands::default();
        generic.apply_config(&["cure_spells".to_string()], &[]);

        assert!(generic.groups[0].commands[0].enabled); // cure_spells enabled
        assert!(!generic.groups[1].commands[0].enabled); // common_spells disabled
        assert!(!generic.groups[2].commands[0].enabled); // navigator disabled
    }

    #[test]
    fn apply_config_disables_specific_commands() {
        let mut generic = GenericCommands::default();
        generic.apply_config(&[], &["clw".to_string(), "cww".to_string()]);

        assert!(!generic.groups[0].commands[0].enabled); // clw disabled
        assert!(generic.groups[0].commands[1].enabled); // csw still enabled
        assert!(!generic.groups[1].commands[0].enabled); // cww disabled
    }

    #[test]
    fn selection_state_all_enabled() {
        let group = GenericCommandGroup::new(
            "test",
            "Test",
            vec![
                GenericCommand::new("a", "cmd a"),
                GenericCommand::new("b", "cmd b"),
            ],
        );
        assert_eq!(group.selection_state(), Some(true));
    }

    #[test]
    fn selection_state_mixed() {
        let mut group = GenericCommandGroup::new(
            "test",
            "Test",
            vec![
                GenericCommand::new("a", "cmd a"),
                GenericCommand::new("b", "cmd b"),
            ],
        );
        group.commands[0].enabled = false;
        assert_eq!(group.selection_state(), None);
    }

    #[test]
    fn cast_cure_light_wounds_with_target() {
        let data = Data::new("clw orc");
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let result = cast_cure_light_wounds(&data, &mut ctx);
        assert_eq!(result, Some("@cast 'cure light wounds' orc".to_string()));
    }

    #[test]
    fn cast_cure_light_wounds_without_target() {
        let data = Data::new("clw");
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let result = cast_cure_light_wounds(&data, &mut ctx);
        assert_eq!(result, Some("@cast 'cure light wounds' me".to_string()));
    }

    #[test]
    fn cast_teleport_without_error_no_args() {
        let data = Data::new("ctwe");
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let result = cast_teleport_with_error(&data, &mut ctx);
        assert_eq!(result, Some("@cast 'teleport with error'".to_string()));
    }

    #[test]
    fn cast_relocate_uses_args() {
        let data = Data::new("cr orthos");
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let result = cast_relocate(&data, &mut ctx);
        assert_eq!(result, Some("@cast 'relocate' orthos".to_string()));
    }
}
