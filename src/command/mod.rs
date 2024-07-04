mod quit;

use crate::command::quit::Quit;
use lazy_static::lazy_static;
use std::collections::HashMap;

pub fn process(cmd: &str, ctx: &egui::Context) {
    let data = Data::new(cmd);
    if let Some(cmd) = COMMANDS.get(&data.cmd) {
        cmd.process(&data, ctx);
    }
}

trait Command {
    fn process(&self, data: &Data, ctx: &egui::Context);
}

struct Data {
    cmd: String,
    args: Vec<String>,
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

lazy_static! {
    static ref COMMANDS: HashMap<String, Box<(dyn Command + Sync)>> = HashMap::from([(
        String::from("quit"),
        Box::new(Quit::default()) as Box<(dyn Command + Sync)>
    )]);
}
