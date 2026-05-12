use crate::abilities;
use crate::command;
use crate::command::Command;
use crate::guilds::ChannellersGuild;
use std::collections::HashMap;

impl ChannellersGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([("cdr".to_string(), Self::cast_drain_room as Command)])
    }

    pub fn cast_drain_room(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast drain room"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::command::CommandContext;

    fn data(cmd: &str, args: &str) -> command::Data {
        command::Data {
            cmd: cmd.to_string(),
            args: args.to_string(),
        }
    }

    fn empty_ctx() -> CommandContext {
        CommandContext::new(HashMap::new(), true, String::new())
    }

    #[test]
    fn drain_room_matches_tinyfugue_line() {
        let result = ChannellersGuild::cast_drain_room(&data("cdr", ""), &mut empty_ctx());
        assert_eq!(result, Some("@cast drain room".to_string()));
    }
}
