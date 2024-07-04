use crate::ansi::StyledLine;
use crate::stats::Stats;
use crate::triggers::Trigger;
use crate::BatApp;
use lazy_static::lazy_static;
use regex::Regex;

lazy_static! {
    //H:571/802 [+20] S:635/635 [] E:311/311 [] $:2786 [] exp:21657 []
    pub static ref SC_REGEX: Regex =
        Regex::new(r"^H:(.+)/(.+) \[(.*)\] S:(.+)/(.+) \[(.*)\] E:(.+)/(.+) \[(.*)\] \$:(.+) \[(.*)\] exp:(.+) \[(.*)\]$").unwrap();
}

#[derive(Default)]
pub struct ShortScoreTrigger {}

impl Trigger for ShortScoreTrigger {
    fn process(&self, app: &mut BatApp, styled_line: &mut StyledLine) {
        if let Some(captures) = SC_REGEX.captures(&styled_line.plain_line) {
            let (_, stats): (&str, [&str; 13]) = captures.extract();
            let stats = stats.map(|stat| stat.parse::<i32>().unwrap_or_default());
            app.stats = Stats::new_from_sc(stats);
            styled_line.gag = true;
        }
    }
}
