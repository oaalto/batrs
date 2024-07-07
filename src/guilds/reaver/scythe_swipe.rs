use crate::command;
use crate::command::Command;
use egui::Context;

#[derive(Default)]
pub struct ScytheSwipe {}

impl Command for ScytheSwipe {
    fn process(&self, data: &command::Data, _ctx: &Context) -> Option<String> {
        let cmd = if data.args.is_empty() {
            String::from("@use 'scythe swipe'")
        } else {
            let target = data.args.join(" ");
            format!("@target {};use 'scythe swipe' {}", &target, &target)
        };

        Some(cmd)
    }
}
