//! Slash commands for Inner Circle (`/cbi`, `/cfw`, `/csp`, `/caoa`).

use std::collections::HashMap;

use super::{INNER_CIRCLE_HAS_ENTITY_FLAG, InnerCircleGuild};
use crate::abilities;
use crate::command;
use crate::command::Command;

impl InnerCircleGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cbi".to_string(), Self::cast_blurred_image as Command),
            ("cfw".to_string(), Self::cast_feather_weight as Command),
            (
                "csp".to_string(),
                Self::cast_shield_of_protection as Command,
            ),
            ("caoa".to_string(), Self::cast_armour_of_aether as Command),
        ])
    }

    pub fn cast_blurred_image(
        data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let at = resolve_at_entity_style(data, ctx);
        command::send(abilities::client_send_line(&format!(
            "cast blurred image at {at}"
        )))
    }

    pub fn cast_feather_weight(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let at = resolve_at_feather_weight(data);
        command::send(abilities::client_send_line(&format!(
            "cast feather weight at {at}"
        )))
    }

    pub fn cast_shield_of_protection(
        data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let at = resolve_at_entity_style(data, ctx);
        command::send(abilities::client_send_line(&format!(
            "cast shield of protection at {at}"
        )))
    }

    pub fn cast_armour_of_aether(
        data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let at = resolve_at_entity_style(data, ctx);
        command::send(abilities::client_send_line(&format!(
            "cast armour of aether at {at}"
        )))
    }
}

fn resolve_at_entity_style(data: &command::Data, ctx: &command::CommandEnvironment) -> String {
    let t = data.args.trim();
    if !t.is_empty() {
        t.to_string()
    } else if ctx.flag(INNER_CIRCLE_HAS_ENTITY_FLAG) {
        "entity".to_string()
    } else {
        "me".to_string()
    }
}

fn resolve_at_feather_weight(data: &command::Data) -> String {
    let t = data.args.trim();
    if t.is_empty() {
        "me".to_string()
    } else {
        t.to_string()
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

    fn ctx_with_entity(has_entity: bool) -> CommandEnvironment {
        let mut flags = std::collections::HashMap::new();
        flags.insert(INNER_CIRCLE_HAS_ENTITY_FLAG.to_string(), has_entity);
        CommandEnvironment::new(flags, HashMap::new())
    }

    fn ctx_default() -> CommandEnvironment {
        CommandEnvironment::empty()
    }

    #[test]
    fn blurred_image_with_explicit_target() {
        assert_eq!(
            InnerCircleGuild::cast_blurred_image(&data("cbi", "orc"), &ctx_default()),
            command::send("@cast blurred image at orc".to_string())
        );
    }

    #[test]
    fn blurred_image_defaults_to_me_without_entity_flag() {
        assert_eq!(
            InnerCircleGuild::cast_blurred_image(&data("cbi", ""), &ctx_default()),
            command::send("@cast blurred image at me".to_string())
        );
    }

    #[test]
    fn blurred_image_targets_entity_when_flag_set_and_no_args() {
        assert_eq!(
            InnerCircleGuild::cast_blurred_image(&data("cbi", ""), &ctx_with_entity(true)),
            command::send("@cast blurred image at entity".to_string())
        );
    }

    #[test]
    fn feather_weight_defaults_to_me() {
        assert_eq!(
            InnerCircleGuild::cast_feather_weight(&data("cfw", ""), &ctx_default()),
            command::send("@cast feather weight at me".to_string())
        );
    }

    #[test]
    fn feather_weight_with_target() {
        assert_eq!(
            InnerCircleGuild::cast_feather_weight(&data("cfw", "ally"), &ctx_default()),
            command::send("@cast feather weight at ally".to_string())
        );
    }

    #[test]
    fn shield_of_protection_uses_entity_when_flag_set() {
        assert_eq!(
            InnerCircleGuild::cast_shield_of_protection(&data("csp", ""), &ctx_with_entity(true)),
            command::send("@cast shield of protection at entity".to_string())
        );
    }

    #[test]
    fn armour_of_aether_with_explicit_target() {
        assert_eq!(
            InnerCircleGuild::cast_armour_of_aether(&data("caoa", "golem"), &ctx_default()),
            command::send("@cast armour of aether at golem".to_string())
        );
    }
}
