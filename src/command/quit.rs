use crate::command::{Command, Data};
use egui::ViewportCommand;

#[derive(Default)]
pub struct Quit {}

impl Command for Quit {
    fn process(&self, _data: &Data, ctx: &egui::Context) -> Option<String> {
        ctx.send_viewport_cmd(ViewportCommand::Close);

        None
    }
}
