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

    pub fn use_kiai_cry(data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        reset_current_skills(ctx);
        if data.args.is_empty() {
            None
        } else {
            Some(format!(
                "@target {};use kiai-cry at {}",
                data.args, data.args
            ))
        }
    }

    pub fn use_joint_lock(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        Some(use_skill("joint lock", data))
    }

    pub fn use_pattern_weave(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        Some(use_skill("pattern weave", data))
    }

    pub fn use_skulking(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        Some("@use skulking".to_string())
    }

    pub fn use_iron_palm(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        Some(use_skill("iron palm", data))
    }

    pub fn use_kata(_data: &command::Data, ctx: &mut command::CommandContext) -> Option<String> {
        reset_current_skills(ctx);
        Some("@use kata".to_string())
    }

    pub fn use_meditation(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        if ctx.flag(KATA_DONE_FLAG) {
            Some("@use meditation".to_string())
        } else {
            ctx.push_action(Action::SetFlag(DOING_MEDITATION_FLAG.to_string(), true));
            Some("@use kata".to_string())
        }
    }

    pub fn use_mind_over_body(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        reset_current_skills(ctx);
        if data.args.is_empty() {
            Some("@use mind over body".to_string())
        } else {
            Some(format!("@use mind over body at {}", data.args))
        }
    }

    pub fn do_disrupt_skill(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        send_current_skill(ctx, CURRENT_DISRUPT_SKILL_VAR, &data.args);
        None
    }

    pub fn do_area_skill(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        send_current_skill(ctx, CURRENT_AREA_SKILL_VAR, &data.args);
        None
    }

    pub fn do_armour_skill(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        send_current_skill(ctx, CURRENT_ARMOUR_SKILL_VAR, &data.args);
        None
    }

    pub fn do_avoid_skill(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        send_current_skill(ctx, CURRENT_AVOID_SKILL_VAR, &data.args);
        None
    }

    pub fn use_wave_crest_strike(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        ctx.push_action(Action::SetVar(
            CURRENT_DISRUPT_SKILL_VAR.to_string(),
            DISRUPT_SKILL_1.to_string(),
        ));
        send_current_skill(ctx, CURRENT_DISRUPT_SKILL_VAR, &data.args);
        None
    }

    pub fn use_geyser_force_kick(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        ctx.push_action(Action::SetVar(
            CURRENT_DISRUPT_SKILL_VAR.to_string(),
            DISRUPT_SKILL_2.to_string(),
        ));
        send_current_skill(ctx, CURRENT_DISRUPT_SKILL_VAR, &data.args);
        None
    }

    pub fn use_earthquake_kick(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        ctx.push_action(Action::SetVar(
            CURRENT_ARMOUR_SKILL_VAR.to_string(),
            ARMOUR_SKILL_2.to_string(),
        ));
        send_current_skill(ctx, CURRENT_ARMOUR_SKILL_VAR, &data.args);
        None
    }

    pub fn use_avalanche_slam(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        ctx.push_action(Action::SetVar(
            CURRENT_ARMOUR_SKILL_VAR.to_string(),
            ARMOUR_SKILL_3.to_string(),
        ));
        send_current_skill(ctx, CURRENT_ARMOUR_SKILL_VAR, &data.args);
        None
    }
}

pub fn reset_current_skills(ctx: &mut command::CommandContext) {
    for (key, value) in default_skill_vars() {
        ctx.push_action(Action::SetVar(key.to_string(), value.to_string()));
    }
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

fn send_current_skill(ctx: &mut command::CommandContext, var: &str, target: &str) {
    let skill_template = format!("{{{var}}}");
    let command = if target.is_empty() {
        format!("@use '{skill_template}'")
    } else {
        format!("@target {target};@use '{skill_template}' {target}")
    };
    ctx.push_action(Action::Send(command));
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

    fn ctx_with_flag(flag: bool) -> command::CommandContext {
        command::CommandContext::new(HashMap::from([(KATA_DONE_FLAG.to_string(), flag)]), true)
    }

    #[test]
    fn kiai_cry_requires_target_and_resets_skills() {
        let mut ctx = ctx_with_flag(false);
        let result = MonkGuild::use_kiai_cry(&data("cs", "orc"), &mut ctx);

        assert_eq!(result, Some("@target orc;use kiai-cry at orc".to_string()));
        assert_eq!(ctx.automation_actions.len(), 4);
    }

    #[test]
    fn meditation_uses_kata_until_kata_is_done() {
        let mut ctx = ctx_with_flag(false);
        let result = MonkGuild::use_meditation(&data("med", ""), &mut ctx);

        assert_eq!(result, Some("@use kata".to_string()));
        assert!(ctx.automation_actions.iter().any(|action| matches!(
            action,
            Action::SetFlag(flag, true) if flag == DOING_MEDITATION_FLAG
        )));
    }

    #[test]
    fn meditation_runs_directly_after_kata_is_done() {
        let mut ctx = ctx_with_flag(true);
        let result = MonkGuild::use_meditation(&data("med", ""), &mut ctx);

        assert_eq!(result, Some("@use meditation".to_string()));
    }

    #[test]
    fn advanced_skill_alias_sets_var_and_sends_template() {
        let mut ctx = ctx_with_flag(false);
        let result = MonkGuild::use_geyser_force_kick(&data("ugk", "troll"), &mut ctx);

        assert!(result.is_none());
        assert!(matches!(
            &ctx.automation_actions[0],
            Action::SetVar(key, value)
                if key == CURRENT_DISRUPT_SKILL_VAR && value == DISRUPT_SKILL_2
        ));
        assert!(matches!(
            &ctx.automation_actions[1],
            Action::Send(command)
                if command == "@target troll;@use '{monk_current_disrupt_skill}' troll"
        ));
    }

    #[test]
    fn default_skill_values_match_active_tf_settings() {
        assert_eq!(ARMOUR_SKILL_3, ARMOUR_SKILL_1);
        assert_eq!(AREA_SKILL_3, AREA_SKILL_1);
        assert_eq!(AVOID_SKILL_3, AVOID_SKILL_1);
        assert_ne!(DISRUPT_SKILL_3, DISRUPT_SKILL_1);
        assert_ne!(AREA_SKILL_2, AREA_SKILL_1);
        assert_ne!(AVOID_SKILL_2, AVOID_SKILL_1);
    }
}
