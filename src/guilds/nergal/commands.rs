use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::{NergalGuild, use_skill};
use std::collections::HashMap;

impl NergalGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cere".to_string(), Self::use_ceremony as Command),
            (
                "cep".to_string(),
                Self::cast_enthralling_parasite as Command,
            ),
            ("chv".to_string(), Self::cast_harvest_vitae),
            ("ccg".to_string(), Self::cast_corrupt_ground),
            ("ceh".to_string(), Self::cast_evaluate_host),
            ("crp".to_string(), Self::cast_reap_potentia),
            ("cps".to_string(), Self::cast_parasitic_swarm),
            ("cee".to_string(), Self::cast_end_enthrallment),
            ("cne".to_string(), Self::cast_nourish_enthralled),
            ("cce".to_string(), Self::cast_call_forth_enthralled),
            ("aura".to_string(), Self::use_embrace_the_gifts_aura),
            ("mutation".to_string(), Self::use_embrace_the_gifts_mutation),
            ("udh".to_string(), Self::use_dreary_hibernation),
            ("us".to_string(), Self::use_stab),
        ])
    }

    fn use_ceremony(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(use_skill("ceremony", data))
    }

    fn cast_enthralling_parasite(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix(
            "enthralling parasite",
            &data.args,
        ))
    }

    fn cast_harvest_vitae(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'harvest vitae' {}", data.args),
        ]))
    }

    fn cast_corrupt_ground(
        _: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix("corrupt ground", ""))
    }

    fn cast_evaluate_host(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'evaluate host' {}", data.args),
        ]))
    }

    fn cast_reap_potentia(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'reap potentia' {}", data.args),
        ]))
    }

    fn cast_parasitic_swarm(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'parasitic swarm' {}", data.args),
        ]))
    }

    fn cast_end_enthrallment(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            return Vec::new();
        }
        command::send(abilities::cast_quoted_with_suffix(
            "end enthrallment",
            data.args.trim(),
        ))
    }

    fn cast_nourish_enthralled(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut parts = data.args.split_whitespace();
        let Some(first) = parts.next() else {
            return Vec::new();
        };
        let Some(second) = parts.next() else {
            return Vec::new();
        };
        let Some(third) = parts.next() else {
            return Vec::new();
        };
        command::send(abilities::cast_quoted_with_suffix(
            "nourish enthralled",
            &format!("{first} consume {second} {third}"),
        ))
    }

    fn cast_call_forth_enthralled(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::cast_quoted_with_suffix(
            "call forth enthralled",
            data.args.trim(),
        ))
    }

    fn use_embrace_the_gifts_aura(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            return Vec::new();
        }
        command::send(abilities::use_quoted_with_suffix(
            "embrace the gifts",
            &format!("start aura {}", data.args.trim()),
        ))
    }

    fn use_embrace_the_gifts_mutation(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            return Vec::new();
        }
        command::send(abilities::use_quoted_with_suffix(
            "embrace the gifts",
            &format!("start mutation {}", data.args.trim()),
        ))
    }

    fn use_dreary_hibernation(
        _: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::use_quoted_with_suffix("dreary hibernation", ""))
    }

    fn use_stab(
        data: &command::Data,
        _: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            return Vec::new();
        }
        command::send(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("use 'stab' {}", data.args),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Data;

    trait CommandEffectTestExt {
        fn as_deref(&self) -> Vec<command::CommandEffect>;
        fn is_none(&self) -> bool;
        fn unwrap(self) -> String;
    }

    impl CommandEffectTestExt for Vec<command::CommandEffect> {
        fn as_deref(&self) -> Vec<command::CommandEffect> {
            self.clone()
        }

        fn is_none(&self) -> bool {
            self.is_empty()
        }

        fn unwrap(self) -> String {
            self.into_iter()
                .find_map(|effect| match effect {
                    command::CommandEffect::Send(line) => Some(line),
                    _ => None,
                })
                .expect("send effect")
        }
    }

    #[test]
    fn cep_with_args_uses_quoted_spell() {
        let data = Data {
            cmd: "cep".into(),
            args: "goblin".into(),
        };
        assert_eq!(
            NergalGuild::cast_enthralling_parasite(&data, &command::CommandEnvironment::empty())
                .as_deref(),
            command::send("@cast 'enthralling parasite' goblin")
        );
    }

    #[test]
    fn cere_uses_ceremony() {
        let data = Data {
            cmd: "cere".into(),
            args: "".into(),
        };
        assert_eq!(
            NergalGuild::use_ceremony(&data, &command::CommandEnvironment::empty()).as_deref(),
            command::send("@use 'ceremony'")
        );
    }

    #[test]
    fn cere_with_target_uses_ceremony_at_target() {
        let data = Data {
            cmd: "cere".into(),
            args: "altar".into(),
        };
        assert_eq!(
            NergalGuild::use_ceremony(&data, &command::CommandEnvironment::empty()).as_deref(),
            command::send("@target altar;use 'ceremony' altar")
        );
    }

    #[test]
    fn cee_requires_args() {
        let data = Data {
            cmd: "cee".into(),
            args: "   ".into(),
        };
        assert!(
            NergalGuild::cast_end_enthrallment(&data, &command::CommandEnvironment::empty())
                .is_none()
        );
    }

    #[test]
    fn cee_sends_end_enthrallment() {
        let data = Data {
            cmd: "cee".into(),
            args: "host1".into(),
        };
        let out = NergalGuild::cast_end_enthrallment(&data, &command::CommandEnvironment::empty())
            .unwrap();
        assert_eq!(out, "@cast 'end enthrallment' host1");
    }

    #[test]
    fn cne_requires_three_tokens() {
        let data = Data {
            cmd: "cne".into(),
            args: "a b".into(),
        };
        assert!(
            NergalGuild::cast_nourish_enthralled(&data, &command::CommandEnvironment::empty())
                .is_none()
        );
    }

    #[test]
    fn cne_three_tokens() {
        let data = Data {
            cmd: "cne".into(),
            args: "host minor vitae".into(),
        };
        let out =
            NergalGuild::cast_nourish_enthralled(&data, &command::CommandEnvironment::empty())
                .unwrap();
        assert_eq!(out, "@cast 'nourish enthralled' host consume minor vitae");
    }

    #[test]
    fn chv_targets_then_casts() {
        let data = Data {
            cmd: "chv".into(),
            args: "orc".into(),
        };
        let out =
            NergalGuild::cast_harvest_vitae(&data, &command::CommandEnvironment::empty()).unwrap();
        assert_eq!(out, "@target orc;cast 'harvest vitae' orc");
    }

    #[test]
    fn us_requires_target() {
        let data = Data {
            cmd: "us".into(),
            args: "".into(),
        };
        assert!(NergalGuild::use_stab(&data, &command::CommandEnvironment::empty()).is_none());
    }
}
