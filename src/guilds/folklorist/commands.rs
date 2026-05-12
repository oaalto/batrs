use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::FolkloristGuild;
use std::collections::HashMap;

fn require_nonempty_target<'a>(
    args: &'a str,
    ctx: &mut command::CommandContext,
) -> Option<&'a str> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        ctx.push_output_line(StyledLine::new("No target!"));
        return None;
    }
    Some(trimmed)
}

impl FolkloristGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("usc".to_string(), Self::use_study_creature as Command),
            ("uel".to_string(), Self::use_eye_of_loraen as Command),
            ("upl".to_string(), Self::use_plant_lore as Command),
            ("cpb".to_string(), Self::cast_poison_blast as Command),
            ("cvs".to_string(), Self::cast_venom_strike as Command),
            ("cts".to_string(), Self::cast_thorn_spray as Command),
            ("chb".to_string(), Self::cast_herbal_poison_blast as Command),
            ("chh".to_string(), Self::cast_herbal_healing as Command),
            ("cmp".to_string(), Self::cast_minor_protection as Command),
            ("cfp".to_string(), Self::cast_field_of_poison as Command),
        ])
    }

    pub fn use_study_creature(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = require_nonempty_target(&data.args, ctx)?;
        Some(abilities::client_send_line(&format!(
            "use study creature at {target}"
        )))
    }

    pub fn use_eye_of_loraen(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = require_nonempty_target(&data.args, ctx)?;
        Some(abilities::client_send_line(&format!(
            "use eye of loraen at {target}"
        )))
    }

    pub fn use_plant_lore(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = require_nonempty_target(&data.args, ctx)?;
        Some(abilities::client_send_line(&format!(
            "use plant lore at {target}"
        )))
    }

    pub fn cast_poison_blast(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast 'poison blast'"))
        } else {
            Some(abilities::compound_send(&[
                &format!("target {args}"),
                &format!("cast poison blast at {args}"),
            ]))
        }
    }

    pub fn cast_venom_strike(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast 'venom strike'"))
        } else {
            Some(abilities::compound_send(&[
                &format!("target {args}"),
                &format!("cast venom strike at {args}"),
            ]))
        }
    }

    pub fn cast_thorn_spray(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::cast_quoted_with_suffix("thorn spray", ""))
        } else {
            Some(abilities::compound_send(&[
                &format!("target {args}"),
                &format!("cast 'thorn spray' {args}"),
            ]))
        }
    }

    pub fn cast_herbal_poison_blast(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let target = require_nonempty_target(&data.args, ctx)?;
        Some(abilities::compound_send(&[
            &format!("target {target}"),
            &format!("cast 'herbal poison blast' {target} use herb"),
        ]))
    }

    pub fn cast_herbal_healing(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line(
                "cast 'herbal healing' me use herb",
            ))
        } else {
            Some(abilities::client_send_line(&format!(
                "cast 'herbal healing' {args} use herb"
            )))
        }
    }

    pub fn cast_minor_protection(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            Some(abilities::client_send_line("cast 'minor protection' me"))
        } else {
            Some(abilities::client_send_line(&format!(
                "cast 'minor protection' {args}"
            )))
        }
    }

    pub fn cast_field_of_poison(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast field of poison"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandContext;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandContext {
        CommandContext::new(HashMap::new(), true, String::new())
    }

    #[test]
    fn use_study_creature_with_target() {
        let result = FolkloristGuild::use_study_creature(&data("usc", "wolf"), &mut empty_ctx());
        assert_eq!(result, Some("@use study creature at wolf".to_string()));
    }

    #[test]
    fn use_study_creature_without_target() {
        let mut ctx = empty_ctx();
        assert!(FolkloristGuild::use_study_creature(&data("usc", ""), &mut ctx).is_none());
        assert_eq!(ctx.output_lines.len(), 1);
        assert_eq!(ctx.output_lines[0].plain_line, "No target!");
    }

    #[test]
    fn poison_blast_untargeted() {
        assert_eq!(
            FolkloristGuild::cast_poison_blast(&data("cpb", ""), &mut empty_ctx()),
            Some("@cast 'poison blast'".to_string())
        );
    }

    #[test]
    fn poison_blast_targeted_unquoted_mid_form() {
        assert_eq!(
            FolkloristGuild::cast_poison_blast(&data("cpb", "orc"), &mut empty_ctx()),
            Some("@target orc;cast poison blast at orc".to_string())
        );
    }

    #[test]
    fn venom_strike_untargeted() {
        assert_eq!(
            FolkloristGuild::cast_venom_strike(&data("cvs", ""), &mut empty_ctx()),
            Some("@cast 'venom strike'".to_string())
        );
    }

    #[test]
    fn thorn_spray_no_target_single_cast() {
        assert_eq!(
            FolkloristGuild::cast_thorn_spray(&data("cts", ""), &mut empty_ctx()),
            Some("@cast 'thorn spray'".to_string())
        );
    }

    #[test]
    fn thorn_spray_with_target() {
        assert_eq!(
            FolkloristGuild::cast_thorn_spray(&data("cts", "goblin"), &mut empty_ctx()),
            Some("@target goblin;cast 'thorn spray' goblin".to_string())
        );
    }

    #[test]
    fn herbal_poison_blast_requires_target() {
        let mut ctx = empty_ctx();
        assert!(FolkloristGuild::cast_herbal_poison_blast(&data("chb", ""), &mut ctx).is_none());
    }

    #[test]
    fn herbal_poison_blast_compound() {
        assert_eq!(
            FolkloristGuild::cast_herbal_poison_blast(&data("chb", "troll"), &mut empty_ctx()),
            Some("@target troll;cast 'herbal poison blast' troll use herb".to_string())
        );
    }

    #[test]
    fn herbal_healing_self() {
        assert_eq!(
            FolkloristGuild::cast_herbal_healing(&data("chh", ""), &mut empty_ctx()),
            Some("@cast 'herbal healing' me use herb".to_string())
        );
    }

    #[test]
    fn minor_protection_fade_candidate_self_vs_other() {
        assert_eq!(
            FolkloristGuild::cast_minor_protection(&data("cmp", ""), &mut empty_ctx()),
            Some("@cast 'minor protection' me".to_string())
        );
        assert_eq!(
            FolkloristGuild::cast_minor_protection(&data("cmp", "ally"), &mut empty_ctx()),
            Some("@cast 'minor protection' ally".to_string())
        );
    }

    #[test]
    fn field_of_poison() {
        assert_eq!(
            FolkloristGuild::cast_field_of_poison(&data("cfp", ""), &mut empty_ctx()),
            Some("@cast field of poison".to_string())
        );
    }
}
