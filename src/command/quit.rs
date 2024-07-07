use crate::command::Data;
use egui::ViewportCommand;

pub fn run(_data: &Data, ctx: &egui::Context) -> Option<String> {
    ctx.send_viewport_cmd(ViewportCommand::Close);

    None
}
