use crate::abilities;
use crate::ansi::StyledLine;
use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::riftwalker::{
    AIR_SKILL, EARTH_SKILL, FIRE_SKILL, RIFTWALKER_ELEMENT_VAR, RIFTWALKER_HAS_ENTITY_FLAG,
    RIFTWALKER_SKILL_VAR, WATER_SKILL,
};
use crate::guilds::{RiftwalkerGuild, cast_spell};
use std::collections::HashMap;

impl RiftwalkerGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("ufire".to_string(), Self::set_skill_fire as Command),
            ("uair".to_string(), Self::set_skill_air as Command),
            ("uearth".to_string(), Self::set_skill_earth as Command),
            ("uwater".to_string(), Self::set_skill_water as Command),
            ("ccs".to_string(), Self::cmd_use_current_skill as Command),
            ("estat".to_string(), Self::cmd_gem_entity as Command),
            ("csum".to_string(), Self::cmd_summon_entity as Command),
            ("cdis".to_string(), Self::cmd_dismiss_entity as Command),
            ("cb".to_string(), Self::cmd_beckon_entity as Command),
            ("ctrl".to_string(), Self::cmd_entity_control as Command),
            (
                "ctrll".to_string(),
                Self::cmd_entity_control_long as Command,
            ),
            ("cer".to_string(), Self::cmd_entity_regen as Command),
            ("cte".to_string(), Self::cmd_transform_entity as Command),
            ("cs".to_string(), Self::cmd_start_spark_birth as Command),
            ("css".to_string(), Self::cmd_start_rift_pulse as Command),
            (
                "csd".to_string(),
                Self::cmd_start_dimensional_leech as Command,
            ),
            ("csb".to_string(), Self::cmd_cast_spark_birth as Command),
            ("crp".to_string(), Self::cmd_cast_rift_pulse as Command),
            (
                "cdl".to_string(),
                Self::cmd_cast_dimensional_leech as Command,
            ),
            (
                "cfa".to_string(),
                Self::cmd_cast_force_absorption as Command,
            ),
            (
                "cmie".to_string(),
                Self::cmd_cast_mirror_image_entity as Command,
            ),
            ("cam".to_string(), Self::cmd_cast_absorbing_meld as Command),
            ("ciw".to_string(), Self::cmd_cast_iron_will as Command),
            ("zz".to_string(), Self::cmd_stop_use as Command),
            ("gwield".to_string(), Self::cmd_gem_wield as Command),
            ("rwdiag".to_string(), Self::cmd_diag as Command),
            ("rwfix".to_string(), Self::cmd_fix as Command),
        ])
    }

    pub fn set_skill_fire(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_set_entity_skill(ctx, FIRE_SKILL, "fire");
        None
    }

    pub fn set_skill_air(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_set_entity_skill(ctx, AIR_SKILL, "air");
        None
    }

    pub fn set_skill_earth(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_set_entity_skill(ctx, EARTH_SKILL, "earth");
        None
    }

    pub fn set_skill_water(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_set_entity_skill(ctx, WATER_SKILL, "water");
        None
    }

    pub fn cmd_use_current_skill(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_target_gem_chain(ctx, data.args.trim());
        None
    }

    pub fn cmd_gem_entity(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        if args.is_empty() {
            ctx.push_action(Action::Send(abilities::client_send_line(&format!(
                "gem entities {{{}}}",
                RIFTWALKER_ELEMENT_VAR
            ))));
        } else {
            ctx.push_action(Action::Send(abilities::client_send_line(&format!(
                "gem entities {args}"
            ))));
        }
        None
    }

    pub fn cmd_summon_entity(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        ctx.push_action(Action::Send(abilities::client_send_line(&format!(
            "cast 'summon rift entity' at {args}"
        ))));
        apply_element_from_free_text(ctx, args);
        ctx.push_action(Action::SetFlag(
            RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
            true,
        ));
        None
    }

    pub fn cmd_dismiss_entity(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        ctx.push_action(Action::SetFlag(
            RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
            false,
        ));
        Some(abilities::client_send_line("cast 'dismiss rift entity'"))
    }

    pub fn cmd_beckon_entity(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'beckon rift entity'"))
    }

    pub fn cmd_entity_control(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            Some(abilities::client_send_line(
                "cast 'establish entity control'",
            ))
        } else {
            Some(abilities::client_send_line(&format!(
                "cast 'establish entity control' at {}",
                data.args.trim()
            )))
        }
    }

    pub fn cmd_entity_control_long(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(
            "cast 'establish entity control' at 10",
        ))
    }

    pub fn cmd_entity_regen(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'regenerate rift entity'"))
    }

    pub fn cmd_transform_entity(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        let args = data.args.trim();
        match args.split_once(char::is_whitespace) {
            Some((target_at, rest)) => {
                ctx.push_action(Action::Send(abilities::client_send_line(&format!(
                    "cast 'transform rift entity' at {target_at}"
                ))));
                let src = rest.trim();
                apply_element_from_free_text(ctx, if src.is_empty() { target_at } else { src });
                None
            }
            None if !args.is_empty() => {
                ctx.push_action(Action::Send(abilities::client_send_line(&format!(
                    "cast 'transform rift entity' at {args}"
                ))));
                apply_element_from_free_text(ctx, args);
                None
            }
            None => {
                ctx.push_output_line(StyledLine::new(
                    "Riftwalker: cte expects a target direction (and optional element text).",
                ));
                None
            }
        }
    }

    pub fn cmd_start_spark_birth(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_opening_battle(ctx, data.args.trim(), "spark birth");
        None
    }

    pub fn cmd_start_rift_pulse(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_opening_battle(ctx, data.args.trim(), "rift pulse");
        None
    }

    pub fn cmd_start_dimensional_leech(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        push_opening_battle(ctx, data.args.trim(), "dimensional leech");
        None
    }

    pub fn cmd_cast_spark_birth(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("spark birth", data))
    }

    pub fn cmd_cast_rift_pulse(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("rift pulse", data))
    }

    pub fn cmd_cast_dimensional_leech(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(cast_spell("dimensional leech", data))
    }

    pub fn cmd_cast_force_absorption(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            Some(abilities::client_send_line(
                "cast 'force absorption' at entity",
            ))
        } else {
            Some(cast_spell("force absorption", data))
        }
    }

    pub fn cmd_cast_mirror_image_entity(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'mirror image' at entity"))
    }

    pub fn cmd_cast_absorbing_meld(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'Absorbing meld'"))
    }

    pub fn cmd_cast_iron_will(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.trim().is_empty() {
            Some(abilities::client_send_line("cast 'iron will' at entity"))
        } else {
            Some(cast_spell("iron will", data))
        }
    }

    pub fn cmd_stop_use(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::compound_send(&["zz", "gem cmd use stop"]))
    }

    pub fn cmd_gem_wield(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line(&format!(
            "gem cmd wield {}",
            data.args.trim()
        )))
    }

    pub fn cmd_diag(_data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        let has = ctx.flag(RIFTWALKER_HAS_ENTITY_FLAG);
        ctx.push_output_line(StyledLine::new(&format!(
            "Riftwalker: has_entity={has} (per-element labels: /guilds)",
        )));
        None
    }

    pub fn cmd_fix(_data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        ctx.push_output_line(StyledLine::new(
            "Riftwalker: TinyFugue borg/hook toggles do not apply in Batrs.",
        ));
        None
    }
}

fn push_set_entity_skill(ctx: &mut command::CommandContext, skill: &str, element: &str) {
    ctx.push_action(Action::SetVar(
        RIFTWALKER_SKILL_VAR.to_string(),
        skill.to_string(),
    ));
    ctx.push_action(Action::SetVar(
        RIFTWALKER_ELEMENT_VAR.to_string(),
        element.to_string(),
    ));
    ctx.push_action(Action::SetFlag(
        RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
        true,
    ));
}

fn apply_element_from_free_text(ctx: &mut command::CommandContext, blob: &str) {
    let lowered = blob.to_ascii_lowercase();
    if lowered.contains("fire") {
        push_set_entity_skill(ctx, FIRE_SKILL, "fire");
    } else if lowered.contains("air") {
        push_set_entity_skill(ctx, AIR_SKILL, "air");
    } else if lowered.contains("earth") {
        push_set_entity_skill(ctx, EARTH_SKILL, "earth");
    } else if lowered.contains("water") {
        push_set_entity_skill(ctx, WATER_SKILL, "water");
    }
}

fn push_target_gem_chain(ctx: &mut command::CommandContext, tail: &str) {
    let trimmed = tail.trim();
    if !trimmed.is_empty() {
        ctx.push_action(Action::Send(abilities::client_send_line(&format!(
            "target {trimmed};gem cmd target {trimmed}"
        ))));
    }
    let gem = if trimmed.is_empty() {
        abilities::client_send_line(&format!("gem cmd use '{{{}}}'", RIFTWALKER_SKILL_VAR))
    } else {
        abilities::client_send_line(&format!(
            "gem cmd use '{{{}}}' {}",
            RIFTWALKER_SKILL_VAR, trimmed
        ))
    };
    ctx.push_action(Action::Send(gem));
}

fn push_opening_battle(ctx: &mut command::CommandContext, tail: &str, spell: &str) {
    let trimmed = tail.trim();
    if !trimmed.is_empty() {
        ctx.push_action(Action::Send(abilities::client_send_line(&format!(
            "target {trimmed};gem cmd target {trimmed}"
        ))));
    }
    let cast_send = abilities::cast_quoted_with_suffix(spell, trimmed);
    ctx.push_action(Action::Send(cast_send));
    let gem_send = if trimmed.is_empty() {
        abilities::client_send_line(&format!("gem cmd use '{{{}}}'", RIFTWALKER_SKILL_VAR))
    } else {
        abilities::client_send_line(&format!(
            "gem cmd use '{{{}}}' {}",
            RIFTWALKER_SKILL_VAR, trimmed
        ))
    };
    ctx.push_action(Action::Send(gem_send));
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::command::{CommandContext, Data};
    use crate::guilds::riftwalker::RIFTWALKER_HAS_ENTITY_FLAG;
    use std::mem;

    #[test]
    fn dismiss_queues_flag_then_sends_on_separate_application() {
        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());
        let outcome = RiftwalkerGuild::cmd_dismiss_entity(
            &Data {
                cmd: "cdis".to_string(),
                args: String::new(),
            },
            &mut ctx,
        );
        assert_eq!(outcome, Some("@cast 'dismiss rift entity'".to_string()));
        assert!(matches!(
            ctx.automation_actions.as_slice(),
            [Action::SetFlag(flag, false)] if flag == RIFTWALKER_HAS_ENTITY_FLAG
        ));
    }

    #[test]
    fn opening_battle_with_target_splits_cast_and_skill() {
        let mut ctx = CommandContext::new(HashMap::new(), true, String::new());
        let mut automation = Automation::new();
        automation.set_var(RIFTWALKER_SKILL_VAR, FIRE_SKILL.to_string());
        RiftwalkerGuild::cmd_start_spark_birth(
            &Data {
                cmd: "cs".to_string(),
                args: "troll".to_string(),
            },
            &mut ctx,
        );
        let sends = automation.apply_actions(mem::take(&mut ctx.automation_actions));
        assert_eq!(
            sends,
            vec![
                "@target troll;gem cmd target troll".to_string(),
                "@cast 'spark birth' troll".to_string(),
                format!("@gem cmd use '{}' troll", FIRE_SKILL),
            ]
        );
    }
}
