use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::NergalGuild;
use std::collections::HashMap;

impl NergalGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
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

    fn cast_enthralling_parasite(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::cast_quoted_with_suffix(
            "enthralling parasite",
            &data.args,
        ))
    }

    fn cast_harvest_vitae(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        Some(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'harvest vitae' {}", data.args),
        ]))
    }

    fn cast_corrupt_ground(_: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        Some(abilities::client_send_line("cast corrupt ground"))
    }

    fn cast_evaluate_host(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        Some(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'evaluate host' {}", data.args),
        ]))
    }

    fn cast_reap_potentia(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        Some(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'reap potentia' {}", data.args),
        ]))
    }

    fn cast_parasitic_swarm(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("cast 'parasitic swarm' {}", data.args),
        ]))
    }

    fn cast_end_enthrallment(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            return None;
        }
        Some(abilities::client_send_line(&format!(
            "cast end enthrallment at {}",
            data.args
        )))
    }

    fn cast_nourish_enthralled(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        let mut parts = data.args.split_whitespace();
        let first = parts.next()?;
        let second = parts.next()?;
        let third = parts.next()?;
        Some(abilities::client_send_line(&format!(
            "cast nourish enthralled at {first} consume {second} {third}"
        )))
    }

    fn cast_call_forth_enthralled(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(&format!(
            "cast call forth enthralled at {}",
            data.args
        )))
    }

    fn use_embrace_the_gifts_aura(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            return None;
        }
        Some(abilities::client_send_line(&format!(
            "use embrace the gifts at start aura {}",
            data.args
        )))
    }

    fn use_embrace_the_gifts_mutation(
        data: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            return None;
        }
        Some(abilities::client_send_line(&format!(
            "use embrace the gifts at start mutation {}",
            data.args
        )))
    }

    fn use_dreary_hibernation(
        _: &command::Data,
        _: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("use dreary hibernation"))
    }

    fn use_stab(data: &command::Data, _: &mut command::CommandContext) -> Option<String> {
        if data.args.trim().is_empty() {
            return None;
        }
        Some(abilities::compound_send(&[
            &format!("target {}", data.args),
            &format!("use 'stab' {}", data.args),
        ]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::Data;
    use std::collections::HashMap;

    #[test]
    fn cep_with_args_uses_quoted_spell() {
        let data = Data {
            cmd: "cep".into(),
            args: "goblin".into(),
        };
        assert_eq!(
            NergalGuild::cast_enthralling_parasite(
                &data,
                &mut command::CommandContext::new(HashMap::new(), true)
            )
            .as_deref(),
            Some("@cast 'enthralling parasite' goblin")
        );
    }

    #[test]
    fn cee_requires_args() {
        let data = Data {
            cmd: "cee".into(),
            args: "   ".into(),
        };
        assert!(
            NergalGuild::cast_end_enthrallment(
                &data,
                &mut command::CommandContext::new(Default::default(), true)
            )
            .is_none()
        );
    }

    #[test]
    fn cee_sends_end_enthrallment() {
        let data = Data {
            cmd: "cee".into(),
            args: "host1".into(),
        };
        let out = NergalGuild::cast_end_enthrallment(
            &data,
            &mut command::CommandContext::new(Default::default(), true),
        )
        .unwrap();
        assert_eq!(out, "@cast end enthrallment at host1");
    }

    #[test]
    fn cne_requires_three_tokens() {
        let data = Data {
            cmd: "cne".into(),
            args: "a b".into(),
        };
        assert!(
            NergalGuild::cast_nourish_enthralled(
                &data,
                &mut command::CommandContext::new(Default::default(), true)
            )
            .is_none()
        );
    }

    #[test]
    fn cne_three_tokens() {
        let data = Data {
            cmd: "cne".into(),
            args: "host minor vitae".into(),
        };
        let out = NergalGuild::cast_nourish_enthralled(
            &data,
            &mut command::CommandContext::new(Default::default(), true),
        )
        .unwrap();
        assert_eq!(out, "@cast nourish enthralled at host consume minor vitae");
    }

    #[test]
    fn chv_targets_then_casts() {
        let data = Data {
            cmd: "chv".into(),
            args: "orc".into(),
        };
        let out = NergalGuild::cast_harvest_vitae(
            &data,
            &mut command::CommandContext::new(Default::default(), true),
        )
        .unwrap();
        assert_eq!(out, "@target orc;cast 'harvest vitae' orc");
    }

    #[test]
    fn us_requires_target() {
        let data = Data {
            cmd: "us".into(),
            args: "".into(),
        };
        assert!(
            NergalGuild::use_stab(
                &data,
                &mut command::CommandContext::new(Default::default(), true)
            )
            .is_none()
        );
    }
}
