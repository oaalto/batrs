//! Guild membership groups derived from the typed Guild Catalog.
//! Thematic buckets are mutually exclusive for saved preferences; multi-background guilds filter per theme.

use std::sync::OnceLock;

use super::catalog::{self, GuildCatalogEntry, GuildGroupingClass, GuildKey};

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

/// Whether a multi-background guild appears in `/guilds` drill for the given thematic index.
pub fn multi_guild_eligible_for_thematic(guild_key: GuildKey, thematic_ix: usize) -> bool {
    use GuildKey::*;
    match guild_key {
        Cavalier | Squire => matches!(thematic_ix, 0 | 4),
        Disciple => matches!(thematic_ix, 0 | 3 | 4),
        Kharim => matches!(thematic_ix, 3 | 4),
        Navigator => matches!(thematic_ix, 0 | 2 | 3 | 4),
        Explorer | Inf | Sailor | Treenav => true,
        _ => false,
    }
}

pub fn visible_indices_multi_drill_for(thematic_ix: usize) -> Vec<usize> {
    let entries = catalog::playable_entries_list();
    guild_grouping()
        .multi_playable_indices
        .iter()
        .copied()
        .filter(|&ix| multi_guild_eligible_for_thematic(entries[ix].key, thematic_ix))
        .collect()
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
    use super::THEMES_UX_ORDER;
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

    fn multi_keys_for_thematic(thematic_ix: usize) -> Vec<&'static str> {
        let entries = catalog::playable_entries_list();
        visible_indices_multi_drill_for(thematic_ix)
            .into_iter()
            .map(|ix| entries[ix].persisted_key)
            .collect()
    }

    #[test]
    fn magical_multi_drill_shows_universal_guilds_only() {
        let magical_ix = thematic_index_for_keyword("magical").expect("magical");
        assert_eq!(
            multi_keys_for_thematic(magical_ix),
            vec!["explorer", "inf", "sailor", "treenav"]
        );
    }

    #[test]
    fn civilized_multi_drill_includes_cavalier_not_kharim() {
        let civilized_ix = thematic_index_for_keyword("civilized").expect("civilized");
        let keys = multi_keys_for_thematic(civilized_ix);
        assert!(keys.contains(&"cavalier"));
        assert!(keys.contains(&"navigator"));
        assert!(!keys.contains(&"kharim"));
    }

    #[test]
    fn nomad_multi_drill_includes_kharim_and_cavalier() {
        let nomad_ix = thematic_index_for_keyword("nomad").expect("nomad");
        let keys = multi_keys_for_thematic(nomad_ix);
        assert!(keys.contains(&"kharim"));
        assert!(keys.contains(&"cavalier"));
    }

    #[test]
    fn background_only_entries_excluded_from_playable_drill_indices() {
        let entries = catalog::playable_entries_list();
        for &(keyword, _) in THEMES_UX_ORDER {
            let bucket_ix = thematic_index_for_keyword(keyword).expect("theme");
            let indices = &guild_grouping().thematic[bucket_ix].playable_def_indices;
            for &ix in indices {
                assert_ne!(
                    entries[ix].persisted_key, keyword,
                    "background keyword {keyword} must not appear in drill toggles"
                );
            }
        }
    }
}
