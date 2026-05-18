use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::sabres::SABRE_WEAPON_VAR;
use crate::guilds::{SabresGuild, use_skill};
use std::collections::HashMap;

pub const UNSET_SABRE_WEAPON_HINT: &str =
    "Set main-hand weapon for Sabres in /guilds or settings key sabre_weapon.";

impl SabresGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ul".to_string(), Self::use_lounging as Command),
            ("usf".to_string(), Self::use_sabre_fence as Command),
            ("ug".to_string(), Self::use_gloveknock as Command),
            ("wsw".to_string(), Self::wield_sabre_weapon as Command),
        ])
    }

    fn sabre_weapon_trimmed(ctx: &command::CommandContext) -> Option<&str> {
        let trimmed = ctx.var(SABRE_WEAPON_VAR).unwrap_or_default().trim();
        (!trimmed.is_empty()).then_some(trimmed)
    }

    pub fn use_lounging(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("lounging", data))
    }

    pub fn use_sabre_fence(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("use 'sabre fence'"))
        } else {
            Some(abilities::client_send_line(&format!(
                "target {args};use sabre fence at {args}"
            )))
        }
    }

    pub fn use_gloveknock(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let Some(weapon) = Self::sabre_weapon_trimmed(ctx) else {
            ctx.push_output_line(StyledLine::new(UNSET_SABRE_WEAPON_HINT));
            return None;
        };
        if data.args.is_empty() {
            ctx.push_output_line(StyledLine::new("No target!"));
            return None;
        }
        let target = data.args.trim();
        Some(abilities::compound_send(&[
            &format!("remove {weapon} from right hand"),
            &format!("target {target}"),
            &format!("use gloveknock at {target}"),
        ]))
    }

    pub fn wield_sabre_weapon(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let Some(weapon) = Self::sabre_weapon_trimmed(ctx) else {
            ctx.push_output_line(StyledLine::new(UNSET_SABRE_WEAPON_HINT));
            return None;
        };
        Some(abilities::client_send_line(&format!("wield {weapon}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn ctx(sabre_weapon: &str) -> command::CommandContext {
        command::CommandContext::with_vars(
            HashMap::new(),
            HashMap::from([(SABRE_WEAPON_VAR.to_string(), sabre_weapon.to_string())]),
        )
    }

    fn empty_ctx() -> command::CommandContext {
        ctx("")
    }

    #[test]
    fn lounging_no_target() {
        let result = SabresGuild::use_lounging(&data("ul", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'lounging'".to_string()));
    }

    #[test]
    fn lounging_with_target() {
        let result = SabresGuild::use_lounging(&data("ul", "orc"), &mut empty_ctx());
        assert_eq!(result, Some("@target orc;use 'lounging' orc".to_string()));
    }

    #[test]
    fn sabre_fence_untargeted() {
        let result = SabresGuild::use_sabre_fence(&data("usf", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'sabre fence'".to_string()));
    }

    #[test]
    fn sabre_fence_targeted() {
        let result = SabresGuild::use_sabre_fence(&data("usf", "orc"), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@target orc;use sabre fence at orc".to_string())
        );
    }

    #[test]
    fn gloveknock_requires_weapon() {
        let mut c = empty_ctx();
        let result = SabresGuild::use_gloveknock(&data("ug", "orc"), &mut c);
        assert!(result.is_none());
        assert_eq!(c.output_lines[0].plain_line, UNSET_SABRE_WEAPON_HINT);
    }

    #[test]
    fn gloveknock_requires_target() {
        let mut c = ctx("sabre");
        let result = SabresGuild::use_gloveknock(&data("ug", ""), &mut c);
        assert!(result.is_none());
        assert_eq!(c.output_lines[0].plain_line, "No target!");
    }

    #[test]
    fn gloveknock_success() {
        let result = SabresGuild::use_gloveknock(&data("ug", "orc"), &mut ctx("sabre"));
        assert_eq!(
            result,
            Some("@remove sabre from right hand;target orc;use gloveknock at orc".to_string())
        );
    }

    #[test]
    fn wield_requires_weapon() {
        let mut c = empty_ctx();
        let result = SabresGuild::wield_sabre_weapon(&data("wsw", ""), &mut c);
        assert!(result.is_none());
        assert_eq!(c.output_lines[0].plain_line, UNSET_SABRE_WEAPON_HINT);
    }

    #[test]
    fn wield_success() {
        let result = SabresGuild::wield_sabre_weapon(&data("wsw", ""), &mut ctx("sabre"));
        assert_eq!(result, Some("@wield sabre".to_string()));
    }
}
