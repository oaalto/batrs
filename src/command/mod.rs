mod quit;

use crate::guilds::Guild;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref COMMANDS: HashMap<String, Command> =
        HashMap::from([("/quit".to_string(), quit::run as Command)]);
}

pub fn process(cmd: &str, ctx: &egui::Context, guilds: &[Box<dyn Guild>]) -> Option<String> {
    let data = Data::new(cmd);
    let guild_cmds: HashMap<String, Command> = guilds.iter().flat_map(|g| g.commands()).collect();

    if let Some(cmd) = COMMANDS.get(&data.cmd) {
        cmd(&data, ctx)
    } else if let Some(cmd) = guild_cmds.get(&data.cmd) {
        cmd(&data, ctx)
    } else {
        Some(cmd.to_string())
    }
}

pub type Command = fn(&Data, &egui::Context) -> Option<String>;

pub struct Data {
    pub cmd: String,
    pub args: String,
}

impl Data {
    fn new(line: &str) -> Self {
        let index = line.find(' ').unwrap_or(line.len());

        Self {
            cmd: line[..index].to_owned(),
            args: line[index..].trim().to_owned(),
        }
    }
}
