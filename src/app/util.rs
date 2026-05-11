use crate::guilds::guild_definitions;
use chrono::{DateTime, Local, Timelike};

pub(crate) fn show_clock() -> String {
    let local: DateTime<Local> = Local::now();
    format!(
        "{:02}:{:02}:{:02}",
        local.hour(),
        local.minute(),
        local.second()
    )
}

pub(crate) fn filter_known_guilds(keys: Vec<String>) -> Vec<String> {
    let definitions = guild_definitions();
    keys.into_iter()
        .filter(|key| definitions.iter().any(|def| def.key == key))
        .collect()
}
