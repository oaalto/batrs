use crate::ansi::StyledLine;
use crate::command::{CommandContext, Data};

const HELP_LINES: [&str; 7] = [
    "Client slash commands:",
    "/help - Shows client slash commands.",
    "/quit - Closes the client.",
    "/guilds - Opens the guild picker.",
    "/generic - Opens generic shortcut groups.",
    "/settings - Opens the settings editor.",
    "/raw_logs - Toggles raw log capture.",
];

pub fn run(_data: &Data, ctx: &mut CommandContext) -> Option<String> {
    for line in HELP_LINES {
        ctx.push_output_line(StyledLine::new(line));
    }

    None
}
