mod guilds;
mod quit;
mod rig;
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
            "/rig".to_string(),
            CommandDef::new(rig::run as Command, true),
        ),
        (
            "/settings".to_string(),
            CommandDef::new(settings::run as Command, true),
        ),
    ]);
}

pub fn process(cmd: &str, ctx: &mut CommandContext, guilds: &[Box<dyn Guild>]) -> CommandOutcome {
    let data = Data::new(cmd);
    let guild_cmds: HashMap<String, Command> = guilds.iter().flat_map(|g| g.commands()).collect();

    let send = if let Some(cmd) = COMMANDS.get(&data.cmd) {
        if cmd.requires_login && !ctx.logged_in {
            None
        } else {
            (cmd.handler)(&data, ctx)
        }
    } else if let Some(cmd) = guild_cmds.get(&data.cmd) {
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
    pub open_settings_dialog: bool,
    pub logged_in: bool,
    pub set_rig: Option<String>,
}

impl CommandContext {
    pub fn new(automation_flags: HashMap<String, bool>, logged_in: bool) -> Self {
        Self {
            should_quit: false,
            automation_actions: Vec::new(),
            automation_flags,
            output_lines: Vec::new(),
            open_guilds_dialog: false,
            open_settings_dialog: false,
            logged_in,
            set_rig: None,
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

    pub fn open_settings_dialog(&mut self) {
        self.open_settings_dialog = true;
    }

    pub fn set_rig(&mut self, rig: String) {
        self.set_rig = Some(rig);
    }
}

pub struct Data {
    pub cmd: String,
    pub args: String,
}

impl Data {
    fn new(line: &str) -> Self {
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
    pub open_settings_dialog: bool,
    pub set_rig: Option<String>,
}

impl CommandOutcome {
    fn from_ctx(send: Option<String>, ctx: &mut CommandContext) -> Self {
        Self {
            send,
            should_quit: ctx.should_quit,
            automation_actions: mem::take(&mut ctx.automation_actions),
            output_lines: mem::take(&mut ctx.output_lines),
            open_guilds_dialog: ctx.open_guilds_dialog,
            open_settings_dialog: ctx.open_settings_dialog,
            set_rig: ctx.set_rig.take(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
        let mut ctx = CommandContext::new(HashMap::new(), false);
        let outcome = process("/quit", &mut ctx, &[]);

        assert!(outcome.should_quit);
        assert!(outcome.send.is_none());
    }

    #[test]
    fn process_respects_login_requirements() {
        let mut ctx = CommandContext::new(HashMap::new(), false);
        let outcome = process("/guilds", &mut ctx, &[]);

        assert!(!outcome.open_guilds_dialog);

        let mut ctx = CommandContext::new(HashMap::new(), true);
        let outcome = process("/guilds", &mut ctx, &[]);

        assert!(outcome.open_guilds_dialog);
    }

    #[test]
    fn process_runs_guild_commands() {
        let guilds: Vec<Box<dyn Guild>> = vec![Box::new(DummyGuild)];
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let outcome = process("ping world", &mut ctx, &guilds);

        assert_eq!(outcome.send, Some("pong world".to_string()));
    }

    #[test]
    fn process_echoes_unknown_commands() {
        let mut ctx = CommandContext::new(HashMap::new(), true);
        let outcome = process("some raw text", &mut ctx, &[]);

        assert_eq!(outcome.send, Some("some raw text".to_string()));
    }
}
