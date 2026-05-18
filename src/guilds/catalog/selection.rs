use crate::guilds::Guild;

use super::{
    GuildCatalogEntry, GuildGroupingClass, GuildKey, entry_for_key,
    playable_entry_for_persisted_key,
};

/// Fixed browse order for thematic rows (exclusive primary among these five).
pub const THEMES_UX_ORDER: &[(&str, &str)] = &[
    ("civilized", "Civilized"),
    ("magical", "Magical"),
    ("good_religious", "Good Religious"),
    ("evil_religious", "Evil Religious"),
    ("nomad", "Nomad"),
];

pub const DEFAULT_GUILD_PRIMARY_KEYWORD: &str = "civilized";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildBucketClass {
    Thematic(usize),
    Multi,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GuildSelection {
    keys: Vec<GuildKey>,
    primary_background_index: usize,
}

impl Default for GuildSelection {
    fn default() -> Self {
        Self {
            keys: Vec::new(),
            primary_background_index: default_primary_background_index(),
        }
    }
}

impl GuildSelection {
    pub fn from_persisted_keys(keys: &[String], primary_background_keyword: Option<&str>) -> Self {
        let playable_keys = keys
            .iter()
            .filter_map(|key| playable_entry_for_persisted_key(key).map(|entry| entry.key));
        Self::from_playable_keys(playable_keys, primary_background_keyword)
    }

    pub fn from_playable_keys(
        keys: impl IntoIterator<Item = GuildKey>,
        primary_background_keyword: Option<&str>,
    ) -> Self {
        let raw_keys = keys
            .into_iter()
            .filter_map(|key| {
                entry_for_key(key)
                    .filter(|entry| entry.is_playable())
                    .map(|entry| entry.key)
            })
            .collect::<Vec<_>>();
        let primary_background_index =
            choose_primary_background_index(&raw_keys, primary_background_keyword);
        let keys = normalize_selected_keys(raw_keys, primary_background_index);

        Self {
            keys,
            primary_background_index,
        }
    }

    pub fn persisted_keys(&self) -> Vec<String> {
        self.keys
            .iter()
            .filter_map(|key| entry_for_key(*key))
            .map(|entry| entry.persisted_key.to_string())
            .collect()
    }

    pub fn persisted_keys_option(&self) -> Option<Vec<String>> {
        let keys = self.persisted_keys();
        (!keys.is_empty()).then_some(keys)
    }

    pub fn primary_background_keyword(&self) -> &'static str {
        THEMES_UX_ORDER[self.primary_background_index].0
    }

    pub fn is_selected(&self, key: GuildKey) -> bool {
        self.keys.contains(&key)
    }

    pub fn build_guilds(&self) -> Vec<Box<dyn Guild>> {
        self.keys
            .iter()
            .filter_map(|key| entry_for_key(*key).and_then(GuildCatalogEntry::build))
            .collect()
    }
}

pub fn thematic_index_for_keyword(keyword: &str) -> Option<usize> {
    THEMES_UX_ORDER
        .iter()
        .position(|(canonical, _)| *canonical == keyword)
}

pub fn classify_guild_key_typed(guild_key: GuildKey) -> Option<GuildBucketClass> {
    match entry_for_key(guild_key)?.grouping {
        GuildGroupingClass::Multi => Some(GuildBucketClass::Multi),
        GuildGroupingClass::Thematic(index) => Some(GuildBucketClass::Thematic(index)),
    }
}

fn choose_primary_background_index(
    keys: &[GuildKey],
    primary_background_keyword: Option<&str>,
) -> usize {
    let occupied_buckets = occupied_thematic_buckets(keys);
    let stored_primary_index = primary_background_keyword.and_then(thematic_index_for_keyword);

    match occupied_buckets.len() {
        0 => stored_primary_index.unwrap_or_else(default_primary_background_index),
        1 => occupied_buckets[0],
        _ => stored_primary_index
            .filter(|primary_index| occupied_buckets.contains(primary_index))
            .unwrap_or_else(|| {
                *occupied_buckets
                    .iter()
                    .min()
                    .expect("occupied_buckets is non-empty")
            }),
    }
}

fn normalize_selected_keys(
    keys: impl IntoIterator<Item = GuildKey>,
    primary_background_index: usize,
) -> Vec<GuildKey> {
    let mut selected = Vec::new();
    for key in keys {
        if let Some(entry) = entry_for_key(key).filter(|entry| entry.is_playable()) {
            match entry.grouping {
                GuildGroupingClass::Multi => selected.push(entry.key),
                GuildGroupingClass::Thematic(index) if index == primary_background_index => {
                    selected.push(entry.key);
                }
                GuildGroupingClass::Thematic(_) => {}
            }
        }
    }
    selected.sort_by_key(|key| entry_for_key(*key).map(|entry| entry.persisted_key));
    selected.dedup();
    selected
}

fn occupied_thematic_buckets(keys: &[GuildKey]) -> Vec<usize> {
    let mut occupied = Vec::new();
    for key in keys {
        if let Some(GuildBucketClass::Thematic(index)) = classify_guild_key_typed(*key)
            && !occupied.contains(&index)
        {
            occupied.push(index);
        }
    }
    occupied.sort_unstable();
    occupied
}

fn default_primary_background_index() -> usize {
    thematic_index_for_keyword(DEFAULT_GUILD_PRIMARY_KEYWORD).expect("civilized theme exists")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys(values: &[&str]) -> Vec<String> {
        values.iter().map(|value| (*value).to_string()).collect()
    }

    #[test]
    fn persisted_keys_filter_unimplemented_unknown_and_duplicates() {
        let selection = GuildSelection::from_persisted_keys(
            &keys(&["animist", "alchemists", "missing", "animist", "kharim"]),
            Some("good_religious"),
        );

        assert_eq!(
            selection.persisted_keys(),
            vec!["animist".to_string(), "kharim".to_string()]
        );
        assert!(selection.is_selected(GuildKey::Animist));
        assert!(selection.is_selected(GuildKey::Kharim));
    }

    #[test]
    fn thematic_conflict_uses_stored_primary_when_selected() {
        let selection = GuildSelection::from_persisted_keys(
            &keys(&["animist", "riftwalker", "disciple"]),
            Some("magical"),
        );

        assert_eq!(
            selection.persisted_keys(),
            vec!["disciple".to_string(), "riftwalker".to_string()]
        );
        assert_eq!(selection.primary_background_keyword(), "magical");
    }

    #[test]
    fn thematic_conflict_falls_back_to_lowest_occupied_bucket() {
        let selection = GuildSelection::from_persisted_keys(
            &keys(&["animist", "riftwalker", "disciple"]),
            Some("civilized"),
        );

        assert_eq!(
            selection.persisted_keys(),
            vec!["disciple".to_string(), "riftwalker".to_string()]
        );
        assert_eq!(selection.primary_background_keyword(), "magical");
    }

    #[test]
    fn empty_selection_keeps_valid_primary_or_defaults() {
        let selection = GuildSelection::from_persisted_keys(&[], Some("nomad"));
        assert_eq!(selection.persisted_keys_option(), None);
        assert_eq!(selection.primary_background_keyword(), "nomad");

        let selection = GuildSelection::from_persisted_keys(&[], Some("missing"));
        assert_eq!(
            selection.primary_background_keyword(),
            DEFAULT_GUILD_PRIMARY_KEYWORD
        );
    }

    #[test]
    fn build_guilds_uses_normalized_selection() {
        let selection = GuildSelection::from_persisted_keys(
            &keys(&["animist", "alchemists", "missing", "animist", "kharim"]),
            Some("good_religious"),
        );

        assert_eq!(selection.build_guilds().len(), 2);
    }
}
