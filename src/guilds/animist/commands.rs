use crate::abilities;
use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::AnimistGuild;
use crate::guilds::animist::{
    CEREMONY_DONE_FLAG, CONJURING_MOUNT_FLAG, DISMISSING_MOUNT_FLAG, JOINING_SOUL_FLAG,
    SEPARATING_SOUL_FLAG, clear_pending_actions,
};
use std::collections::HashMap;

impl AnimistGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("cere".to_string(), Self::use_ceremony as Command),
            ("csoul".to_string(), Self::cast_separate_soul as Command),
            ("cjoin".to_string(), Self::cast_join_soul as Command),
            ("csum".to_string(), Self::cast_conjure_mount as Command),
            ("cdis".to_string(), Self::cast_dismiss_mount as Command),
        ])
    }

    pub fn use_ceremony(
        _data: &command::Data,
        _ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        command::send(abilities::client_send_line("use 'ceremony'"))
    }

    pub fn cast_separate_soul(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        cast_after_ceremony(ctx, SEPARATING_SOUL_FLAG, "cast 'separate soul'")
    }

    pub fn cast_join_soul(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        cast_after_ceremony(ctx, JOINING_SOUL_FLAG, "cast 'join soul'")
    }

    pub fn cast_conjure_mount(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        cast_after_ceremony(ctx, CONJURING_MOUNT_FLAG, "cast 'conjure animal soul'")
    }

    pub fn cast_dismiss_mount(
        _data: &command::Data,
        ctx: &command::CommandEnvironment,
    ) -> Vec<command::CommandEffect> {
        cast_after_ceremony(
            ctx,
            DISMISSING_MOUNT_FLAG,
            "cast 'animal soul link' dismiss",
        )
    }
}

fn cast_after_ceremony(
    ctx: &command::CommandEnvironment,
    pending_flag: &str,
    command: &str,
) -> Vec<command::CommandEffect> {
    if ctx.flag(CEREMONY_DONE_FLAG) {
        return command::send(abilities::client_send_line(command));
    }

    let mut effects = command::automations(clear_pending_actions());
    effects.push(command::automation(Action::SetFlag(
        pending_flag.to_string(),
        true,
    )));
    effects.extend(command::send(abilities::client_send_line("use 'ceremony'")));
    effects
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::guilds::animist::PENDING_FLAGS;

    fn data(cmd: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: String::new(),
        }
    }

    fn ctx(ceremony_done: bool) -> command::CommandEnvironment {
        command::CommandEnvironment::new(
            HashMap::from([(CEREMONY_DONE_FLAG.to_string(), ceremony_done)]),
            HashMap::new(),
        )
    }

    #[test]
    fn ceremony_alias_uses_ceremony() {
        let ctx = ctx(false);

        assert_eq!(
            AnimistGuild::use_ceremony(&data("cere"), &ctx),
            command::send("@use 'ceremony'".to_string())
        );
    }

    #[test]
    fn cast_runs_directly_when_ceremony_is_done() {
        let ctx = ctx(true);

        assert_eq!(
            AnimistGuild::cast_separate_soul(&data("csoul"), &ctx),
            command::send("@cast 'separate soul'".to_string())
        );
    }

    #[test]
    fn cast_queues_ceremony_when_needed() {
        let ctx = ctx(false);

        let effects = AnimistGuild::cast_conjure_mount(&data("csum"), &ctx);
        assert!(effects.contains(&command::CommandEffect::Send("@use 'ceremony'".to_string())));
        let actions: Vec<&Action> = effects
            .iter()
            .filter_map(|effect| match effect {
                command::CommandEffect::Automation(action) => Some(action),
                _ => None,
            })
            .collect();
        assert_eq!(actions.len(), PENDING_FLAGS.len() + 1);
        assert!(matches!(
            actions.last(),
            Some(Action::SetFlag(flag, true)) if flag == CONJURING_MOUNT_FLAG
        ));
    }
}
