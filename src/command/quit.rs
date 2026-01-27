use crate::command::CommandContext;
use crate::command::Data;

pub fn run(_data: &Data, ctx: &mut CommandContext) -> Option<String> {
    ctx.should_quit = true;

    None
}
