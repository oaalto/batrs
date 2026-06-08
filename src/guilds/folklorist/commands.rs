use crate::abilities;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::guilds::FolkloristGuild;
use std::collections::HashMap;

fn require_nonempty_target(args: &str) -> Result<&str, command::CommandEffect> {
    let trimmed = args.trim();
    if trimmed.is_empty() {
        return Err(command::output(StyledLine::new("No target!")));
    }
    Ok(trimmed)
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
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = match require_nonempty_target(&data.args) {
            Ok(target) => target,
            Err(effect) => return vec![effect],
        };
        command::send(abilities::use_quoted_with_suffix("study creature", target))
    }

    pub fn use_eye_of_loraen(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = match require_nonempty_target(&data.args) {
            Ok(target) => target,
            Err(effect) => return vec![effect],
        };
        command::send(abilities::use_quoted_with_suffix("eye of loraen", target))
    }

    pub fn use_plant_lore(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = match require_nonempty_target(&data.args) {
            Ok(target) => target,
            Err(effect) => return vec![effect],
        };
        command::send(abilities::use_quoted_with_suffix("plant lore", target))
    }

    pub fn cast_poison_blast(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::send(abilities::client_send_line("cast 'poison blast'"))
        } else {
            command::send(abilities::compound_send(&[
                &format!("target {args}"),
                &abilities::cast_quoted_tail("poison blast", args),
            ]))
        }
    }

    pub fn cast_venom_strike(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::send(abilities::client_send_line("cast 'venom strike'"))
        } else {
            command::send(abilities::compound_send(&[
                &format!("target {args}"),
                &abilities::cast_quoted_tail("venom strike", args),
            ]))
        }
    }

    pub fn cast_thorn_spray(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::send(abilities::cast_quoted_with_suffix("thorn spray", ""))
        } else {
            command::send(abilities::compound_send(&[
                &format!("target {args}"),
                &format!("cast 'thorn spray' {args}"),
            ]))
        }
    }

    pub fn cast_herbal_poison_blast(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let target = match require_nonempty_target(&data.args) {
            Ok(target) => target,
            Err(effect) => return vec![effect],
        };
        command::send(abilities::compound_send(&[
            &format!("target {target}"),
            &format!("cast 'herbal poison blast' {target} use herb"),
        ]))
    }

    pub fn cast_herbal_healing(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::send(abilities::client_send_line(
                "cast 'herbal healing' me use herb",
            ))
        } else {
            command::send(abilities::client_send_line(&format!(
                "cast 'herbal healing' {args} use herb"
            )))
        }
    }

    pub fn cast_minor_protection(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::send(abilities::client_send_line("cast 'minor protection' me"))
        } else {
            command::send(abilities::client_send_line(&format!(
                "cast 'minor protection' {args}"
            )))
        }
    }

    pub fn cast_field_of_poison(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix("field of poison", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandEnvironment;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandEnvironment {
        CommandEnvironment::empty()
    }

    #[test]
    fn use_study_creature_with_target() {
        let result = FolkloristGuild::use_study_creature(&data("usc", "wolf"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@use 'study creature' wolf".to_string())
        );
    }

    #[test]
    fn use_study_creature_without_target() {
        assert_eq!(
            FolkloristGuild::use_study_creature(&data("usc", ""), &empty_ctx()),
            vec![command::output(StyledLine::new("No target!"))]
        );
    }

    #[test]
    fn poison_blast_untargeted() {
        assert_eq!(
            FolkloristGuild::cast_poison_blast(&data("cpb", ""), &empty_ctx()),
            command::send("@cast 'poison blast'".to_string())
        );
    }

    #[test]
    fn poison_blast_targeted() {
        assert_eq!(
            FolkloristGuild::cast_poison_blast(&data("cpb", "orc"), &empty_ctx()),
            command::send("@target orc;cast 'poison blast' orc".to_string())
        );
    }

    #[test]
    fn venom_strike_untargeted() {
        assert_eq!(
            FolkloristGuild::cast_venom_strike(&data("cvs", ""), &empty_ctx()),
            command::send("@cast 'venom strike'".to_string())
        );
    }

    #[test]
    fn thorn_spray_no_target_single_cast() {
        assert_eq!(
            FolkloristGuild::cast_thorn_spray(&data("cts", ""), &empty_ctx()),
            command::send("@cast 'thorn spray'".to_string())
        );
    }

    #[test]
    fn thorn_spray_with_target() {
        assert_eq!(
            FolkloristGuild::cast_thorn_spray(&data("cts", "goblin"), &empty_ctx()),
            command::send("@target goblin;cast 'thorn spray' goblin".to_string())
        );
    }

    #[test]
    fn herbal_poison_blast_requires_target() {
        assert_eq!(
            FolkloristGuild::cast_herbal_poison_blast(&data("chb", ""), &empty_ctx()),
            vec![command::output(StyledLine::new("No target!"))]
        );
    }

    #[test]
    fn herbal_poison_blast_compound() {
        assert_eq!(
            FolkloristGuild::cast_herbal_poison_blast(&data("chb", "troll"), &empty_ctx()),
            command::send("@target troll;cast 'herbal poison blast' troll use herb".to_string())
        );
    }

    #[test]
    fn herbal_healing_self() {
        assert_eq!(
            FolkloristGuild::cast_herbal_healing(&data("chh", ""), &empty_ctx()),
            command::send("@cast 'herbal healing' me use herb".to_string())
        );
    }

    #[test]
    fn minor_protection_fade_candidate_self_vs_other() {
        assert_eq!(
            FolkloristGuild::cast_minor_protection(&data("cmp", ""), &empty_ctx()),
            command::send("@cast 'minor protection' me".to_string())
        );
        assert_eq!(
            FolkloristGuild::cast_minor_protection(&data("cmp", "ally"), &empty_ctx()),
            command::send("@cast 'minor protection' ally".to_string())
        );
    }

    #[test]
    fn field_of_poison() {
        assert_eq!(
            FolkloristGuild::cast_field_of_poison(&data("cfp", ""), &empty_ctx()),
            command::send("@cast 'field of poison'".to_string())
        );
    }
}
