use crate::command::Data;
use crate::command::CommandContext;

pub fn run(_data: &Data, ctx: &mut CommandContext) -> Option<String> {
    ctx.should_quit = true;

    None
}
