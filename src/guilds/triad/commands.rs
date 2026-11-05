use crate::abilities;
use crate::abilities::cast_spell;
use crate::ansi::StyledLine;
use crate::command;
use crate::command::Command;
use crate::command::ShortcutEntry;
use crate::guilds::TriadGuild;
use crate::player_profile::DAMN_ARMAMENT_TARGET_KEY;
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

    pub fn get_shortcut_catalog(&self) -> Vec<ShortcutEntry> {
        vec![
            ShortcutEntry::new("cclw", "Cast cause light wounds."),
            ShortcutEntry::new("ccsw", "Cast cause serious wounds."),
            ShortcutEntry::new("ccw", "Cast cause critical wounds."),
            ShortcutEntry::new(
                "cda",
                "Cast damn armament on the target configured in /settings.",
            ),
            ShortcutEntry::new("cmc", "Cast mellon collie."),
            ShortcutEntry::new("caoh", "Cast aura of hate."),
        ]
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

    /// Cast on the configured item; invalid or unset targets report a local error without sending.
    fn cast_damn_armament(
        _: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let raw = ctx.var(DAMN_ARMAMENT_TARGET_KEY).unwrap_or_default();
        let target = raw.trim();
        if target.is_empty() {
            return vec![command::output(StyledLine::new(
                "Set the Damn Armament target in /settings",
            ))];
        }
        if raw.contains([';', '\r', '\n']) {
            return vec![command::output(StyledLine::new(
                "Damn Armament target must not contain semicolons or line breaks",
            ))];
        }
        command::send(abilities::cast_quoted_with_suffix("damn armament", target))
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
    fn damn_armament_requires_configured_target() {
        for target in [None, Some(""), Some(" \t ")] {
            let vars = target
                .map(|value| {
                    HashMap::from([(DAMN_ARMAMENT_TARGET_KEY.to_string(), value.to_string())])
                })
                .unwrap_or_default();
            let ctx = CommandEnvironment::with_vars(HashMap::new(), vars);
            assert_eq!(
                TriadGuild::cast_damn_armament(&data("cda", "weapon2"), &ctx),
                vec![command::output(StyledLine::new(
                    "Set the Damn Armament target in /settings"
                ))]
            );
        }
    }

    #[test]
    fn damn_armament_uses_configured_target_and_ignores_args() {
        for (target, expected) in [("weapon1", "weapon1"), (" Ivory sabre ", "Ivory sabre")] {
            let ctx = CommandEnvironment::with_vars(
                HashMap::new(),
                HashMap::from([(DAMN_ARMAMENT_TARGET_KEY.to_string(), target.to_string())]),
            );
            for args in ["", "weapon2"] {
                assert_eq!(
                    TriadGuild::cast_damn_armament(&data("cda", args), &ctx),
                    command::send(format!("@cast 'damn armament' {expected}"))
                );
            }
        }
    }

    #[test]
    fn damn_armament_rejects_command_sequences() {
        for target in [
            "weapon1;quit",
            "weapon1\nquit",
            "weapon1\rquit",
            "\nweapon1",
            "weapon1\r\n",
        ] {
            let ctx = CommandEnvironment::with_vars(
                HashMap::new(),
                HashMap::from([(DAMN_ARMAMENT_TARGET_KEY.to_string(), target.to_string())]),
            );
            assert_eq!(
                TriadGuild::cast_damn_armament(&data("cda", ""), &ctx),
                vec![command::output(StyledLine::new(
                    "Damn Armament target must not contain semicolons or line breaks",
                ))]
            );
        }
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
