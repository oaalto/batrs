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
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            Some("@use 'chaotic spawn'".to_string())
        } else {
            Some(format!("@use 'chaotic spawn' {}", data.args))
        }
    }

    pub fn use_clawed_strike(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("clawed strike", data))
    }

    pub fn use_kiss_of_death(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            Some(format!(
                "@target {};use kiss of death at {}",
                data.args, data.args
            ))
        }
    }
}
