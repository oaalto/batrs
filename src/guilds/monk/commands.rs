use crate::abilities;
use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::monk::{
    AREA_SKILL_1, ARMOUR_SKILL_1, ARMOUR_SKILL_2, ARMOUR_SKILL_3, AVOID_SKILL_1,
    CURRENT_AREA_SKILL_VAR, CURRENT_ARMOUR_SKILL_VAR, CURRENT_AVOID_SKILL_VAR,
    CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1, DISRUPT_SKILL_2, DOING_MEDITATION_FLAG,
    KATA_DONE_FLAG,
};
use crate::guilds::{MonkGuild, use_skill};
use std::collections::HashMap;

impl MonkGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cs".to_string(), Self::use_kiai_cry as Command),
            ("ujl".to_string(), Self::use_joint_lock as Command),
            ("upw".to_string(), Self::use_pattern_weave as Command),
            ("usk".to_string(), Self::use_skulking as Command),
            ("ip".to_string(), Self::use_iron_palm as Command),
            ("kata".to_string(), Self::use_kata as Command),
            ("med".to_string(), Self::use_meditation as Command),
            ("umb".to_string(), Self::use_mind_over_body as Command),
            ("uds".to_string(), Self::do_disrupt_skill as Command),
            ("uaa".to_string(), Self::do_area_skill as Command),
            ("uar".to_string(), Self::do_armour_skill as Command),
            ("uav".to_string(), Self::do_avoid_skill as Command),
            ("uws".to_string(), Self::use_wave_crest_strike as Command),
            ("ugk".to_string(), Self::use_geyser_force_kick as Command),
            ("uek".to_string(), Self::use_earthquake_kick as Command),
            ("uas".to_string(), Self::use_avalanche_slam as Command),
        ])
    }

    pub fn use_kiai_cry(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        if data.args.is_empty() {
            effects
        } else {
            effects.extend(command::send(abilities::client_send_line(&format!(
                "target {};use 'kiai-cry' {}",
                data.args, data.args
            ))));
            effects
        }
    }

    pub fn use_joint_lock(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        effects.extend(command::send(use_skill("joint lock", data)));
        effects
    }

    pub fn use_pattern_weave(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        effects.extend(command::send(use_skill("pattern weave", data)));
        effects
    }

    pub fn use_skulking(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        effects.extend(command::send(abilities::client_send_line("use 'skulking'")));
        effects
    }

    pub fn use_iron_palm(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        effects.extend(command::send(use_skill("iron palm", data)));
        effects
    }

    pub fn use_kata(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        effects.extend(command::send(abilities::client_send_line("use 'kata'")));
        effects
    }

    pub fn use_meditation(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        if ctx.flag(KATA_DONE_FLAG) {
            effects.extend(command::send(abilities::client_send_line(
                "use 'meditation'",
            )));
        } else {
            effects.push(command::automation(Action::SetFlag(
                DOING_MEDITATION_FLAG.to_string(),
                true,
            )));
            effects.extend(command::send(abilities::client_send_line("use 'kata'")));
        }
        effects
    }

    pub fn use_mind_over_body(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = reset_current_skill_effects();
        if data.args.is_empty() {
            effects.extend(command::send(abilities::client_send_line(
                "use 'mind over body'",
            )));
        } else {
            effects.extend(command::send(abilities::client_send_line(&format!(
                "use 'mind over body' {}",
                data.args
            ))));
        }
        effects
    }

    pub fn do_disrupt_skill(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        current_skill_effects(CURRENT_DISRUPT_SKILL_VAR, &data.args)
    }

    pub fn do_area_skill(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        current_skill_effects(CURRENT_AREA_SKILL_VAR, &data.args)
    }

    pub fn do_armour_skill(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        current_skill_effects(CURRENT_ARMOUR_SKILL_VAR, &data.args)
    }

    pub fn do_avoid_skill(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        current_skill_effects(CURRENT_AVOID_SKILL_VAR, &data.args)
    }

    pub fn use_wave_crest_strike(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = vec![command::automation(Action::SetVar(
            CURRENT_DISRUPT_SKILL_VAR.to_string(),
            DISRUPT_SKILL_1.to_string(),
        ))];
        effects.extend(current_skill_effects(CURRENT_DISRUPT_SKILL_VAR, &data.args));
        effects
    }

    pub fn use_geyser_force_kick(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = vec![command::automation(Action::SetVar(
            CURRENT_DISRUPT_SKILL_VAR.to_string(),
            DISRUPT_SKILL_2.to_string(),
        ))];
        effects.extend(current_skill_effects(CURRENT_DISRUPT_SKILL_VAR, &data.args));
        effects
    }

    pub fn use_earthquake_kick(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = vec![command::automation(Action::SetVar(
            CURRENT_ARMOUR_SKILL_VAR.to_string(),
            ARMOUR_SKILL_2.to_string(),
        ))];
        effects.extend(current_skill_effects(CURRENT_ARMOUR_SKILL_VAR, &data.args));
        effects
    }

    pub fn use_avalanche_slam(
        data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        let mut effects = vec![command::automation(Action::SetVar(
            CURRENT_ARMOUR_SKILL_VAR.to_string(),
            ARMOUR_SKILL_3.to_string(),
        ))];
        effects.extend(current_skill_effects(CURRENT_ARMOUR_SKILL_VAR, &data.args));
        effects
    }
}

pub fn reset_current_skill_effects() -> Vec<command::CommandEffect> {
    command::automations(reset_current_skill_actions())
}

pub fn reset_current_skill_actions() -> Vec<Action> {
    default_skill_vars()
        .into_iter()
        .map(|(key, value)| Action::SetVar(key.to_string(), value.to_string()))
        .collect()
}

fn default_skill_vars() -> [(&'static str, &'static str); 4] {
    [
        (CURRENT_ARMOUR_SKILL_VAR, ARMOUR_SKILL_1),
        (CURRENT_DISRUPT_SKILL_VAR, DISRUPT_SKILL_1),
        (CURRENT_AREA_SKILL_VAR, AREA_SKILL_1),
        (CURRENT_AVOID_SKILL_VAR, AVOID_SKILL_1),
    ]
}

fn current_skill_effects(var: &str, target: &str) -> Vec<command::CommandEffect> {
    let skill_template = format!("{{{var}}}");
    let command = if target.is_empty() {
        abilities::client_send_line(&format!("use '{skill_template}'"))
    } else {
        abilities::client_send_line(&format!("target {target};use '{skill_template}' {target}"))
    };
    vec![command::automation(Action::Send(command))]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guilds::monk::{
        AREA_SKILL_2, AREA_SKILL_3, AVOID_SKILL_2, AVOID_SKILL_3, DISRUPT_SKILL_3,
    };

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn ctx_with_flag(flag: bool) -> command::CommandEnvironment {
        command::CommandEnvironment::new(
            HashMap::from([(KATA_DONE_FLAG.to_string(), flag)]),
            HashMap::new(),
        )
    }

    fn automation_actions(effects: &[command::CommandEffect]) -> Vec<&Action> {
        effects
            .iter()
            .filter_map(|effect| match effect {
                command::CommandEffect::Automation(action) => Some(action),
                _ => None,
            })
            .collect()
    }

    #[test]
    fn kiai_cry_requires_target_and_resets_skills() {
        let ctx = ctx_with_flag(false);
        let result = MonkGuild::use_kiai_cry(&data("cs", "orc"), &ctx);

        assert!(result.contains(&command::CommandEffect::Send(
            "@target orc;use 'kiai-cry' orc".to_string()
        )));
        assert_eq!(automation_actions(&result).len(), 4);
    }

    #[test]
    fn meditation_uses_kata_until_kata_is_done() {
        let ctx = ctx_with_flag(false);
        let result = MonkGuild::use_meditation(&data("med", ""), &ctx);

        assert!(result.contains(&command::CommandEffect::Send("@use 'kata'".to_string())));
        assert!(automation_actions(&result).iter().any(|action| matches!(
            *action,
            Action::SetFlag(flag, true) if flag == DOING_MEDITATION_FLAG
        )));
    }

    #[test]
    fn meditation_runs_directly_after_kata_is_done() {
        let ctx = ctx_with_flag(true);
        let result = MonkGuild::use_meditation(&data("med", ""), &ctx);

        assert!(result.contains(&command::CommandEffect::Send(
            "@use 'meditation'".to_string()
        )));
    }

    #[test]
    fn advanced_skill_alias_sets_var_and_sends_template() {
        let ctx = ctx_with_flag(false);
        let result = MonkGuild::use_geyser_force_kick(&data("ugk", "troll"), &ctx);
        let actions = automation_actions(&result);

        assert!(matches!(
            actions[0],
            Action::SetVar(key, value)
                if key == CURRENT_DISRUPT_SKILL_VAR && value == DISRUPT_SKILL_2
        ));
        assert!(matches!(
            actions[1],
            Action::Send(command)
                if command == "@target troll;use '{monk_current_disrupt_skill}' troll"
        ));
    }

    #[test]
    fn default_skill_values_match_active_profile() {
        assert_eq!(ARMOUR_SKILL_3, ARMOUR_SKILL_1);
        assert_eq!(AREA_SKILL_3, AREA_SKILL_1);
        assert_eq!(AVOID_SKILL_3, AVOID_SKILL_1);
        assert_ne!(DISRUPT_SKILL_3, DISRUPT_SKILL_1);
        assert_ne!(AREA_SKILL_2, AREA_SKILL_1);
        assert_ne!(AVOID_SKILL_2, AVOID_SKILL_1);
    }
}
