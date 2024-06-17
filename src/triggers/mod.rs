use crate::ansi::StyledLine;
use crate::BatApp;
use lazy_static::lazy_static;

mod prompt;

pub use prompt::PromptTrigger;

pub trait Trigger {
    fn process(&self, app: &mut BatApp, styled_line: &mut StyledLine);
}

lazy_static! {
    pub static ref TRIGGERS: Vec<Box<(dyn Trigger + Sync)>> =
        vec![Box::new(PromptTrigger::default())];
}
