use crate::automation::Action;
use crate::command::{CommandContext, Data};

pub fn run(data: &Data, ctx: &mut CommandContext) -> Option<String> {
    if data.args.is_empty() {
        return None;
    }

    ctx.push_action(Action::SetVar("rig".to_string(), data.args.clone()));
    ctx.set_rig(data.args.clone());
    None
}
