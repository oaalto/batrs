mod catalog;
mod show;

use crate::ansi::StyledLine;
use crate::automation::Action;
use crate::generic_commands::GenericCommands;
use crate::guilds::Guild;
use crate::guilds::MonkSkillsConfig;
use crate::guilds::catalog::GuildSelection;
use crate::triggers::TriggerConfig;
use std::collections::HashMap;
use std::sync::LazyLock;

pub use catalog::{ShortcutEntry, TriggerCatalogEntry};

static BUILTINS: LazyLock<HashMap<String, BuiltinCommand>> = LazyLock::new(|| {
    HashMap::from([
        (
            "/help".to_string(),
            BuiltinCommand::new(builtin_help, false),
        ),
        (
            "/quit".to_string(),
            BuiltinCommand::new(builtin_quit, false),
        ),
        (
            "/connect".to_string(),
            BuiltinCommand::new(builtin_connect, false),
        ),
        (
            "/guilds".to_string(),
            BuiltinCommand::new(builtin_open_guilds, true),
        ),
        (
            "/generic".to_string(),
            BuiltinCommand::new(builtin_open_generic, true),
        ),
        (
            "/triggers".to_string(),
            BuiltinCommand::new(builtin_open_triggers, true),
        ),
        (
            "/settings".to_string(),
            BuiltinCommand::new(builtin_open_settings, true),
        ),
        (
            "/monk".to_string(),
            BuiltinCommand::new(builtin_open_monk, true),
        ),
        (
            "/raw_logs".to_string(),
            BuiltinCommand::new(builtin_toggle_raw_logs, false),
        ),
        (
            "/clear".to_string(),
            BuiltinCommand::new(builtin_clear, false),
        ),
        ("/show".to_string(), BuiltinCommand::new(builtin_show, true)),
    ])
});

const HELP_LINES: [&str; 12] = [
    "Client slash commands:",
    "/help - Shows client slash commands.",
    "/quit - Closes the client.",
    "/connect - Starts a fresh BatMUD connection.",
    "/guilds - Opens the guild picker.",
    "/generic - Opens generic shortcut groups.",
    "/triggers - Toggle built-in trigger groups.",
    "/show - List active shortcuts or line triggers.",
    "/settings - Opens the settings editor.",
    "/monk - Configure monk skill tracks.",
    "/raw_logs - Toggles raw log capture.",
    "/clear - Redraws the display from memory (fixes screen artifacts).",
];

pub fn dispatch(
    input: CommandDispatchInput,
    guilds: &[Box<dyn Guild>],
    guild_command_lookup: &HashMap<String, Command>,
    generic: &GenericCommands,
) -> Vec<CommandEffect> {
    let env = CommandEnvironment::new(
        input.flags.clone(),
        input.vars.clone(),
        input.monk_skills.clone(),
    );
    if input.logged_in && !input.line.trim_start().starts_with('/') && input.line.contains(';') {
        return dispatch_compound_line(&input, guilds, guild_command_lookup, generic, &env);
    }
    collapse_adjacent_sends(dispatch_single_segment(
        &input,
        guilds,
        guild_command_lookup,
        generic,
        &env,
        &input.line,
    ))
}

fn dispatch_compound_line(
    input: &CommandDispatchInput,
    guilds: &[Box<dyn Guild>],
    guild_command_lookup: &HashMap<String, Command>,
    generic: &GenericCommands,
    env: &CommandEnvironment,
) -> Vec<CommandEffect> {
    input
        .line
        .split(';')
        .map(str::trim)
        .filter(|segment| !segment.is_empty())
        .flat_map(|segment| {
            collapse_adjacent_sends(dispatch_single_segment(
                input,
                guilds,
                guild_command_lookup,
                generic,
                env,
                segment,
            ))
        })
        .collect()
}

fn dispatch_single_segment(
    input: &CommandDispatchInput,
    guilds: &[Box<dyn Guild>],
    guild_command_lookup: &HashMap<String, Command>,
    generic: &GenericCommands,
    env: &CommandEnvironment,
    line: &str,
) -> Vec<CommandEffect> {
    let parsed = ParsedCommand::new(line);
    if parsed.original().is_empty() {
        return if input.logged_in {
            vec![CommandEffect::Send(String::new())]
        } else {
            Vec::new()
        };
    }

    if parsed.name() == "/show" {
        if !input.logged_in {
            return Vec::new();
        }
        return show::dispatch_show(
            &parsed,
            &show::ShowContext {
                guild_selection: &input.guild_selection,
                active_guilds: guilds,
                generic,
                trigger_config: &input.trigger_config,
            },
        );
    }

    if let Some(builtin) = BUILTINS.get(parsed.name()) {
        if builtin.requires_login && !input.logged_in {
            return Vec::new();
        }
        return (builtin.run)(&parsed);
    }

    if !input.logged_in {
        return Vec::new();
    }

    if let Some(cmd) = guild_command_lookup.get(parsed.name()) {
        return cmd(&parsed, env);
    }

    if let Some(send) = generic.render_command(parsed.name(), &parsed.args) {
        return vec![CommandEffect::Send(send)];
    }

    vec![CommandEffect::Send(parsed.original())]
}

fn collapse_adjacent_sends(effects: Vec<CommandEffect>) -> Vec<CommandEffect> {
    effects
        .into_iter()
        .fold(Vec::new(), |mut collapsed, effect| {
            match effect {
                CommandEffect::Send(send) => match collapsed.last_mut() {
                    Some(CommandEffect::Send(previous)) => {
                        previous.push(';');
                        previous.push_str(&send);
                    }
                    _ => collapsed.push(CommandEffect::Send(send)),
                },
                other => collapsed.push(other),
            }
            collapsed
        })
}

pub type Command = fn(&ParsedCommand, &CommandEnvironment) -> Vec<CommandEffect>;

pub type Data = ParsedCommand;

/// Merged command lookup for the active guild set. First selected guild wins
/// per alias. Rebuilt on guild-selection changes, not on every dispatch.
pub fn build_guild_command_lookup(guilds: &[Box<dyn Guild>]) -> HashMap<String, Command> {
    let mut lookup: HashMap<String, Command> = HashMap::new();
    for g in guilds {
        for (key, handler) in g.commands() {
            lookup.entry(key).or_insert(handler);
        }
    }
    lookup
}

pub struct CommandDispatchInput {
    line: String,
    logged_in: bool,
    flags: HashMap<String, bool>,
    vars: HashMap<String, String>,
    monk_skills: MonkSkillsConfig,
    guild_selection: GuildSelection,
    trigger_config: TriggerConfig,
}

impl CommandDispatchInput {
    pub fn new(
        line: &str,
        logged_in: bool,
        flags: HashMap<String, bool>,
        vars: HashMap<String, String>,
        monk_skills: MonkSkillsConfig,
        guild_selection: GuildSelection,
        trigger_config: TriggerConfig,
    ) -> Self {
        Self {
            line: line.to_string(),
            logged_in,
            flags,
            vars,
            monk_skills,
            guild_selection,
            trigger_config,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CommandEffect {
    Send(String),
    Automation(Action),
    Output(StyledLine),
    OpenDialog(DialogKind),
    Reconnect,
    ToggleRawLogs,
    Quit,
    Redraw,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DialogKind {
    Guilds,
    GenericCommands,
    Triggers,
    Settings,
    Monk,
}

struct BuiltinCommand {
    run: fn(&ParsedCommand) -> Vec<CommandEffect>,
    requires_login: bool,
}

impl BuiltinCommand {
    fn new(run: fn(&ParsedCommand) -> Vec<CommandEffect>, requires_login: bool) -> Self {
        Self {
            run,
            requires_login,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CommandEnvironment {
    flags: HashMap<String, bool>,
    vars: HashMap<String, String>,
    monk_skills: MonkSkillsConfig,
}

impl CommandEnvironment {
    pub fn new(
        flags: HashMap<String, bool>,
        vars: HashMap<String, String>,
        monk_skills: MonkSkillsConfig,
    ) -> Self {
        Self {
            flags,
            vars,
            monk_skills,
        }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new(), MonkSkillsConfig::default())
    }

    #[cfg(test)]
    pub fn with_vars(flags: HashMap<String, bool>, vars: HashMap<String, String>) -> Self {
        Self::new(flags, vars, MonkSkillsConfig::default())
    }

    pub fn monk_skills(&self) -> &MonkSkillsConfig {
        &self.monk_skills
    }

    pub fn flag(&self, key: &str) -> bool {
        self.flags.get(key).copied().unwrap_or(false)
    }

    pub fn var(&self, key: &str) -> Option<&str> {
        self.vars.get(key).map(String::as_str)
    }
}

pub fn send(line: impl Into<String>) -> Vec<CommandEffect> {
    vec![CommandEffect::Send(line.into())]
}

pub fn automation(action: Action) -> CommandEffect {
    CommandEffect::Automation(action)
}

pub fn automations(actions: impl IntoIterator<Item = Action>) -> Vec<CommandEffect> {
    actions.into_iter().map(CommandEffect::Automation).collect()
}

pub fn output(line: StyledLine) -> CommandEffect {
    CommandEffect::Output(line)
}

pub struct ParsedCommand {
    #[cfg(not(test))]
    pub original: String,
    #[cfg(not(test))]
    pub name: String,
    #[cfg(test)]
    pub cmd: String,
    pub args: String,
}

impl ParsedCommand {
    pub fn new(line: &str) -> Self {
        let original = line.trim().to_string();
        let index = original.find(char::is_whitespace).unwrap_or(original.len());
        let name = original[..index].to_ascii_lowercase();
        let args = original[index..].trim().to_owned();

        Self {
            #[cfg(not(test))]
            original,
            #[cfg(test)]
            cmd: name.clone(),
            #[cfg(not(test))]
            name,
            args,
        }
    }

    pub fn original(&self) -> String {
        #[cfg(test)]
        {
            if self.args.is_empty() {
                self.cmd.clone()
            } else {
                format!("{} {}", self.cmd, self.args)
            }
        }
        #[cfg(not(test))]
        {
            self.original.clone()
        }
    }

    pub fn name(&self) -> &str {
        #[cfg(test)]
        {
            &self.cmd
        }
        #[cfg(not(test))]
        {
            &self.name
        }
    }
}

fn builtin_help(_data: &ParsedCommand) -> Vec<CommandEffect> {
    HELP_LINES
        .into_iter()
        .map(|line| CommandEffect::Output(StyledLine::new(line)))
        .collect()
}

fn builtin_quit(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::Quit]
}

fn builtin_connect(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::Reconnect]
}

fn builtin_open_guilds(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::Guilds)]
}

fn builtin_open_generic(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::GenericCommands)]
}

fn builtin_open_triggers(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::Triggers)]
}

fn builtin_open_settings(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::Settings)]
}

fn builtin_open_monk(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::Monk)]
}

fn builtin_toggle_raw_logs(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::ToggleRawLogs]
}

fn builtin_clear(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::Redraw]
}

fn builtin_show(_data: &ParsedCommand) -> Vec<CommandEffect> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generic_commands::GenericCommands;
    use crate::guilds::Guild;
    use crate::guilds::catalog::GuildSelection;
    use crate::triggers::TriggerConfig;
    use std::collections::HashMap;

    struct DummyGuild;

    impl Guild for DummyGuild {
        fn commands(&self) -> HashMap<String, Command> {
            HashMap::from([("ping".to_string(), ping_handler as Command)])
        }

        fn triggers(&self) -> Vec<crate::triggers::Trigger> {
            Vec::new()
        }
    }

    fn ping_handler(data: &Data, _env: &CommandEnvironment) -> Vec<CommandEffect> {
        if data.args.is_empty() {
            send("pong")
        } else {
            send(format!("pong {}", data.args))
        }
    }

    fn dispatch_line(line: &str, logged_in: bool, guilds: &[Box<dyn Guild>]) -> Vec<CommandEffect> {
        dispatch_with_generic(line, logged_in, guilds, GenericCommands::default())
    }

    fn dispatch_with_generic(
        line: &str,
        logged_in: bool,
        guilds: &[Box<dyn Guild>],
        generic: GenericCommands,
    ) -> Vec<CommandEffect> {
        dispatch(
            CommandDispatchInput::new(
                line,
                logged_in,
                HashMap::new(),
                HashMap::new(),
                MonkSkillsConfig::default(),
                GuildSelection::default(),
                TriggerConfig::default(),
            ),
            guilds,
            &build_guild_command_lookup(guilds),
            &generic,
        )
    }

    fn send_effects(effects: &[CommandEffect]) -> Vec<&str> {
        effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Send(line) => Some(line.as_str()),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn parsed_command_trims_and_parses_command_and_args() {
        let data = ParsedCommand::new("  TeSt arg1 arg2  ");

        assert_eq!(data.original(), "test arg1 arg2");
        assert_eq!(data.name(), "test");
        assert_eq!(data.args, "arg1 arg2");
    }

    #[test]
    fn dispatch_sends_empty_input_when_logged_in() {
        let effects = dispatch_line("  ", true, &[]);

        assert_eq!(send_effects(&effects), vec![""]);
    }

    #[test]
    fn dispatch_handles_builtin_quit() {
        let effects = dispatch_line("/quit", false, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::Quit]));
    }

    #[test]
    fn dispatch_handles_connect_before_login_as_client_reconnect() {
        let effects = dispatch_line("/connect", false, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::Reconnect]));
        assert!(send_effects(&effects).is_empty());
    }

    #[test]
    fn dispatch_handles_connect_after_login_as_client_reconnect() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("/connect", true, &guilds);

        assert!(matches!(effects.as_slice(), [CommandEffect::Reconnect]));
        assert!(send_effects(&effects).is_empty());
    }

    #[test]
    fn dispatch_handles_raw_logs_toggle() {
        let effects = dispatch_line("/raw_logs", false, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::ToggleRawLogs]));
    }

    #[test]
    fn dispatch_handles_builtin_help() {
        let effects = dispatch_line("/help", false, &[]);
        let lines: Vec<&str> = effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Output(line) => Some(line.plain_line.as_str()),
                _ => None,
            })
            .collect();

        assert!(lines.contains(&"/help - Shows client slash commands."));
        assert!(lines.contains(&"/quit - Closes the client."));
        assert!(lines.contains(&"/connect - Starts a fresh BatMUD connection."));
        assert!(lines.contains(&"/guilds - Opens the guild picker."));
        assert!(lines.contains(&"/generic - Opens generic shortcut groups."));
        assert!(lines.contains(&"/triggers - Toggle built-in trigger groups."));
        assert!(lines.contains(&"/show - List active shortcuts or line triggers."));
        assert!(lines.contains(&"/settings - Opens the settings editor."));
        assert!(lines.contains(&"/monk - Configure monk skill tracks."));
        assert!(lines.contains(&"/raw_logs - Toggles raw log capture."));
        assert!(
            lines.contains(&"/clear - Redraws the display from memory (fixes screen artifacts).")
        );
    }

    #[test]
    fn dispatch_handles_clear_before_login() {
        let effects = dispatch_line("/clear", false, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::Redraw]));
        assert!(send_effects(&effects).is_empty());
    }

    #[test]
    fn dispatch_handles_clear_after_login() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("/clear", true, &guilds);

        assert!(matches!(effects.as_slice(), [CommandEffect::Redraw]));
        assert!(send_effects(&effects).is_empty());
    }

    #[test]
    fn dispatch_handles_clear_with_extra_args() {
        let effects = dispatch_line("/clear foo", false, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::Redraw]));
        assert!(send_effects(&effects).is_empty());
    }

    #[test]
    fn dispatch_respects_builtin_login_requirements() {
        let effects = dispatch_line("/guilds", false, &[]);

        assert!(effects.is_empty());

        let effects = dispatch_line("/guilds", true, &[]);

        assert!(matches!(
            effects.as_slice(),
            [CommandEffect::OpenDialog(DialogKind::Guilds)]
        ));
    }

    #[test]
    fn dispatch_requires_login_for_guild_commands() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("ping world", false, &guilds);

        assert!(effects.is_empty());
    }

    #[test]
    fn dispatch_runs_guild_commands() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("ping world", true, &guilds);

        assert_eq!(send_effects(&effects), vec!["pong world"]);
    }

    #[test]
    fn dispatch_runs_generic_commands_after_guild_commands() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("ping world", true, &guilds);

        assert_eq!(send_effects(&effects), vec!["pong world"]);
    }

    #[test]
    fn dispatch_runs_generic_commands() {
        let effects = dispatch_line("clw", true, &[]);

        assert_eq!(send_effects(&effects), vec!["@cast 'cure light wounds' me"]);
    }

    #[test]
    fn dispatch_show_requires_login() {
        let effects = dispatch_line("/show commands", false, &[]);
        assert!(effects.is_empty());
    }

    #[test]
    fn dispatch_show_monk_lists_shortcuts() {
        use crate::guilds::MonkGuild;
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(MonkGuild::default())];
        let effects = dispatch(
            CommandDispatchInput::new(
                "/show commands monk",
                true,
                HashMap::new(),
                HashMap::new(),
                MonkSkillsConfig::default(),
                GuildSelection::from_persisted_keys(&["monk".to_string()], Some("magical")),
                TriggerConfig::default(),
            ),
            &guilds,
            &build_guild_command_lookup(&guilds),
            &GenericCommands::default(),
        );
        let lines: Vec<&str> = effects
            .iter()
            .filter_map(|effect| match effect {
                CommandEffect::Output(line) => Some(line.plain_line.as_str()),
                _ => None,
            })
            .collect();
        assert!(lines.iter().any(|line| line.contains("kata -")));
    }

    #[test]
    fn dispatch_trims_unknown_command_passthrough() {
        let effects = dispatch_line("  some raw text  ", true, &[]);

        assert_eq!(send_effects(&effects), vec!["some raw text"]);
    }

    #[test]
    fn dispatch_splits_logged_in_compound_input_and_preserves_passthrough_order() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("say hello; ping world ; raw", true, &guilds);

        assert_eq!(
            send_effects(&effects),
            vec!["say hello", "pong world", "raw"]
        );
    }

    #[test]
    fn dispatch_expands_generic_and_guild_shortcuts_in_one_compound_line() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("clw;ping world;raw", true, &guilds);

        assert_eq!(
            send_effects(&effects),
            vec!["@cast 'cure light wounds' me", "pong world", "raw"]
        );
    }

    #[test]
    fn dispatch_expands_multiple_shortcut_segments_with_whitespace() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line(" ping one ; clw ; ping two ", true, &guilds);

        assert_eq!(
            send_effects(&effects),
            vec!["pong one", "@cast 'cure light wounds' me", "pong two"]
        );
    }

    #[test]
    fn dispatch_skips_empty_compound_segments() {
        let effects = dispatch_line(" ; look ;; score ; ", true, &[]);

        assert_eq!(send_effects(&effects), vec!["look", "score"]);
    }

    #[test]
    fn dispatch_keeps_pre_login_input_unchanged() {
        let effects = dispatch_line("look;score", false, &[]);

        assert!(effects.is_empty());
    }

    #[test]
    fn dispatch_keeps_standalone_slash_commands_unchanged() {
        let effects = dispatch_line("/clear", true, &[]);

        assert!(matches!(effects.as_slice(), [CommandEffect::Redraw]));
    }

    #[test]
    fn dispatch_keeps_compound_input_with_leading_slash_as_standalone_only() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let effects = dispatch_line("/clear;ping world", true, &guilds);

        assert_eq!(send_effects(&effects), vec!["/clear;ping world"]);
    }

    #[test]
    fn dispatch_keeps_non_send_effects_in_order_when_collapsing_compound_sends() {
        let effects = collapse_adjacent_sends(vec![
            CommandEffect::Send("use 'ceremony'".to_string()),
            CommandEffect::Automation(Action::SetFlag("pending".to_string(), true)),
            CommandEffect::Send("cast 'join soul'".to_string()),
            CommandEffect::Send("raw".to_string()),
        ]);

        assert_eq!(
            effects,
            vec![
                CommandEffect::Send("use 'ceremony'".to_string()),
                CommandEffect::Automation(Action::SetFlag("pending".to_string(), true)),
                CommandEffect::Send("cast 'join soul';raw".to_string()),
            ]
        );
    }

    #[test]
    fn dispatch_does_not_resplit_existing_shortcut_send_payloads() {
        let effects = collapse_adjacent_sends(vec![
            CommandEffect::Send("target foe;use ceremony foe".to_string()),
            CommandEffect::Send("look".to_string()),
        ]);

        assert_eq!(
            send_effects(&effects),
            vec!["target foe;use ceremony foe;look"]
        );
    }

    #[test]
    fn dispatch_preserves_segment_effect_order_with_automation() {
        let effects = dispatch_line(
            "csum;look",
            true,
            &[Box::new(crate::guilds::AnimistGuild {})],
        );

        assert_eq!(
            effects,
            vec![
                CommandEffect::Automation(Action::SetFlag(
                    "animist_separating_soul".to_string(),
                    false,
                )),
                CommandEffect::Automation(Action::SetFlag(
                    "animist_joining_soul".to_string(),
                    false,
                )),
                CommandEffect::Automation(Action::SetFlag(
                    "animist_conjuring_mount".to_string(),
                    false,
                )),
                CommandEffect::Automation(Action::SetFlag(
                    "animist_dismissing_mount".to_string(),
                    false,
                )),
                CommandEffect::Automation(Action::SetFlag(
                    "animist_conjuring_mount".to_string(),
                    true,
                )),
                CommandEffect::Send("@use 'ceremony'".to_string()),
                CommandEffect::Send("look".to_string()),
            ]
        );
    }

    #[test]
    fn dispatch_collapses_adjacent_send_effects_within_one_segment() {
        let effects = dispatch_line("cmd", true, &[Box::new(MultiSendGuild)]);

        assert_eq!(send_effects(&effects), vec!["first;second"]);
    }

    struct MultiSendGuild;

    impl Guild for MultiSendGuild {
        fn commands(&self) -> HashMap<String, Command> {
            HashMap::from([("cmd".to_string(), multi_send_handler as Command)])
        }

        fn triggers(&self) -> Vec<crate::triggers::Trigger> {
            Vec::new()
        }
    }

    fn multi_send_handler(_data: &Data, _env: &CommandEnvironment) -> Vec<CommandEffect> {
        vec![
            CommandEffect::Send("first".to_string()),
            CommandEffect::Send("second".to_string()),
        ]
    }
}
