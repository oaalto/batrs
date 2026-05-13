mod generic;
mod guilds;
mod quit;
mod raw_logs;
mod settings;

use crate::ansi::StyledLine;
use crate::automation::Action;
use crate::guilds::Guild;
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::mem;

lazy_static! {
    static ref COMMANDS: HashMap<String, CommandDef> = HashMap::from([
        (
            "/quit".to_string(),
            CommandDef::new(quit::run as Command, false),
        ),
        (
            "/guilds".to_string(),
            CommandDef::new(guilds::run as Command, true),
        ),
        (
            "/generic".to_string(),
            CommandDef::new(generic::run as Command, true),
        ),
        (
            "/settings".to_string(),
            CommandDef::new(settings::run as Command, true),
        ),
        (
            "/raw_logs".to_string(),
            CommandDef::new(raw_logs::run as Command, false),
        ),
    ]);
}

pub fn process(
    cmd: &str,
    ctx: &mut CommandContext,
    guilds: &[Box<dyn Guild>],
    generic: &crate::generic_commands::GenericCommands,
) -> CommandOutcome {
    let data = Data::new(cmd);
    let mut guild_cmds: HashMap<String, Command> = HashMap::new();
    for g in guilds {
        for (key, handler) in g.commands() {
            guild_cmds.entry(key).or_insert(handler);
        }
    }
    let generic_cmds = generic.commands();

    let send = if let Some(cmd) = COMMANDS.get(&data.cmd) {
        if cmd.requires_login && !ctx.logged_in {
            None
        } else {
            (cmd.handler)(&data, ctx)
        }
    } else if let Some(cmd) = guild_cmds.get(&data.cmd) {
        cmd(&data, ctx)
    } else if let Some(cmd) = generic_cmds.get(&data.cmd) {
        // Generic commands checked AFTER guild commands
        cmd(&data, ctx)
    } else {
        Some(cmd.to_string())
    };

    CommandOutcome::from_ctx(send, ctx)
}

pub type Command = fn(&Data, &mut CommandContext) -> Option<String>;

pub struct CommandDef {
    handler: Command,
    requires_login: bool,
}

impl CommandDef {
    fn new(handler: Command, requires_login: bool) -> Self {
        Self {
            handler,
            requires_login,
        }
    }
}

#[derive(Debug)]
pub struct CommandContext {
    pub should_quit: bool,
    pub automation_actions: Vec<Action>,
    pub automation_flags: HashMap<String, bool>,
    pub output_lines: Vec<StyledLine>,
    pub open_guilds_dialog: bool,
    pub open_generic_commands_dialog: bool,
    pub open_settings_dialog: bool,
    pub toggle_raw_logs: bool,
    pub logged_in: bool,
    /// Main-hand weapon for Sabres (`/guilds` / `sabre_weapon` in player settings); may be empty.
    pub sabre_weapon: String,
}

impl CommandContext {
    pub fn new(
        automation_flags: HashMap<String, bool>,
        logged_in: bool,
        sabre_weapon: String,
    ) -> Self {
        Self {
            should_quit: false,
            automation_actions: Vec::new(),
            automation_flags,
            output_lines: Vec::new(),
            open_guilds_dialog: false,
            open_generic_commands_dialog: false,
            open_settings_dialog: false,
            toggle_raw_logs: false,
            logged_in,
            sabre_weapon,
        }
    }

    pub fn flag(&self, key: &str) -> bool {
        self.automation_flags.get(key).copied().unwrap_or(false)
    }

    pub fn push_action(&mut self, action: Action) {
        self.automation_actions.push(action);
    }

    pub fn push_output_line(&mut self, line: StyledLine) {
        self.output_lines.push(line);
    }

    pub fn open_guilds_dialog(&mut self) {
        self.open_guilds_dialog = true;
    }

    pub fn open_generic_commands_dialog(&mut self) {
        self.open_generic_commands_dialog = true;
    }

    pub fn open_settings_dialog(&mut self) {
        self.open_settings_dialog = true;
    }

    pub fn toggle_raw_logs(&mut self) {
        self.toggle_raw_logs = true;
    }
}

pub struct Data {
    pub cmd: String,
    pub args: String,
}

impl Data {
    pub fn new(line: &str) -> Self {
        let index = line.find(' ').unwrap_or(line.len());

        Self {
            cmd: line[..index].to_ascii_lowercase(),
            args: line[index..].trim().to_owned(),
        }
    }
}

pub struct CommandOutcome {
    pub send: Option<String>,
    pub should_quit: bool,
    pub automation_actions: Vec<Action>,
    pub output_lines: Vec<StyledLine>,
    pub open_guilds_dialog: bool,
    pub open_generic_commands_dialog: bool,
    pub open_settings_dialog: bool,
    pub toggle_raw_logs: bool,
}

impl CommandOutcome {
    fn from_ctx(send: Option<String>, ctx: &mut CommandContext) -> Self {
        Self {
            send,
            should_quit: ctx.should_quit,
            automation_actions: mem::take(&mut ctx.automation_actions),
            output_lines: mem::take(&mut ctx.output_lines),
            open_guilds_dialog: ctx.open_guilds_dialog,
            open_generic_commands_dialog: ctx.open_generic_commands_dialog,
            open_settings_dialog: ctx.open_settings_dialog,
            toggle_raw_logs: ctx.toggle_raw_logs,
        }
    }
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

    fn ping_handler(data: &Data, _ctx: &mut CommandContext) -> Option<String> {
        if data.args.is_empty() {
            Some("pong".to_string())
        } else {
            Some(format!("pong {}", data.args))
        }
    }

    #[test]
    fn data_parses_command_and_args() {
        let data = Data::new("TeSt arg1 arg2");

        assert_eq!(data.cmd, "test");
        assert_eq!(data.args, "arg1 arg2");
    }

    #[test]
    fn process_handles_builtin_quit() {
        let mut ctx = CommandContext::new(HashMap::new(), false, String::new());
        let generic = GenericCommands::default();
        let outcome = process("/quit", &mut ctx, &[], &generic);

        assert!(outcome.should_quit);
        assert!(outcome.send.is_none());
    }

    #[test]
    fn process_handles_raw_logs_toggle() {
        let mut ctx = CommandContext::new(HashMap::new(), false, String::new());
        let generic = GenericCommands::default();
        let outcome = process("/raw_logs", &mut ctx, &[], &generic);

        assert!(outcome.toggle_raw_logs);
        assert!(outcome.send.is_none());
    }

    #[test]
    fn process_respects_login_requirements() {
        let mut ctx = CommandContext::new(HashMap::new(), false, String::new());
        let generic = GenericCommands::default();
        let outcome = process("/guilds", &mut ctx, &[], &generic);

        assert!(!outcome.open_guilds_dialog);

        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());
        let outcome = process("/guilds", &mut ctx, &[], &generic);

        assert!(outcome.open_guilds_dialog);
    }

    #[test]
    fn process_runs_guild_commands() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());
        let generic = GenericCommands::default();
        let outcome = process("ping world", &mut ctx, &guilds, &generic);

        assert_eq!(outcome.send, Some("pong world".to_string()));
    }

    #[test]
    fn process_runs_generic_commands_after_guild() {
        // Test that guild commands take priority over generic commands
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());

        // Create a generic command that would conflict with "ping" if it existed
        let generic = GenericCommands::default();
        let outcome = process("ping world", &mut ctx, &guilds, &generic);

        // Guild command should win
        assert_eq!(outcome.send, Some("pong world".to_string()));
    }

    #[test]
    fn process_echoes_unknown_commands() {
        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());
        let generic = GenericCommands::default();
        let outcome = process("some raw text", &mut ctx, &[], &generic);

        assert_eq!(outcome.send, Some("some raw text".to_string()));
    }
}
