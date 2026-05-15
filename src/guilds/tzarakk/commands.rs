use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{TzarakkGuild, use_skill};
use std::collections::HashMap;

impl TzarakkGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            // Skills
            ("ut".to_string(), Self::use_trample as Command),
            ("ur".to_string(), Self::use_rampage as Command),
            ("cs".to_string(), Self::use_charge as Command),
            (
                "uht".to_string(),
                Self::use_create_hunting_trophy as Command,
            ),
            ("uhs".to_string(), Self::use_harvest_soul as Command),
            // Spells
            ("cpc".to_string(), Self::cast_preserve_corpse as Command),
            ("cst".to_string(), Self::cast_steed_of_tzarakk as Command),
            ("cban".to_string(), Self::cast_banish_mount as Command),
            ("csdb".to_string(), Self::cast_summon_dire_boar as Command),
            // Utility
            ("med".to_string(), Self::use_meditation as Command),
            ("dmed".to_string(), Self::use_dark_meditation as Command),
            ("sleep".to_string(), Self::do_sleep as Command),
            // Modes
            ("feed_mode".to_string(), Self::set_feed_mode as Command),
            ("heal_mode".to_string(), Self::set_heal_mode as Command),
            ("hunt_mode".to_string(), Self::set_hunt_mode as Command),
        ])
    }

    // Skill handlers
    pub fn use_trample(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("trample", data))
    }

    pub fn use_rampage(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("rampage", data))
    }

    pub fn use_charge(data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(use_skill("charge", data))
    }

    pub fn use_create_hunting_trophy(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "use 'create hunting trophy' at corpse",
        ))
    }

    pub fn use_harvest_soul(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("use 'harvest soul' at corpse"))
    }

    // Spell handlers
    pub fn cast_preserve_corpse(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast 'preserve corpse' at corpse",
        ))
    }

    pub fn cast_steed_of_tzarakk(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'steed of tzarakk'"))
    }

    pub fn cast_banish_mount(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'banish mount'"))
    }

    pub fn cast_summon_dire_boar(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast summon dire boar"))
    }

    // Utility handlers
    pub fn use_meditation(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&["dismount", "use 'meditation'"]))
    }

    pub fn do_sleep(_data: &command::Data, _ctx: &mut command::CommandContext) -> Option<String> {
        Some(abilities::compound_send(&["dismount", "sleep"]))
    }

    pub fn use_dark_meditation(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let logical = match data.args.trim() {
            "hp" => "use dark meditation at sacrifice health",
            "sp" => "use dark meditation at sacrifice power",
            _ => "use dark meditation at sacrifice endurance",
        };
        Some(abilities::compound_send(&["dismount", logical]))
    }

    // Mode handlers
    pub fn set_feed_mode(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some("@rip_action set get all from corpse;tzarakk chaosfeed corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string())
    }

    pub fn set_heal_mode(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(
            "@rip_action set get all from corpse;use 'harvest soul' at corpse;drop zinc;drop mowgles"
                .to_string(),
        )
    }

    pub fn set_hunt_mode(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(
            "@rip_action set get all from corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles"
                .to_string(),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> command::CommandContext {
        command::CommandContext::new(HashMap::new(), true, String::new())
    }

    #[test]
    fn trample_without_target() {
        let result = TzarakkGuild::use_trample(&data("ut", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'trample'".to_string()));
    }

    #[test]
    fn trample_with_target() {
        let result = TzarakkGuild::use_trample(&data("ut", "orc"), &mut empty_ctx());
        assert_eq!(result, Some("@target orc;use 'trample' orc".to_string()));
    }

    #[test]
    fn rampage_without_target() {
        let result = TzarakkGuild::use_rampage(&data("ur", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'rampage'".to_string()));
    }

    #[test]
    fn charge_without_target() {
        let result = TzarakkGuild::use_charge(&data("cs", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'charge'".to_string()));
    }

    #[test]
    fn create_hunting_trophy() {
        let result = TzarakkGuild::use_create_hunting_trophy(&data("uht", ""), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@use 'create hunting trophy' at corpse".to_string())
        );
    }

    #[test]
    fn harvest_soul() {
        let result = TzarakkGuild::use_harvest_soul(&data("uhs", ""), &mut empty_ctx());
        assert_eq!(result, Some("@use 'harvest soul' at corpse".to_string()));
    }

    #[test]
    fn preserve_corpse() {
        let result = TzarakkGuild::cast_preserve_corpse(&data("cpc", ""), &mut empty_ctx());
        assert_eq!(
            result,
            Some("@cast 'preserve corpse' at corpse".to_string())
        );
    }

    #[test]
    fn steed_of_tzarakk() {
        let result = TzarakkGuild::cast_steed_of_tzarakk(&data("cst", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'steed of tzarakk'".to_string()));
    }

    #[test]
    fn banish_mount() {
        let result = TzarakkGuild::cast_banish_mount(&data("cban", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast 'banish mount'".to_string()));
    }

    #[test]
    fn summon_dire_boar() {
        let result = TzarakkGuild::cast_summon_dire_boar(&data("csdb", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast summon dire boar".to_string()));
    }

    #[test]
    fn meditation_includes_dismount() {
        let result = TzarakkGuild::use_meditation(&data("med", ""), &mut empty_ctx());
        assert_eq!(result, Some("@dismount;use 'meditation'".to_string()));
    }

    #[test]
    fn dark_meditation_includes_dismount() {
        let hp = TzarakkGuild::use_dark_meditation(&data("dmed", "hp"), &mut empty_ctx());
        assert_eq!(
            hp,
            Some("@dismount;use dark meditation at sacrifice health".to_string())
        );

        let sp = TzarakkGuild::use_dark_meditation(&data("dmed", "sp"), &mut empty_ctx());
        assert_eq!(
            sp,
            Some("@dismount;use dark meditation at sacrifice power".to_string())
        );

        let endurance = TzarakkGuild::use_dark_meditation(&data("dmed", ""), &mut empty_ctx());
        assert_eq!(
            endurance,
            Some("@dismount;use dark meditation at sacrifice endurance".to_string())
        );
    }

    #[test]
    fn sleep_includes_dismount() {
        let result = TzarakkGuild::do_sleep(&data("sleep", ""), &mut empty_ctx());
        assert_eq!(result, Some("@dismount;sleep".to_string()));
    }

    #[test]
    fn feed_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_feed_mode(&data("feed_mode", ""), &mut empty_ctx());
        assert_eq!(result, Some("@rip_action set get all from corpse;tzarakk chaosfeed corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()));
    }

    #[test]
    fn heal_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_heal_mode(&data("heal_mode", ""), &mut empty_ctx());
        assert_eq!(result, Some("@rip_action set get all from corpse;use 'harvest soul' at corpse;drop zinc;drop mowgles".to_string()));
    }

    #[test]
    fn hunt_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_hunt_mode(&data("hunt_mode", ""), &mut empty_ctx());
        assert_eq!(result, Some("@rip_action set get all from corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()));
    }
}
