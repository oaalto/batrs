use crate::ansi::StyledLine;
use crate::BatApp;
use lazy_static::lazy_static;

mod prompt;
mod short_score;

lazy_static! {
    static ref TRIGGERS: Vec<Trigger> = vec![prompt::trigger, short_score::trigger];
}

pub type Trigger = fn(app: &mut BatApp, styled_line: &mut StyledLine);

// TODO: Add a way to add new lines to output
pub fn process(app: &mut BatApp, styled_line: &mut StyledLine) {
    TRIGGERS
        .iter()
        .for_each(|trigger| trigger(app, styled_line));

    let guild_triggers: Vec<Trigger> = app
        .selected_guilds
        .iter()
        .flat_map(|g| g.triggers())
        .collect();
    guild_triggers
        .iter()
        .for_each(|trigger| trigger(app, styled_line))
}
