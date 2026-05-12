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
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("use 'ceremony'"))
    }

    pub fn cast_separate_soul(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        cast_after_ceremony(ctx, SEPARATING_SOUL_FLAG, "cast 'separate soul'")
    }

    pub fn cast_join_soul(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        cast_after_ceremony(ctx, JOINING_SOUL_FLAG, "cast 'join soul'")
    }

    pub fn cast_conjure_mount(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        cast_after_ceremony(ctx, CONJURING_MOUNT_FLAG, "cast 'conjure animal soul'")
    }

    pub fn cast_dismiss_mount(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        cast_after_ceremony(
            ctx,
            DISMISSING_MOUNT_FLAG,
            "cast 'animal soul link' at dismiss",
        )
    }
}

fn cast_after_ceremony(
    ctx: &mut command::CommandContext,
    pending_flag: &str,
    command: &str,
) -> Option<String> {
    if ctx.flag(CEREMONY_DONE_FLAG) {
        return Some(abilities::client_send_line(command));
    }

    for action in clear_pending_actions() {
        ctx.push_action(action);
    }
    ctx.push_action(Action::SetFlag(pending_flag.to_string(), true));
    Some(abilities::client_send_line("use 'ceremony'"))
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

    fn ctx(ceremony_done: bool) -> command::CommandContext {
        command::CommandContext::new(
            HashMap::from([(CEREMONY_DONE_FLAG.to_string(), ceremony_done)]),
            true,
        )
    }

    #[test]
    fn ceremony_alias_uses_ceremony() {
        let mut ctx = ctx(false);

        assert_eq!(
            AnimistGuild::use_ceremony(&data("cere"), &mut ctx),
            Some("@use 'ceremony'".to_string())
        );
    }

    #[test]
    fn cast_runs_directly_when_ceremony_is_done() {
        let mut ctx = ctx(true);

        assert_eq!(
            AnimistGuild::cast_separate_soul(&data("csoul"), &mut ctx),
            Some("@cast 'separate soul'".to_string())
        );
        assert!(ctx.automation_actions.is_empty());
    }

    #[test]
    fn cast_queues_ceremony_when_needed() {
        let mut ctx = ctx(false);

        assert_eq!(
            AnimistGuild::cast_conjure_mount(&data("csum"), &mut ctx),
            Some("@use 'ceremony'".to_string())
        );
        assert_eq!(ctx.automation_actions.len(), PENDING_FLAGS.len() + 1);
        assert!(matches!(
            ctx.automation_actions.last(),
            Some(Action::SetFlag(flag, true)) if flag == CONJURING_MOUNT_FLAG
        ));
    }
}
