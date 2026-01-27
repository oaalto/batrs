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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::automation::Action;
    use std::collections::HashMap;

    #[test]
    fn run_sets_rig_and_action() {
        let data = Data {
            cmd: "/rig".to_string(),
            args: "bag".to_string(),
        };
        let mut ctx = CommandContext::new(HashMap::new(), true);

        let result = run(&data, &mut ctx);

        assert!(result.is_none());
        assert_eq!(ctx.set_rig, Some("bag".to_string()));
        assert_eq!(ctx.automation_actions.len(), 1);
        match &ctx.automation_actions[0] {
            Action::SetVar(key, value) => {
                assert_eq!(key, "rig");
                assert_eq!(value, "bag");
            }
            _ => panic!("expected SetVar action"),
        }
    }
}
