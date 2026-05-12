//! Slash commands from TinyFugue `tf/done_liberator.tf`.

use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{LiberatorGuild, use_skill};
use std::collections::HashMap;

/// `liberator select weakest non armoursmith,guardian,weaponsmith,soldier,locksmith,ranger`
const NON_SPECIALIST_SELECT: &str =
    "weakest non armoursmith,guardian,weaponsmith,soldier,locksmith,ranger";

fn liberator_select_non_specialist() -> String {
    format!("liberator select {}", NON_SPECIALIST_SELECT)
}

fn optional_target_then(cursor_tail: &str, logical_rest: &str) -> String {
    let tail = cursor_tail.trim();
    let logical = if tail.is_empty() {
        logical_rest.to_string()
    } else {
        format!("target {tail};{logical_rest}")
    };
    abilities::client_send_line(&logical)
}

impl LiberatorGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            (
                "lib_autoselect".to_string(),
                Self::lib_autoselect as Command,
            ),
            ("cgl".to_string(), Self::cast_ghost_light as Command),
            ("cgc".to_string(), Self::cast_ghost_chill as Command),
            ("cga".to_string(), Self::cast_ghost_armour as Command),
            ("cgs".to_string(), Self::cast_ghost_sword as Command),
            ("cgcom".to_string(), Self::cast_ghost_companion as Command),
            ("clink".to_string(), Self::cast_ghost_link as Command),
            ("crs".to_string(), Self::cast_restful_sleep as Command),
            ("chg".to_string(), Self::cast_holy_glow as Command),
            ("us".to_string(), Self::use_slash as Command),
            ("ugs".to_string(), Self::use_ghost_slash as Command),
            ("urs".to_string(), Self::use_radiant_slash as Command),
            (
                "ugcf".to_string(),
                Self::use_ghost_channeling_fire as Command,
            ),
            (
                "ugcc".to_string(),
                Self::use_ghost_channeling_camp as Command,
            ),
        ])
    }

    pub fn lib_autoselect(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(&format!(
            "liberator autoselect {}",
            NON_SPECIALIST_SELECT
        )))
    }

    pub fn cast_ghost_light(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let select = liberator_select_non_specialist();
        let logical = if tail.is_empty() {
            format!("{select};cast 'ghost light'")
        } else {
            format!("target {tail};{select};cast 'ghost light' {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn cast_ghost_chill(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let select = liberator_select_non_specialist();
        let logical = if tail.is_empty() {
            format!("{select};cast 'ghost chill'")
        } else {
            format!("target {tail};{select};cast 'ghost chill' {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn cast_ghost_armour(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "liberator select weakest armoursmith",
            "cast ghost armour",
        ]))
    }

    pub fn cast_ghost_sword(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        if tail.is_empty() {
            return None;
        }
        Some(abilities::compound_send(&[
            "liberator select weakest weaponsmith",
            &format!("cast ghost sword at {tail}"),
        ]))
    }

    pub fn cast_ghost_companion(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "liberator select weakest guardian",
            "cast ghost companion",
        ]))
    }

    pub fn cast_ghost_link(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let select = liberator_select_non_specialist();
        Some(abilities::compound_send(&[
            select.as_str(),
            "cast ghost link at me",
        ]))
    }

    pub fn cast_restful_sleep(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let select = liberator_select_non_specialist();
        let logical = if tail.is_empty() {
            format!("{select};cast restful sleep")
        } else {
            format!("{select};cast restful sleep {tail}")
        };
        Some(abilities::client_send_line(&logical))
    }

    pub fn cast_holy_glow(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast holy glow"))
    }

    pub fn use_slash(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("slash", data))
    }

    pub fn use_ghost_slash(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let core = if tail.is_empty() {
            "liberator select clear;liberator select weakest soldier;use 'ghost slash'".to_string()
        } else {
            format!(
                "liberator select clear;liberator select weakest soldier;use 'ghost slash' {}",
                tail
            )
        };
        Some(optional_target_then(tail, &core))
    }

    pub fn use_radiant_slash(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let tail = data.args.trim();
        let select = liberator_select_non_specialist();
        let core = if tail.is_empty() {
            format!("{select};use 'radiant slash'")
        } else {
            format!("{select};use 'radiant slash' {}", tail)
        };
        Some(optional_target_then(tail, &core))
    }

    pub fn use_ghost_channeling_fire(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "liberator select weakest ranger",
            "use 'ghost channeling' fire",
        ]))
    }

    pub fn use_ghost_channeling_camp(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            "liberator select weakest ranger",
            "use 'ghost channeling' camp",
        ]))
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
    fn lib_autoselect_matches_tf() {
        let out = LiberatorGuild::lib_autoselect(&data("lib_autoselect", ""), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@liberator autoselect weakest non armoursmith,guardian,weaponsmith,soldier,locksmith,ranger".to_string())
        );
    }

    #[test]
    fn ghost_light_without_target() {
        let out = LiberatorGuild::cast_ghost_light(&data("cgl", ""), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@liberator select weakest non armoursmith,guardian,weaponsmith,soldier,locksmith,ranger;cast 'ghost light'".to_string())
        );
    }

    #[test]
    fn ghost_light_with_target() {
        let out = LiberatorGuild::cast_ghost_light(&data("cgl", "orc"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@target orc;liberator select weakest non armoursmith,guardian,weaponsmith,soldier,locksmith,ranger;cast 'ghost light' orc".to_string())
        );
    }

    #[test]
    fn ghost_armour() {
        let out = LiberatorGuild::cast_ghost_armour(&data("cga", ""), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@liberator select weakest armoursmith;cast ghost armour".to_string())
        );
    }

    #[test]
    fn ghost_sword_requires_target() {
        assert_eq!(
            LiberatorGuild::cast_ghost_sword(&data("cgs", ""), &mut empty_ctx()),
            None
        );
        let out = LiberatorGuild::cast_ghost_sword(&data("cgs", "troll"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@liberator select weakest weaponsmith;cast ghost sword at troll".to_string())
        );
    }

    #[test]
    fn ghost_slash_with_target() {
        let out = LiberatorGuild::use_ghost_slash(&data("ugs", "orc"), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@target orc;liberator select clear;liberator select weakest soldier;use 'ghost slash' orc".to_string())
        );
    }

    #[test]
    fn holy_glow_unquoted() {
        let out = LiberatorGuild::cast_holy_glow(&data("chg", ""), &mut empty_ctx());
        assert_eq!(out, Some("@cast holy glow".to_string()));
    }

    #[test]
    fn ghost_channeling_fire() {
        let out = LiberatorGuild::use_ghost_channeling_fire(&data("ugcf", ""), &mut empty_ctx());
        assert_eq!(
            out,
            Some("@liberator select weakest ranger;use 'ghost channeling' fire".to_string())
        );
    }
}
