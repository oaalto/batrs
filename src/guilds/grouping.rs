//! Guild membership groups derived from the typed Guild Catalog.
//! Thematic buckets are mutually exclusive for saved preferences; `background_multi` guilds overlap every thematic drill.

use std::sync::OnceLock;

use super::{
    GuildDefinition, catalog,
    catalog::{GuildGroupingClass, GuildKey},
    guild_definitions,
};

pub const MULTI_BACKGROUND_LABEL: &str = "Multi-Background";

/// Fixed browse order for thematic rows (exclusive primary among these five).
pub const THEMES_UX_ORDER: &[(&str, &str)] = &[
    ("civilized", "Civilized"),
    ("magical", "Magical"),
    ("good_religious", "Good Religious"),
    ("evil_religious", "Evil Religious"),
    ("nomad", "Nomad"),
];

pub fn thematic_index_for_keyword(keyword: &str) -> Option<usize> {
    THEMES_UX_ORDER
        .iter()
        .position(|(canonical, _)| *canonical == keyword)
}

fn playable_indices_for_grouping(
    grouping: GuildGroupingClass,
    defs: &[GuildDefinition],
) -> Vec<usize> {
    let mut out: Vec<usize> = defs
        .iter()
        .enumerate()
        .filter_map(|(index, definition)| {
            let entry = catalog::entry_for_key(definition.guild_key)?;
            (entry.grouping == grouping && entry.is_playable()).then_some(index)
        })
        .collect();
    out.sort_unstable();
    out
}

/// Static grouping data from playable Guild Catalog entries.
pub struct GuildGrouping {
    pub thematic: [ThematicBucket; 5],
    pub multi_playable_indices: Vec<usize>,
}

pub struct ThematicBucket {
    pub label: &'static str,
    pub playable_def_indices: Vec<usize>,
}

static GROUPING: OnceLock<GuildGrouping> = OnceLock::new();

pub fn guild_grouping() -> &'static GuildGrouping {
    GROUPING.get_or_init(|| {
        let defs = guild_definitions();
        let thematic = std::array::from_fn(|index| {
            let (_, label) = THEMES_UX_ORDER[index];
            ThematicBucket {
                label,
                playable_def_indices: playable_indices_for_grouping(
                    GuildGroupingClass::Thematic(index),
                    &defs,
                ),
            }
        });
        GuildGrouping {
            thematic,
            multi_playable_indices: playable_indices_for_grouping(GuildGroupingClass::Multi, &defs),
        }
    })
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildBucketClass {
    Thematic(usize),
    Multi,
}

/// Classify a guild definition key (batrs implementation keyword) for exclusivity rules.
pub fn classify_guild_key(def_key: &str) -> Option<GuildBucketClass> {
    match catalog::entry_for_persisted_key(def_key)?.grouping {
        GuildGroupingClass::Multi => Some(GuildBucketClass::Multi),
        GuildGroupingClass::Thematic(index) => Some(GuildBucketClass::Thematic(index)),
    }
}

pub fn classify_guild_key_typed(guild_key: GuildKey) -> Option<GuildBucketClass> {
    match catalog::entry_for_key(guild_key)?.grouping {
        GuildGroupingClass::Multi => Some(GuildBucketClass::Multi),
        GuildGroupingClass::Thematic(index) => Some(GuildBucketClass::Thematic(index)),
    }
}

pub fn visible_indices_multi_drill() -> Vec<usize> {
    guild_grouping().multi_playable_indices.clone()
}

/// Clear selected flags for thematic guilds outside `active_thematic`, keep multi and in-bucket thematic.
pub fn clear_selected_outside_thematic_bucket(
    definitions: &[GuildDefinition],
    selected: &mut [bool],
    active_thematic: usize,
) {
    for (index, definition) in definitions.iter().enumerate() {
        let Some(class) = classify_guild_key_typed(definition.guild_key) else {
            selected[index] = false;
            continue;
        };
        match class {
            GuildBucketClass::Multi => {}
            GuildBucketClass::Thematic(ix) => {
                if ix != active_thematic {
                    selected[index] = false;
                }
            }
        }
    }
}

pub const DEFAULT_GUILD_PRIMARY_KEYWORD: &str = "civilized";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disciple_classifies_as_multi_background() {
        assert!(matches!(
            classify_guild_key("disciple"),
            Some(GuildBucketClass::Multi)
        ));
    }

    #[test]
    fn multi_playable_contains_disciple_when_implemented() {
        let defs = guild_definitions();
        assert!(
            guild_grouping()
                .multi_playable_indices
                .iter()
                .any(|&ix| defs[ix].key == "disciple")
        );
    }

    #[test]
    fn multi_playable_contains_kharim_when_implemented() {
        let defs = guild_definitions();
        assert!(
            guild_grouping()
                .multi_playable_indices
                .iter()
                .any(|&ix| defs[ix].key == "kharim")
        );
    }
}
