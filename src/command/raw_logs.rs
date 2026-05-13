use crate::command::{CommandContext, Data};

pub fn run(_data: &Data, ctx: &mut CommandContext) -> Option<String> {
    ctx.toggle_raw_logs();
    None
}
