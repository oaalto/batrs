use crate::ansi::StyledLine;
use crate::automation::{Action, Automation};
use crate::guilds::Guild;
use crate::stats::Stats;
use lazy_static::lazy_static;

mod combat_round;
mod common;
mod prompt;
mod recovery_bracket;
mod short_score;
mod spell_vocal_data;
mod spell_vocals;

lazy_static! {
    static ref COMMON_TRIGGERS: Vec<Trigger> = vec![common::trigger];
    static ref CORE_TRIGGERS: Vec<Trigger> = vec![
        combat_round::trigger,
        prompt::trigger,
        short_score::trigger,
        recovery_bracket::trigger,
    ];
}

#[derive(Default)]
pub struct TriggerOutput {
    pub lines: Vec<StyledLine>,
    pub actions: Vec<Action>,
}

pub struct TriggerContext<'a> {
    pub stats: &'a mut Stats,
    pub automation: &'a mut Automation,
    pub rig: Option<&'a str>,
    /// Logged-in character name; used for soul-companion combat lines and similar.
    pub player_name: Option<&'a str>,
}

pub type Trigger = fn(ctx: &mut TriggerContext<'_>, styled_line: &mut StyledLine) -> TriggerOutput;

pub fn process(
    ctx: &mut TriggerContext<'_>,
    guilds: &[Box<dyn Guild>],
    styled_line: &mut StyledLine,
) -> TriggerOutput {
    let guild_triggers: Vec<Trigger> = guilds.iter().flat_map(|g| g.triggers()).collect();
    let mut output = TriggerOutput::default();

    // Guild triggers first so stats hooks (e.g. Animist soul companion) always run before spell labels and common rules.
    for trigger in guild_triggers.iter() {
        let result = trigger(ctx, styled_line);
        output.lines.extend(result.lines);
        output.actions.extend(result.actions);
    }

    spell_vocals::annotate(styled_line);

    for trigger in COMMON_TRIGGERS.iter().chain(CORE_TRIGGERS.iter()) {
        let result = trigger(ctx, styled_line);
        output.lines.extend(result.lines);
        output.actions.extend(result.actions);
    }

    output
}
