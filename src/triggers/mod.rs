use crate::ansi::StyledLine;
use crate::guilds::Guild;
use crate::stats::Stats;
use lazy_static::lazy_static;

mod prompt;
mod short_score;

lazy_static! {
    static ref TRIGGERS: Vec<Trigger> = vec![prompt::trigger, short_score::trigger];
}

pub struct TriggerContext<'a> {
    pub stats: &'a mut Stats,
}

pub type Trigger =
    fn(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> Vec<StyledLine>;

pub fn process(
    ctx: &mut TriggerContext<'_>,
    guilds: &[Box<dyn Guild>],
    styled_line: &mut StyledLine,
) -> Vec<StyledLine> {
    let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();

    guild_triggers
        .iter()
        .chain(TRIGGERS.iter())
        .flat_map(|trigger| trigger(ctx, styled_line))
        .collect()
}
