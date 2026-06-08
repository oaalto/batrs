use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{DiscipleGuild, use_skill};
use std::collections::HashMap;

impl DiscipleGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ucs".to_string(), Self::use_chaotic_spawn as Command),
            ("uc".to_string(), Self::use_clawed_strike as Command),
            ("ukd".to_string(), Self::use_kiss_of_death as Command),
        ])
    }

    pub fn use_chaotic_spawn(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            command::send(abilities::client_send_line("use 'chaotic spawn'"))
        } else {
            command::send(abilities::client_send_line(&format!(
                "use 'chaotic spawn' {}",
                data.args
            )))
        }
    }

    pub fn use_clawed_strike(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("clawed strike", data))
    }

    pub fn use_kiss_of_death(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            Vec::new()
        } else {
            command::send(abilities::client_send_line(&format!(
                "target {};use 'kiss of death' {}",
                data.args, data.args
            )))
        }
    }
}
