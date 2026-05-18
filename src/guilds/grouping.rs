//! Guild membership groups derived from the typed Guild Catalog.
//! Thematic buckets are mutually exclusive for saved preferences; `background_multi` guilds overlap every thematic drill.

use std::sync::OnceLock;

use super::catalog::{self, GuildCatalogEntry, GuildGroupingClass};

pub const MULTI_BACKGROUND_LABEL: &str = "Multi-Background";
pub use catalog::{
    DEFAULT_GUILD_PRIMARY_KEYWORD, GuildBucketClass, THEMES_UX_ORDER, classify_guild_key_typed,
    thematic_index_for_keyword,
};

fn playable_indices_for_grouping(
    grouping: GuildGroupingClass,
    entries: &[&'static GuildCatalogEntry],
) -> Vec<usize> {
    let mut out: Vec<usize> = entries
        .iter()
        .enumerate()
        .filter_map(|(index, entry)| (entry.grouping == grouping).then_some(index))
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
        let entries = catalog::playable_entries_list();
        let thematic = std::array::from_fn(|index| {
            let (_, label) = THEMES_UX_ORDER[index];
            ThematicBucket {
                label,
                playable_def_indices: playable_indices_for_grouping(
                    GuildGroupingClass::Thematic(index),
                    &entries,
                ),
            }
        });
        GuildGrouping {
            thematic,
            multi_playable_indices: playable_indices_for_grouping(
                GuildGroupingClass::Multi,
                &entries,
            ),
        }
    })
}

pub fn visible_indices_multi_drill() -> Vec<usize> {
    guild_grouping().multi_playable_indices.clone()
}

/// Clear selected flags for thematic guilds outside `active_thematic`, keep multi and in-bucket thematic.
pub fn clear_selected_outside_thematic_bucket(
    entries: &[&'static GuildCatalogEntry],
    selected: &mut [bool],
    active_thematic: usize,
) {
    for (index, entry) in entries.iter().enumerate() {
        let Some(class) = classify_guild_key_typed(entry.key) else {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disciple_classifies_as_multi_background() {
        assert!(matches!(
            classify_guild_key_typed(catalog::GuildKey::Disciple),
            Some(GuildBucketClass::Multi)
        ));
    }

    #[test]
    fn multi_playable_contains_disciple_when_implemented() {
        let entries = catalog::playable_entries_list();
        assert!(
            guild_grouping()
                .multi_playable_indices
                .iter()
                .any(|&ix| entries[ix].persisted_key == "disciple")
        );
    }

    #[test]
    fn multi_playable_contains_kharim_when_implemented() {
        let entries = catalog::playable_entries_list();
        assert!(
            guild_grouping()
                .multi_playable_indices
                .iter()
                .any(|&ix| entries[ix].persisted_key == "kharim")
        );
    }
}
