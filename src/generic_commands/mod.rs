use crate::abilities;

/// An individual generic command with an alias and structured execution shape.
#[derive(Debug, Clone)]
pub struct GenericCommand {
    pub alias: String,
    execution: GenericCommandExecution,
    pub enabled: bool,
}

impl GenericCommand {
    fn new(alias: &str, execution: GenericCommandExecution) -> Self {
        Self {
            alias: alias.to_string(),
            execution,
            enabled: true,
        }
    }

    pub fn display_command(&self) -> String {
        self.execution.display_command()
    }

    fn render(&self, args: &str) -> String {
        self.execution.render(args)
    }
}

#[derive(Debug, Clone)]
enum GenericCommandExecution {
    Fixed(&'static str),
    AppendArgs {
        prefix: &'static str,
    },
    AppendArgsDefault {
        prefix: &'static str,
        default: &'static str,
    },
}

impl GenericCommandExecution {
    fn fixed(command: &'static str) -> Self {
        Self::Fixed(command)
    }

    fn append_args(prefix: &'static str) -> Self {
        Self::AppendArgs { prefix }
    }

    fn append_args_default(prefix: &'static str, default: &'static str) -> Self {
        Self::AppendArgsDefault { prefix, default }
    }

    fn display_command(&self) -> String {
        match self {
            GenericCommandExecution::Fixed(command) => (*command).to_string(),
            GenericCommandExecution::AppendArgs { prefix } => format!("{prefix} {{args}}"),
            GenericCommandExecution::AppendArgsDefault { prefix, .. } => {
                format!("{prefix} {{args}}")
            }
        }
    }

    fn render(&self, args: &str) -> String {
        let args = args.trim();
        let logical = match self {
            GenericCommandExecution::Fixed(command) => (*command).to_string(),
            GenericCommandExecution::AppendArgs { prefix } => {
                if args.is_empty() {
                    (*prefix).to_string()
                } else {
                    format!("{prefix} {args}")
                }
            }
            GenericCommandExecution::AppendArgsDefault { prefix, default } => {
                let target = if args.is_empty() { *default } else { args };
                format!("{prefix} {target}")
            }
        };
        abilities::client_send_line(&logical)
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
                Self::common_skills_group(),
                Self::misc_group(),
            ],
        }
    }

    pub fn render_command(&self, alias: &str, args: &str) -> Option<String> {
        self.groups
            .iter()
            .flat_map(|group| group.commands.iter())
            .find(|command| command.enabled && command.alias == alias)
            .map(|command| command.render(args))
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

    // Predefined command groups

    fn cure_spells_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "cure_spells",
            "Cure Spells",
            vec![
                GenericCommand::new(
                    "clw",
                    GenericCommandExecution::append_args_default("cast 'cure light wounds'", "me"),
                ),
                GenericCommand::new(
                    "csw",
                    GenericCommandExecution::append_args_default(
                        "cast 'cure serious wounds'",
                        "me",
                    ),
                ),
                GenericCommand::new(
                    "clwf",
                    GenericCommandExecution::append_args_default(
                        "repeat inf cast 'cure light wounds'",
                        "me",
                    ),
                ),
                GenericCommand::new(
                    "cswf",
                    GenericCommandExecution::append_args_default(
                        "repeat inf cast 'cure serious wounds'",
                        "me",
                    ),
                ),
            ],
        )
    }

    fn common_spells_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "common_spells",
            "Common Spells",
            vec![
                GenericCommand::new(
                    "cww",
                    GenericCommandExecution::append_args_default("cast 'water walking'", "me"),
                ),
                GenericCommand::new(
                    "cinv",
                    GenericCommandExecution::append_args_default("cast 'invisibility'", "me"),
                ),
                GenericCommand::new(
                    "cinf",
                    GenericCommandExecution::append_args_default("cast 'infravision'", "me"),
                ),
            ],
        )
    }

    fn navigator_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "navigator",
            "Navigator",
            vec![
                GenericCommand::new(
                    "ctwe",
                    GenericCommandExecution::fixed("cast 'teleport with error'"),
                ),
                GenericCommand::new(
                    "ctw",
                    GenericCommandExecution::fixed("cast 'teleport without error'"),
                ),
                GenericCommand::new(
                    "cr",
                    GenericCommandExecution::append_args("cast 'relocate'"),
                ),
                GenericCommand::new(
                    "chw",
                    GenericCommandExecution::append_args_default("cast 'heavy weight'", "me"),
                ),
            ],
        )
    }

    fn common_skills_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "common_skills",
            "Common Skills",
            vec![
                GenericCommand::new("ufb", GenericCommandExecution::fixed("use 'fire building'")),
                GenericCommand::new("camp", GenericCommandExecution::fixed("use 'camping'")),
            ],
        )
    }

    fn misc_group() -> GenericCommandGroup {
        GenericCommandGroup::new(
            "misc",
            "Misc",
            vec![
                GenericCommand::new(
                    "lich_rip",
                    GenericCommandExecution::fixed(
                        "rip_action set get all from corpse;drop zinc;drop mowgles",
                    ),
                ),
                GenericCommand::new(
                    "normal_rip",
                    GenericCommandExecution::fixed(
                        "rip_action set get all from corpse;dig grave;drop zinc;drop mowgles",
                    ),
                ),
                GenericCommand::new(
                    "dig_rip",
                    GenericCommandExecution::fixed(
                        "rip_action set get all from corpse;dig grave;drop zinc;drop mowgles",
                    ),
                ),
            ],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_has_predefined_groups() {
        let generic = GenericCommands::default();
        assert_eq!(generic.groups.len(), 5);
        assert!(generic.groups.iter().any(|g| g.name == "cure_spells"));
        assert!(generic.groups.iter().any(|g| g.name == "common_spells"));
        assert!(generic.groups.iter().any(|g| g.name == "navigator"));
        assert!(generic.groups.iter().any(|g| g.name == "common_skills"));
        assert!(generic.groups.iter().any(|g| g.name == "misc"));
    }

    #[test]
    fn render_command_returns_only_enabled() {
        let mut generic = GenericCommands::default();
        generic.groups[0].commands[0].enabled = false; // Disable clw

        assert!(generic.render_command("clw", "orc").is_none());
        assert_eq!(
            generic.render_command("csw", "orc"),
            Some("@cast 'cure serious wounds' orc".to_string())
        );
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
                GenericCommand::new("a", GenericCommandExecution::fixed("cmd a")),
                GenericCommand::new("b", GenericCommandExecution::fixed("cmd b")),
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
                GenericCommand::new("a", GenericCommandExecution::fixed("cmd a")),
                GenericCommand::new("b", GenericCommandExecution::fixed("cmd b")),
            ],
        );
        group.commands[0].enabled = false;
        assert_eq!(group.selection_state(), None);
    }

    #[test]
    fn cast_cure_light_wounds_with_target() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("clw", "orc"),
            Some("@cast 'cure light wounds' orc".to_string())
        );
    }

    #[test]
    fn cast_cure_light_wounds_without_target() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("clw", ""),
            Some("@cast 'cure light wounds' me".to_string())
        );
    }

    #[test]
    fn cast_teleport_without_error_no_args() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("ctwe", ""),
            Some("@cast 'teleport with error'".to_string())
        );
    }

    #[test]
    fn cast_relocate_uses_args() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("cr", "orthos"),
            Some("@cast 'relocate' orthos".to_string())
        );
    }

    #[test]
    fn use_fire_building_fixed_output() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("ufb", ""),
            Some("@use 'fire building'".to_string())
        );
    }

    #[test]
    fn use_camping_fixed_output() {
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("camp", ""),
            Some("@use 'camping'".to_string())
        );
    }

    #[test]
    fn rip_lich_matches_expected() {
        assert_eq!(
            GenericCommands::default().render_command("lich_rip", ""),
            Some("@rip_action set get all from corpse;drop zinc;drop mowgles".to_string())
        );
    }

    #[test]
    fn rip_dig_grave_matches_expected() {
        let expected =
            "@rip_action set get all from corpse;dig grave;drop zinc;drop mowgles".to_string();
        let generic = GenericCommands::default();
        assert_eq!(
            generic.render_command("normal_rip", ""),
            Some(expected.clone())
        );
        assert_eq!(generic.render_command("dig_rip", ""), Some(expected));
    }

    #[test]
    fn predefined_commands_render_expected_send_lines() {
        let generic = GenericCommands::default();
        let cases = [
            ("clw", "", "@cast 'cure light wounds' me"),
            ("clw", "ally", "@cast 'cure light wounds' ally"),
            ("csw", "", "@cast 'cure serious wounds' me"),
            ("clwf", "", "@repeat inf cast 'cure light wounds' me"),
            (
                "cswf",
                "ally",
                "@repeat inf cast 'cure serious wounds' ally",
            ),
            ("cww", "", "@cast 'water walking' me"),
            ("cinv", "ally", "@cast 'invisibility' ally"),
            ("cinf", "", "@cast 'infravision' me"),
            ("ctwe", "ignored", "@cast 'teleport with error'"),
            ("ctw", "", "@cast 'teleport without error'"),
            ("cr", "orthos", "@cast 'relocate' orthos"),
            ("chw", "", "@cast 'heavy weight' me"),
            ("ufb", "ignored", "@use 'fire building'"),
            ("camp", "", "@use 'camping'"),
            (
                "lich_rip",
                "",
                "@rip_action set get all from corpse;drop zinc;drop mowgles",
            ),
            (
                "normal_rip",
                "",
                "@rip_action set get all from corpse;dig grave;drop zinc;drop mowgles",
            ),
            (
                "dig_rip",
                "",
                "@rip_action set get all from corpse;dig grave;drop zinc;drop mowgles",
            ),
        ];

        for (alias, args, expected) in cases {
            assert_eq!(
                generic.render_command(alias, args).as_deref(),
                Some(expected),
                "alias {alias}"
            );
        }
    }

    #[test]
    fn display_command_is_derived_from_execution_shape() {
        let generic = GenericCommands::default();
        let clw = &generic.groups[0].commands[0];
        let ctwe = &generic.groups[2].commands[0];

        assert_eq!(clw.display_command(), "cast 'cure light wounds' {args}");
        assert_eq!(ctwe.display_command(), "cast 'teleport with error'");
    }
}
