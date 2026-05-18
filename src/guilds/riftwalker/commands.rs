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
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        set_entity_skill_effects(FIRE_SKILL, "fire")
    }

    pub fn set_skill_air(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        set_entity_skill_effects(AIR_SKILL, "air")
    }

    pub fn set_skill_earth(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        set_entity_skill_effects(EARTH_SKILL, "earth")
    }

    pub fn set_skill_water(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        set_entity_skill_effects(WATER_SKILL, "water")
    }

    pub fn cmd_use_current_skill(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        target_gem_chain_effects(data.args.trim())
    }

    pub fn cmd_gem_entity(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        if args.is_empty() {
            command::automations([Action::Send(abilities::client_send_line(&format!(
                "gem entities {{{}}}",
                RIFTWALKER_ELEMENT_VAR
            )))])
        } else {
            command::automations([Action::Send(abilities::client_send_line(&format!(
                "gem entities {args}"
            )))])
        }
    }

    pub fn cmd_summon_entity(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        let mut effects = command::automations([Action::Send(abilities::client_send_line(
            &format!("cast 'summon rift entity' {args}"),
        ))]);
        effects.extend(element_from_free_text_effects(args));
        effects.push(command::automation(Action::SetFlag(
            RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
            true,
        )));
        effects
    }

    pub fn cmd_dismiss_entity(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        vec![
            command::automation(Action::SetFlag(
                RIFTWALKER_HAS_ENTITY_FLAG.to_string(),
                false,
            )),
            command::CommandEffect::Send(abilities::client_send_line("cast 'dismiss rift entity'")),
        ]
    }

    pub fn cmd_beckon_entity(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'beckon rift entity'"))
    }

    pub fn cmd_entity_control(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            command::send(abilities::client_send_line(
                "cast 'establish entity control'",
            ))
        } else {
            command::send(abilities::client_send_line(&format!(
                "cast 'establish entity control' {}",
                data.args.trim()
            )))
        }
    }

    pub fn cmd_entity_control_long(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line(
            "cast 'establish entity control' 10",
        ))
    }

    pub fn cmd_entity_regen(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'regenerate rift entity'"))
    }

    pub fn cmd_transform_entity(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let args = data.args.trim();
        match args.split_once(char::is_whitespace) {
            Some((target_at, rest)) => {
                let mut effects =
                    command::automations([Action::Send(abilities::client_send_line(&format!(
                        "cast 'transform rift entity' {target_at}"
                    )))]);
                let src = rest.trim();
                effects.extend(element_from_free_text_effects(if src.is_empty() {
                    target_at
                } else {
                    src
                }));
                effects
            }
            None if !args.is_empty() => {
                let mut effects = command::automations([Action::Send(
                    abilities::client_send_line(&format!("cast 'transform rift entity' {args}")),
                )]);
                effects.extend(element_from_free_text_effects(args));
                effects
            }
            None => {
                vec![command::output(StyledLine::new(
                    "Riftwalker: cte expects a target direction (and optional element text).",
                ))]
            }
        }
    }

    pub fn cmd_start_spark_birth(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        opening_battle_effects(data.args.trim(), "spark birth")
    }

    pub fn cmd_start_rift_pulse(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        opening_battle_effects(data.args.trim(), "rift pulse")
    }

    pub fn cmd_start_dimensional_leech(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        opening_battle_effects(data.args.trim(), "dimensional leech")
    }

    pub fn cmd_cast_spark_birth(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("spark birth", data))
    }

    pub fn cmd_cast_rift_pulse(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("rift pulse", data))
    }

    pub fn cmd_cast_dimensional_leech(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(cast_spell("dimensional leech", data))
    }

    pub fn cmd_cast_force_absorption(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            command::send(abilities::client_send_line(
                "cast 'force absorption' entity",
            ))
        } else {
            command::send(cast_spell("force absorption", data))
        }
    }

    pub fn cmd_cast_mirror_image_entity(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'mirror image' entity"))
    }

    pub fn cmd_cast_absorbing_meld(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("cast 'Absorbing meld'"))
    }

    pub fn cmd_cast_iron_will(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        if data.args.trim().is_empty() {
            command::send(abilities::client_send_line("cast 'iron will' entity"))
        } else {
            command::send(cast_spell("iron will", data))
        }
    }

    pub fn cmd_stop_use(
        _data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::compound_send(&["zz", "gem cmd use stop"]))
    }

    pub fn cmd_gem_wield(
        data: &command::Data,
        __ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line(&format!(
            "gem cmd wield {}",
            data.args.trim()
        )))
    }

    pub fn cmd_diag(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let has = ctx.flag(RIFTWALKER_HAS_ENTITY_FLAG);
        vec![command::output(StyledLine::new(&format!(
            "Riftwalker: has_entity={has} (per-element labels: /guilds)",
        )))]
    }

    pub fn cmd_fix(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        vec![command::output(StyledLine::new(
            "Riftwalker: external borg/hook toggles from third-party scripts do not apply in Batrs.",
        ))]
    }
}

fn set_entity_skill_effects(skill: &str, element: &str) -> Vec<command::CommandEffect> {
    command::automations([
        Action::SetVar(RIFTWALKER_SKILL_VAR.to_string(), skill.to_string()),
        Action::SetVar(RIFTWALKER_ELEMENT_VAR.to_string(), element.to_string()),
        Action::SetFlag(RIFTWALKER_HAS_ENTITY_FLAG.to_string(), true),
    ])
}

fn element_from_free_text_effects(blob: &str) -> Vec<command::CommandEffect> {
    let lowered = blob.to_ascii_lowercase();
    if lowered.contains("fire") {
        set_entity_skill_effects(FIRE_SKILL, "fire")
    } else if lowered.contains("air") {
        set_entity_skill_effects(AIR_SKILL, "air")
    } else if lowered.contains("earth") {
        set_entity_skill_effects(EARTH_SKILL, "earth")
    } else if lowered.contains("water") {
        set_entity_skill_effects(WATER_SKILL, "water")
    } else {
        Vec::new()
    }
}

fn target_gem_chain_effects(tail: &str) -> Vec<command::CommandEffect> {
    let trimmed = tail.trim();
    let mut effects = Vec::new();
    if !trimmed.is_empty() {
        effects.push(command::automation(Action::Send(
            abilities::client_send_line(&format!("target {trimmed};gem cmd target {trimmed}")),
        )));
    }
    let gem = if trimmed.is_empty() {
        abilities::client_send_line(&format!("gem cmd use '{{{}}}'", RIFTWALKER_SKILL_VAR))
    } else {
        abilities::client_send_line(&format!(
            "gem cmd use '{{{}}}' {}",
            RIFTWALKER_SKILL_VAR, trimmed
        ))
    };
    effects.push(command::automation(Action::Send(gem)));
    effects
}

fn opening_battle_effects(tail: &str, spell: &str) -> Vec<command::CommandEffect> {
    let trimmed = tail.trim();
    let mut effects = Vec::new();
    if !trimmed.is_empty() {
        effects.push(command::automation(Action::Send(
            abilities::client_send_line(&format!("target {trimmed};gem cmd target {trimmed}")),
        )));
    }
    let cast_send = abilities::cast_quoted_with_suffix(spell, trimmed);
    effects.push(command::automation(Action::Send(cast_send)));
    let gem_send = if trimmed.is_empty() {
        abilities::client_send_line(&format!("gem cmd use '{{{}}}'", RIFTWALKER_SKILL_VAR))
    } else {
        abilities::client_send_line(&format!(
            "gem cmd use '{{{}}}' {}",
            RIFTWALKER_SKILL_VAR, trimmed
        ))
    };
    effects.push(command::automation(Action::Send(gem_send)));
    effects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Automation;
    use crate::command::{CommandEnvironment, Data};
    use crate::guilds::riftwalker::RIFTWALKER_HAS_ENTITY_FLAG;

    #[test]
    fn dismiss_queues_flag_then_sends_on_separate_application() {
        let env = CommandEnvironment::empty();
        let outcome = RiftwalkerGuild::cmd_dismiss_entity(
            &Data {
                cmd: "cdis".to_string(),
                args: String::new(),
            },
            &env,
        );
        assert!(matches!(
            outcome.as_slice(),
            [
                command::CommandEffect::Automation(Action::SetFlag(flag, false)),
                command::CommandEffect::Send(command),
            ] if flag == RIFTWALKER_HAS_ENTITY_FLAG && command == "@cast 'dismiss rift entity'"
        ));
    }

    #[test]
    fn opening_battle_with_target_splits_cast_and_skill() {
        let env = CommandEnvironment::empty();
        let mut automation = Automation::new();
        automation.set_var(RIFTWALKER_SKILL_VAR, FIRE_SKILL.to_string());
        let effects = RiftwalkerGuild::cmd_start_spark_birth(
            &Data {
                cmd: "cs".to_string(),
                args: "troll".to_string(),
            },
            &env,
        );
        let actions: Vec<Action> = effects
            .into_iter()
            .filter_map(|effect| match effect {
                command::CommandEffect::Automation(action) => Some(action),
                _ => None,
            })
            .collect();
        let sends = automation.apply_actions(actions);
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
