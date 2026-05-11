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
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("iron palm", data))
    }

    pub fn use_dim_mak(data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        if data.args.is_empty() {
            ctx.push_output_line(StyledLine::new("No target!"));
            None
        } else {
            Some(format!(
                "@target {};use dim mak at {}",
                data.args, data.args
            ))
        }
    }

    pub fn use_meditation(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag(MOUNT_SUMMONED_FLAG) {
            Some("@dismount;@use meditation".to_string())
        } else {
            Some("@use meditation".to_string())
        }
    }

    pub fn use_pick_locks(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            ctx.push_output_line(StyledLine::new("No target!"));
            None
        } else {
            Some(format!(
                "@target {};use pick locks at {}",
                data.args, data.args
            ))
        }
    }

    pub fn cast_tiger_claw(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("tiger claw", data))
    }

    pub fn cast_flame_fists(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some("@cast flame fists".to_string())
    }

    pub fn use_sneak(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some("use sneak".to_string())
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

    fn empty_ctx() -> command::CommandContext {
        command::CommandContext::new(HashMap::new(), true)
    }

    fn ctx_with_mount_summoned() -> command::CommandContext {
        command::CommandContext::new(
            HashMap::from([(MOUNT_SUMMONED_FLAG.to_string(), true)]),
            true,
        )
    }

    #[test]
    fn iron_palm_without_target() {
        let result = TigerGuild::use_iron_palm(&data("ip", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'iron palm'".to_string()));
    }

    #[test]
    fn iron_palm_with_target() {
        let result = TigerGuild::use_iron_palm(&data("ip", "orc"), &mut empty_ctx());
        assert_eq!(result, Some("@target orc;use 'iron palm' orc".to_string()));
    }

    #[test]
    fn dim_mak_without_target_shows_message() {
        let mut ctx = empty_ctx();
        let result = TigerGuild::use_dim_mak(&data("dm", ""), &mut ctx);
        assert!(result.is_none());
        assert_eq!(ctx.output_lines.len(), 1);
        assert_eq!(ctx.output_lines[0].plain_line, "No target!");
    }

    #[test]
    fn dim_mak_with_target() {
        let result = TigerGuild::use_dim_mak(&data("dm", "orc"), &mut empty_ctx());
        assert_eq!(result, Some("@target orc;use dim mak at orc".to_string()));
    }

    #[test]
    fn meditation_without_mount() {
        let result = TigerGuild::use_meditation(&data("med", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use meditation".to_string()));
    }

    #[test]
    fn meditation_with_mount_summoned_prefixes_dismount() {
        let result = TigerGuild::use_meditation(&data("med", ""), &mut ctx_with_mount_summoned());
        assert_eq!(result, Some("@dismount;@use meditation".to_string()));
    }

    #[test]
    fn pick_locks_without_target_shows_message() {
        let mut ctx = empty_ctx();
        let result = TigerGuild::use_pick_locks(&data("upl", ""), &mut ctx);
        assert!(result.is_none());
        assert_eq!(ctx.output_lines[0].plain_line, "No target!");
    }

    #[test]
    fn pick_locks_with_target() {
        let result = TigerGuild::use_pick_locks(&data("upl", "chest"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target chest;use pick locks at chest".to_string())
        );
    }

    #[test]
    fn tiger_claw_without_target() {
        let result = TigerGuild::cast_tiger_claw(&data("tc", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'tiger claw'".to_string()));
    }

    #[test]
    fn tiger_claw_with_target() {
        let result = TigerGuild::cast_tiger_claw(&data("tc", "orc"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target orc;cast 'tiger claw' orc".to_string())
        );
    }

    #[test]
    fn flame_fists() {
        let result = TigerGuild::cast_flame_fists(&data("cff", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast flame fists".to_string()));
    }

    #[test]
    fn sneak() {
        let result = TigerGuild::use_sneak(&data("usn", ""), &mut empty_ctx());
        assert_eq!(result, Some("use sneak".to_string()));
    }
}
