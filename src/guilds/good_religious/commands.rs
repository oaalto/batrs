use crate::abilities::{cast_quoted_with_suffix, cast_spell};
use crate::command;
use crate::command::Command;
use crate::guilds::GoodReligiousGuild;
use std::collections::HashMap;

impl GoodReligiousGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ccs".to_string(), Self::cast_celestial_spark as Command),
            ("clw".to_string(), Self::cast_cure_light_wounds as Command),
            ("csw".to_string(), Self::cast_cure_serious_wounds as Command),
            (
                "ccw".to_string(),
                Self::cast_cure_critical_wounds as Command,
            ),
            ("ccf".to_string(), Self::cast_create_food as Command),
        ])
    }

    fn cast_celestial_spark(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("celestial spark", data))
    }

    fn cast_cure_light_wounds(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        Self::cast_cure_default_me("cure light wounds", data)
    }

    fn cast_cure_serious_wounds(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        Self::cast_cure_default_me("cure serious wounds", data)
    }

    fn cast_cure_critical_wounds(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        Self::cast_cure_default_me("cure critical wounds", data)
    }

    fn cast_create_food(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_quoted_with_suffix("create food", ""))
    }

    fn cast_cure_default_me(spell: &str, data: &command::Data) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        let target = if args.is_empty() { "me" } else { args };
        command::send(cast_quoted_with_suffix(spell, target))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::{CommandDispatchInput, CommandEnvironment, dispatch};
    use crate::generic_commands::GenericCommands;
    use crate::guilds::{
        GoodReligiousGuild, Guild, MageGuild, RiftwalkerGuild, SpiderGuild, TriadGuild,
    };
    use std::collections::HashMap;

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
    fn celestial_spark_without_target() {
        let result = GoodReligiousGuild::cast_celestial_spark(&data("ccs", ""), &empty_ctx());
        assert_eq!(result, command::send("@cast 'celestial spark'".to_string()));
    }

    #[test]
    fn celestial_spark_with_target() {
        let result = GoodReligiousGuild::cast_celestial_spark(&data("ccs", "orc"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@target orc;cast 'celestial spark' orc".to_string())
        );
    }

    #[test]
    fn cure_light_wounds_defaults_to_me() {
        let result = GoodReligiousGuild::cast_cure_light_wounds(&data("clw", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure light wounds' me".to_string())
        );
    }

    #[test]
    fn cure_light_wounds_with_target() {
        let result = GoodReligiousGuild::cast_cure_light_wounds(&data("clw", "ally"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure light wounds' ally".to_string())
        );
    }

    #[test]
    fn cure_serious_wounds_defaults_to_me() {
        let result = GoodReligiousGuild::cast_cure_serious_wounds(&data("csw", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure serious wounds' me".to_string())
        );
    }

    #[test]
    fn cure_serious_wounds_with_target() {
        let result =
            GoodReligiousGuild::cast_cure_serious_wounds(&data("csw", "ally"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure serious wounds' ally".to_string())
        );
    }

    #[test]
    fn cure_critical_wounds_defaults_to_me() {
        let result = GoodReligiousGuild::cast_cure_critical_wounds(&data("ccw", ""), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure critical wounds' me".to_string())
        );
    }

    #[test]
    fn cure_critical_wounds_with_target() {
        let result =
            GoodReligiousGuild::cast_cure_critical_wounds(&data("ccw", "ally"), &empty_ctx());
        assert_eq!(
            result,
            command::send("@cast 'cure critical wounds' ally".to_string())
        );
    }

    #[test]
    fn create_food_ignores_args() {
        let bare = GoodReligiousGuild::cast_create_food(&data("ccf", ""), &empty_ctx());
        let with_args = GoodReligiousGuild::cast_create_food(&data("ccf", "foo"), &empty_ctx());
        assert_eq!(bare, command::send("@cast 'create food'".to_string()));
        assert_eq!(with_args, command::send("@cast 'create food'".to_string()));
    }

    fn dispatch_send(cmd: &str, guilds: &[Box<dyn Guild>]) -> String {
        let effects = dispatch(
            CommandDispatchInput::new(
                cmd,
                true,
                HashMap::new(),
                HashMap::new(),
                crate::guilds::MonkSkillsConfig::default(),
            ),
            guilds,
            &GenericCommands::default(),
        );
        effects
            .into_iter()
            .filter_map(|effect| match effect {
                command::CommandEffect::Send(line) => Some(line),
                _ => None,
            })
            .next()
            .expect("send effect")
    }

    #[test]
    fn dispatch_prefers_good_religious_ccf_over_mage() {
        let guilds: Vec<Box<dyn Guild>> = vec![
            Box::new(GoodReligiousGuild::default()),
            Box::new(MageGuild::default()),
        ];
        assert_eq!(dispatch_send("ccf", &guilds), "@cast 'create food'");
    }

    #[test]
    fn dispatch_prefers_good_religious_csw_over_spider() {
        let guilds: Vec<Box<dyn Guild>> = vec![
            Box::new(GoodReligiousGuild::default()),
            Box::new(SpiderGuild::default()),
        ];
        assert_eq!(
            dispatch_send("csw", &guilds),
            "@cast 'cure serious wounds' me"
        );
    }

    #[test]
    fn dispatch_prefers_good_religious_ccw_over_triad() {
        let guilds: Vec<Box<dyn Guild>> = vec![
            Box::new(GoodReligiousGuild::default()),
            Box::new(TriadGuild::default()),
        ];
        assert_eq!(
            dispatch_send("ccw", &guilds),
            "@cast 'cure critical wounds' me"
        );
    }

    #[test]
    fn dispatch_prefers_good_religious_ccs_over_riftwalker() {
        let guilds: Vec<Box<dyn Guild>> = vec![
            Box::new(GoodReligiousGuild::default()),
            Box::new(RiftwalkerGuild::default()),
        ];
        assert_eq!(dispatch_send("ccs", &guilds), "@cast 'celestial spark'");
    }
}
