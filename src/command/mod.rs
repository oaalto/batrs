mod quit;

use crate::command::quit::Quit;
use crate::guilds::Guild;
use std::collections::HashMap;

pub fn process(cmd: &str, ctx: &egui::Context, guilds: &[Box<dyn Guild>]) -> Option<String> {
    let data = Data::new(cmd);
    let guild_cmds: HashMap<String, Box<dyn Command>> =
        guilds.iter().flat_map(|g| g.commands()).collect();

    if let Some(cmd) = commands().get(&data.cmd) {
        cmd.process(&data, ctx)
    } else if let Some(cmd) = guild_cmds.get(&data.cmd) {
        cmd.process(&data, ctx)
    } else {
        Some(cmd.to_string())
    }
}

pub fn commands() -> HashMap<String, Box<dyn Command>> {
    HashMap::from([(
        "/quit".to_string(),
        Box::new(Quit::default()) as Box<(dyn Command)>,
    )])
}

pub trait Command {
    fn process(&self, data: &Data, ctx: &egui::Context) -> Option<String>;
}

pub struct Data {
    pub cmd: String,
    pub args: Vec<String>,
}

impl Data {
    fn new(line: &str) -> Self {
        let split: Vec<String> = line.split(' ').map(|s| s.to_string()).collect();

        Self {
            cmd: split[0].to_owned(),
            args: split[1..].iter().map(String::from).collect(),
        }
    }
}
