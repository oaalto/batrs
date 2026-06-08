use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::tzarakk::MOUNT_SUMMONED_FLAG;
use crate::guilds::{TigerGuild, cast_spell, use_skill};
use std::collections::HashMap;

impl TigerGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ip".to_string(), Self::use_iron_palm as Command),
            ("dm".to_string(), Self::use_dim_mak as Command),
            ("med".to_string(), Self::use_meditation as Command),
            ("upl".to_string(), Self::use_pick_locks as Command),
            ("tc".to_string(), Self::cast_tiger_claw as Command),
            ("cff".to_string(), Self::cast_flame_fists as Command),
            ("usn".to_string(), Self::use_sneak as Command),
        ])
    }

    pub fn use_iron_palm(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("iron palm", data))
    }

    pub fn use_dim_mak(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            vec![command::output(StyledLine::new("No target!"))]
        } else {
            command::send(abilities::client_send_line(&format!(
                "target {};use 'dim mak' {}",
                data.args, data.args
            )))
        }
    }

    pub fn use_meditation(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if ctx.flag(MOUNT_SUMMONED_FLAG) {
            command::send(abilities::compound_send(&["dismount", "use 'meditation'"]))
        } else {
            command::send(abilities::client_send_line("use 'meditation'"))
        }
    }

    pub fn use_pick_locks(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.is_empty() {
            vec![command::output(StyledLine::new("No target!"))]
        } else {
            command::send(abilities::client_send_line(&format!(
                "target {};use 'pick locks' {}",
                data.args, data.args
            )))
        }
    }

    pub fn cast_tiger_claw(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("tiger claw", data))
    }

    pub fn cast_flame_fists(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'flame fists'"))
    }

    pub fn use_sneak(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("use 'sneak'"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guilds::tzarakk::MOUNT_SUMMONED_FLAG;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> command::CommandEnvironment {
        command::CommandEnvironment::empty()
    }

    fn ctx_with_mount_summoned() -> command::CommandEnvironment {
        command::CommandEnvironment::new(
            HashMap::from([(MOUNT_SUMMONED_FLAG.to_string(), true)]),
            HashMap::new(),
        )
    }

    #[test]
    fn iron_palm_without_target() {
        let result = TigerGuild::use_iron_palm(&data("ip", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'iron palm'".to_string()));
    }

    #[test]
    fn iron_palm_with_target() {
        let result = TigerGuild::use_iron_palm(&data("ip", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;use 'iron palm' orc".to_string())
        );
    }

    #[test]
    fn dim_mak_without_target_shows_message() {
        let result = TigerGuild::use_dim_mak(&data("dm", ""), &empty_ctx());
        assert_eq!(result, vec![command::output(StyledLine::new("No target!"))]);
    }

    #[test]
    fn dim_mak_with_target() {
        let result = TigerGuild::use_dim_mak(&data("dm", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;use 'dim mak' orc".to_string())
        );
    }

    #[test]
    fn meditation_without_mount() {
        let result = TigerGuild::use_meditation(&data("med", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'meditation'".to_string()));
    }

    #[test]
    fn meditation_with_mount_summoned_prefixes_dismount() {
        let result = TigerGuild::use_meditation(&data("med", ""), &ctx_with_mount_summoned());
        assert_eq!(
            result,
            command::send("@dismount;use 'meditation'".to_string())
        );
    }

    #[test]
    fn pick_locks_without_target_shows_message() {
        let result = TigerGuild::use_pick_locks(&data("upl", ""), &empty_ctx());
        assert_eq!(result, vec![command::output(StyledLine::new("No target!"))]);
    }

    #[test]
    fn pick_locks_with_target() {
        let result = TigerGuild::use_pick_locks(&data("upl", "chest"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target chest;use 'pick locks' chest".to_string())
        );
    }

    #[test]
    fn tiger_claw_without_target() {
        let result = TigerGuild::cast_tiger_claw(&data("tc", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'tiger claw'".to_string()));
    }

    #[test]
    fn tiger_claw_with_target() {
        let result = TigerGuild::cast_tiger_claw(&data("tc", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;cast 'tiger claw' orc".to_string())
        );
    }

    #[test]
    fn flame_fists() {
        let result = TigerGuild::cast_flame_fists(&data("cff", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'flame fists'".to_string()));
    }

    #[test]
    fn sneak() {
        let result = TigerGuild::use_sneak(&data("usn", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'sneak'".to_string()));
    }
}
