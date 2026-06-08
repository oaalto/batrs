use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{TriadGuild, cast_spell};
use std::collections::HashMap;

impl TriadGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cclw".to_string(), Self::cast_cause_light_wounds as Command),
            ("ccsw".to_string(), Self::cast_cause_serious_wounds),
            ("ccw".to_string(), Self::cast_cause_critical_wounds),
            ("cda".to_string(), Self::cast_damn_armament),
            ("cmc".to_string(), Self::cast_mellon_collie),
            ("caoh".to_string(), Self::cast_aura_of_hate),
        ])
    }

    fn cast_cause_light_wounds(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("cause light wounds", data))
    }

    fn cast_cause_serious_wounds(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("cause serious wounds", data))
    }

    fn cast_cause_critical_wounds(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("cause critical wounds", data))
    }

    fn cast_damn_armament(
        _: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix(
            "damn armament",
            "weapon2",
        ))
    }

    fn cast_mellon_collie(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("mellon collie", data))
    }

    fn cast_aura_of_hate(
        _: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix("aura of hate", ""))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandEnvironment;

    trait CommandEffectTestExt {
        fn unwrap(self) -> String;
    }

    impl CommandEffectTestExt for Vec<command::CommandEffect> {
        fn unwrap(self) -> String {
            self.into_iter()
                .find_map(|effect| match effect {
                    command::CommandEffect::Send(line) => Some(line),
                    _ => None,
                })
                .expect("send effect")
        }
    }

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
    fn cause_light_wounds_targeted() {
        let result =
            TriadGuild::cast_cause_light_wounds(&data("cclw", "orc"), &empty_ctx()).unwrap();
        assert_eq!(result, "@target orc;cast 'cause light wounds' orc");
    }

    #[test]
    fn damn_armament_weapon2() {
        let result = TriadGuild::cast_damn_armament(&data("cda", ""), &empty_ctx()).unwrap();
        assert_eq!(result, "@cast 'damn armament' weapon2");
    }

    #[test]
    fn aura_of_hate_no_quotes() {
        let result = TriadGuild::cast_aura_of_hate(&data("caoh", ""), &empty_ctx()).unwrap();
        assert_eq!(result, "@cast 'aura of hate'");
    }

    #[test]
    fn mellon_collie_targeted() {
        let result = TriadGuild::cast_mellon_collie(&data("cmc", "troll"), &empty_ctx()).unwrap();
        assert_eq!(result, "@target troll;cast 'mellon collie' troll");
    }
}
