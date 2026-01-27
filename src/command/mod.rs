mod guilds;
mod quit;
mod rig;
mod settings;

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
            open_guilds_dialog: ctx.open_guilds_dialog,
            open_settings_dialog: ctx.open_settings_dialog,
            set_rig: ctx.set_rig.take(),
        }
    }
}
