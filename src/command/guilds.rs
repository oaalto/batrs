use crate::command::{CommandContext, Data};

pub fn run(_data: &Data, ctx: &mut CommandContext) -> Option<String> {
    ctx.open_guilds_dialog();
    None
}
