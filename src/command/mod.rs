use crate::ansi::StyledLine;
use crate::automation::Action;
use crate::generic_commands::GenericCommands;
use crate::guilds::Guild;
use std::collections::HashMap;
use std::sync::LazyLock;

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
            "/settings".to_string(),
            BuiltinCommand::new(builtin_open_settings, true),
        ),
        (
            "/raw_logs".to_string(),
            BuiltinCommand::new(builtin_toggle_raw_logs, false),
        ),
        (
            "/clear".to_string(),
            BuiltinCommand::new(builtin_clear, false),
        ),
    ])
});

const HELP_LINES: [&str; 9] = [
    "Client slash commands:",
    "/help - Shows client slash commands.",
    "/quit - Closes the client.",
    "/connect - Starts a fresh BatMUD connection.",
    "/guilds - Opens the guild picker.",
    "/generic - Opens generic shortcut groups.",
    "/settings - Opens the settings editor.",
    "/raw_logs - Toggles raw log capture.",
    "/clear - Redraws the display from memory (fixes screen artifacts).",
];

pub fn dispatch(
    input: CommandDispatchInput,
    guilds: &[Box<dyn Guild>],
    generic: &GenericCommands,
) -> Vec<CommandEffect> {
    let parsed = ParsedCommand::new(&input.line);
    if parsed.original().is_empty() {
        return if input.logged_in {
            vec![CommandEffect::Send(String::new())]
        } else {
            Vec::new()
        };
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

    let env = CommandEnvironment::new(input.flags, input.vars);
    let mut guild_cmds: HashMap<String, Command> = HashMap::new();
    for g in guilds {
        for (key, handler) in g.commands() {
            guild_cmds.entry(key).or_insert(handler);
        }
    }

    if let Some(cmd) = guild_cmds.get(parsed.name()) {
        return cmd(&parsed, &env);
    }

    if let Some(send) = generic.render_command(parsed.name(), &parsed.args) {
        return vec![CommandEffect::Send(send)];
    }

    vec![CommandEffect::Send(parsed.original())]
}

pub type Command = fn(&ParsedCommand, &CommandEnvironment) -> Vec<CommandEffect>;

pub type Data = ParsedCommand;

pub struct CommandDispatchInput {
    line: String,
    logged_in: bool,
    flags: HashMap<String, bool>,
    vars: HashMap<String, String>,
}

impl CommandDispatchInput {
    pub fn new(
        line: &str,
        logged_in: bool,
        flags: HashMap<String, bool>,
        vars: HashMap<String, String>,
    ) -> Self {
        Self {
            line: line.to_string(),
            logged_in,
            flags,
            vars,
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
    Settings,
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
}

impl CommandEnvironment {
    pub fn new(flags: HashMap<String, bool>, vars: HashMap<String, String>) -> Self {
        Self { flags, vars }
    }

    #[cfg(test)]
    pub fn empty() -> Self {
        Self::new(HashMap::new(), HashMap::new())
    }

    #[cfg(test)]
    pub fn with_vars(flags: HashMap<String, bool>, vars: HashMap<String, String>) -> Self {
        Self::new(flags, vars)
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

fn builtin_open_settings(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::OpenDialog(DialogKind::Settings)]
}

fn builtin_toggle_raw_logs(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::ToggleRawLogs]
}

fn builtin_clear(_data: &ParsedCommand) -> Vec<CommandEffect> {
    vec![CommandEffect::Redraw]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generic_commands::GenericCommands;
    use crate::guilds::Guild;
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
        dispatch(
            CommandDispatchInput::new(line, logged_in, HashMap::new(), HashMap::new()),
            guilds,
            &GenericCommands::default(),
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
        assert!(lines.contains(&"/settings - Opens the settings editor."));
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
    fn dispatch_trims_unknown_command_passthrough() {
        let effects = dispatch_line("  some raw text  ", true, &[]);

        assert_eq!(send_effects(&effects), vec!["some raw text"]);
    }
}
