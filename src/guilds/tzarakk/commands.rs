use crate::abilities;
use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::tzarakk::MOUNT_SUMMONED_FLAG;
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
    pub fn use_trample(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("trample", data))
    }

    pub fn use_rampage(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("rampage", data))
    }

    pub fn use_charge(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("charge", data))
    }

    pub fn use_create_hunting_trophy(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line(
            "use 'create hunting trophy' at corpse",
        ))
    }

    pub fn use_harvest_soul(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("use 'harvest soul' at corpse"))
    }

    // Spell handlers
    pub fn cast_preserve_corpse(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line(
            "cast 'preserve corpse' at corpse",
        ))
    }

    pub fn cast_steed_of_tzarakk(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'steed of tzarakk'"))
    }

    pub fn cast_banish_mount(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'banish mount'"))
    }

    pub fn cast_summon_dire_boar(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast summon dire boar"))
    }

    // Utility handlers
    pub fn use_meditation(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        remount_then_send(abilities::compound_send(&["dismount", "use 'meditation'"]))
    }

    pub fn do_sleep(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        remount_then_send(abilities::compound_send(&["dismount", "sleep"]))
    }

    pub fn use_dark_meditation(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let logical = match data.args.trim() {
            "hp" => "use dark meditation at sacrifice health",
            "sp" => "use dark meditation at sacrifice power",
            _ => "use dark meditation at sacrifice endurance",
        };
        remount_then_send(abilities::compound_send(&["dismount", logical]))
    }

    // Mode handlers
    pub fn set_feed_mode(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send("@rip_action set get all from corpse;tzarakk chaosfeed corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string())
    }

    pub fn set_heal_mode(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(
            "@rip_action set get all from corpse;use 'harvest soul' at corpse;drop zinc;drop mowgles"
                .to_string(),
        )
    }

    pub fn set_hunt_mode(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(
            "@rip_action set get all from corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles"
                .to_string(),
        )
    }
}

fn remount_then_send(send: String) -> Vec<command::CommandEffect> {
    vec![
        command::automation(Action::SetFlag(MOUNT_SUMMONED_FLAG.to_string(), true)),
        command::CommandEffect::Send(send),
    ]
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

    fn empty_ctx() -> command::CommandEnvironment {
        command::CommandEnvironment::empty()
    }

    fn sets_mount_summoned(effects: &[command::CommandEffect]) -> bool {
        effects.iter().any(|effect| {
            matches!(
                effect,
                command::CommandEffect::Automation(Action::SetFlag(flag, true))
                    if flag == MOUNT_SUMMONED_FLAG
            )
        })
    }

    #[test]
    fn trample_without_target() {
        let result = TzarakkGuild::use_trample(&data("ut", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'trample'".to_string()));
    }

    #[test]
    fn trample_with_target() {
        let result = TzarakkGuild::use_trample(&data("ut", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;use 'trample' orc".to_string())
        );
    }

    #[test]
    fn rampage_without_target() {
        let result = TzarakkGuild::use_rampage(&data("ur", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'rampage'".to_string()));
    }

    #[test]
    fn charge_without_target() {
        let result = TzarakkGuild::use_charge(&data("cs", ""), &empty_ctx());
        assert_eq!(result, command::send("@use 'charge'".to_string()));
    }

    #[test]
    fn create_hunting_trophy() {
        let result = TzarakkGuild::use_create_hunting_trophy(&data("uht", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@use 'create hunting trophy' at corpse".to_string())
        );
    }

    #[test]
    fn harvest_soul() {
        let result = TzarakkGuild::use_harvest_soul(&data("uhs", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@use 'harvest soul' at corpse".to_string())
        );
    }

    #[test]
    fn preserve_corpse() {
        let result = TzarakkGuild::cast_preserve_corpse(&data("cpc", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'preserve corpse' at corpse".to_string())
        );
    }

    #[test]
    fn steed_of_tzarakk() {
        let result = TzarakkGuild::cast_steed_of_tzarakk(&data("cst", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'steed of tzarakk'".to_string())
        );
    }

    #[test]
    fn banish_mount() {
        let result = TzarakkGuild::cast_banish_mount(&data("cban", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'banish mount'".to_string()));
    }

    #[test]
    fn summon_dire_boar() {
        let result = TzarakkGuild::cast_summon_dire_boar(&data("csdb", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast summon dire boar".to_string()));
    }

    #[test]
    fn meditation_includes_dismount() {
        let result = TzarakkGuild::use_meditation(&data("med", ""), &empty_ctx());
        assert!(result.contains(&command::CommandEffect::Send(
            "@dismount;use 'meditation'".to_string()
        )));
        assert!(sets_mount_summoned(&result));
    }

    #[test]
    fn dark_meditation_includes_dismount() {
        let hp = TzarakkGuild::use_dark_meditation(&data("dmed", "hp"), &empty_ctx());
        assert!(hp.contains(&command::CommandEffect::Send(
            "@dismount;use dark meditation at sacrifice health".to_string()
        )));
        assert!(sets_mount_summoned(&hp));

        let sp = TzarakkGuild::use_dark_meditation(&data("dmed", "sp"), &empty_ctx());
        assert!(sp.contains(&command::CommandEffect::Send(
            "@dismount;use dark meditation at sacrifice power".to_string()
        )));
        assert!(sets_mount_summoned(&sp));

        let endurance = TzarakkGuild::use_dark_meditation(&data("dmed", ""), &empty_ctx());
        assert!(endurance.contains(&command::CommandEffect::Send(
            "@dismount;use dark meditation at sacrifice endurance".to_string()
        )));
        assert!(sets_mount_summoned(&endurance));
    }

    #[test]
    fn sleep_includes_dismount() {
        let result = TzarakkGuild::do_sleep(&data("sleep", ""), &empty_ctx());
        assert!(result.contains(&command::CommandEffect::Send("@dismount;sleep".to_string())));
        assert!(sets_mount_summoned(&result));
    }

    #[test]
    fn feed_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_feed_mode(&data("feed_mode", ""), &empty_ctx());
        assert_eq!(result, command::send("@rip_action set get all from corpse;tzarakk chaosfeed corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()));
    }

    #[test]
    fn heal_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_heal_mode(&data("heal_mode", ""), &empty_ctx());
        assert_eq!(result, command::send("@rip_action set get all from corpse;use 'harvest soul' at corpse;drop zinc;drop mowgles".to_string()));
    }

    #[test]
    fn hunt_mode_sets_correct_rip_action() {
        let result = TzarakkGuild::set_hunt_mode(&data("hunt_mode", ""), &empty_ctx());
        assert_eq!(result, command::send("@rip_action set get all from corpse;tzarakk chaosfeed corpse;drop zinc;drop mowgles".to_string()));
    }
}
