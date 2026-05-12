//! Guild membership groups from [`tf/guild_urls.csv`] (embedded at compile time).
//! Thematic buckets are mutually exclusive for saved preferences; CSV `background_multi`
//! overlaps every thematic drill.

use std::collections::HashMap;
use std::sync::OnceLock;

use super::{GuildDefinition, guild_definitions};
use crate::config::PlayerToml;

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

fn keyword_is_implemented(csv_keyword: &str, defs: &[GuildDefinition]) -> bool {
    defs.iter().any(|definition| definition.key == csv_keyword)
}

fn playable_indices_for_csv_keywords(
    csv_keywords: &[String],
    defs: &[GuildDefinition],
) -> Vec<usize> {
    let mut out: Vec<usize> = csv_keywords
        .iter()
        .filter_map(|keyword| {
            if !keyword_is_implemented(keyword, defs) {
                return None;
            }
            defs.iter()
                .position(|definition| definition.key == keyword.as_str())
        })
        .collect();
    out.sort_unstable();
    out.dedup();
    out
}

fn parse_guild_urls_embedded() -> ParsedCsv {
    static CSV: &str = include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/tf/guild_urls.csv"));

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    enum Header {
        Thematic(&'static str),
        Multi,
    }

    let mut thematic_lists: HashMap<&'static str, Vec<String>> = HashMap::new();
    for (keyword, _) in THEMES_UX_ORDER {
        thematic_lists.insert(keyword, Vec::new());
    }

    let mut multi_list = Vec::<String>::new();
    let mut current: Option<Header> = None;

    for raw_line in CSV.lines().skip(1) {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }
        let Some((kind, guild_keyword_chunk, _display, _url)) = split_csv_line(line) else {
            continue;
        };

        match kind {
            "background" => {
                let bucket = thematic_index_for_keyword(guild_keyword_chunk);
                current = bucket.map(|ix| Header::Thematic(THEMES_UX_ORDER[ix].0));
                if bucket.is_none() {
                    log::warn!("guild_urls.csv: unknown background keyword {guild_keyword_chunk}");
                }
                continue;
            }
            "background_multi" => {
                current = Some(Header::Multi);
                continue;
            }
            "guild" => {
                let Some(hdr) = current else {
                    log::warn!(
                        "guild_urls.csv: guild row before header: {}",
                        guild_keyword_chunk
                    );
                    continue;
                };
                match hdr {
                    Header::Thematic(theme_key) => {
                        thematic_lists
                            .entry(theme_key)
                            .or_default()
                            .push(guild_keyword_chunk.to_string());
                    }
                    Header::Multi => multi_list.push(guild_keyword_chunk.to_string()),
                }
            }
            _ => {}
        }
    }

    ParsedCsv {
        thematic_lists,
        multi_list,
    }
}

fn split_csv_line(line: &str) -> Option<(&str, &str, &str, &str)> {
    let mut parts = line.splitn(4, ',');
    Some((
        parts.next()?.trim(),
        parts.next()?.trim(),
        parts.next()?.trim(),
        parts.next().map(|s| s.trim()).unwrap_or(""),
    ))
}

struct ParsedCsv {
    thematic_lists: HashMap<&'static str, Vec<String>>,
    multi_list: Vec<String>,
}

/// Static grouping data derived from the CSV and current [`guild_definitions`].
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
        let parsed = parse_guild_urls_embedded();
        let defs = guild_definitions();
        let thematic = std::array::from_fn(|index| {
            let (keyword, label) = THEMES_UX_ORDER[index];
            let keywords = parsed
                .thematic_lists
                .get(keyword)
                .cloned()
                .unwrap_or_default();
            ThematicBucket {
                label,
                playable_def_indices: playable_indices_for_csv_keywords(&keywords, &defs),
            }
        });
        GuildGrouping {
            thematic,
            multi_playable_indices: playable_indices_for_csv_keywords(&parsed.multi_list, &defs),
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
    let grouping = guild_grouping();
    if grouping
        .multi_playable_indices
        .iter()
        .any(|&ix| guild_definitions()[ix].key == def_key)
    {
        return Some(GuildBucketClass::Multi);
    }
    for (index, bucket) in grouping.thematic.iter().enumerate() {
        if bucket
            .playable_def_indices
            .iter()
            .any(|&ix| guild_definitions()[ix].key == def_key)
        {
            return Some(GuildBucketClass::Thematic(index));
        }
    }
    None
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
        let Some(class) = classify_guild_key(definition.key) else {
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

/// Normalize guild list and `guild_primary_background` for one player TOML. Returns whether data changed.
pub fn normalize_player_guild_toml(player: &mut PlayerToml) -> bool {
    let defs = guild_definitions();
    let mut changed = false;

    let guilds_owned = player
        .guilds
        .clone()
        .unwrap_or_default()
        .into_iter()
        .filter(|key| defs.iter().any(|definition| definition.key == key))
        .collect::<Vec<_>>();

    let mut thematic_by_bucket: [Vec<&str>; 5] = std::array::from_fn(|_| Vec::new());
    let mut multi_keys = Vec::<&str>::new();

    for key in guilds_owned.iter().map(|s| s.as_str()) {
        match classify_guild_key(key) {
            Some(GuildBucketClass::Multi) => multi_keys.push(key),
            Some(GuildBucketClass::Thematic(ix)) => thematic_by_bucket[ix].push(key),
            None => {}
        }
    }

    let occupied_buckets: Vec<usize> = thematic_by_bucket
        .iter()
        .enumerate()
        .filter_map(|(ix, bucket)| (!bucket.is_empty()).then_some(ix))
        .collect();

    let stored_primary_ix = player
        .guild_primary_background
        .as_deref()
        .and_then(thematic_index_for_keyword);

    let chosen_primary_ix: usize = match occupied_buckets.len() {
        0 => stored_primary_ix.unwrap_or_else(|| {
            thematic_index_for_keyword(DEFAULT_GUILD_PRIMARY_KEYWORD)
                .expect("civilized in THEMES_UX_ORDER")
        }),
        1 => occupied_buckets[0],
        _ => {
            if let Some(pix) = stored_primary_ix
                && occupied_buckets.contains(&pix)
                && !thematic_by_bucket[pix].is_empty()
            {
                pix
            } else {
                *occupied_buckets
                    .iter()
                    .min_by_key(|ix| **ix)
                    .expect("non-empty occupied_buckets")
            }
        }
    };

    let new_primary_keyword = THEMES_UX_ORDER[chosen_primary_ix].0.to_string();
    if player.guild_primary_background.as_deref() != Some(new_primary_keyword.as_str()) {
        player.guild_primary_background = Some(new_primary_keyword.clone());
        changed = true;
    }

    let mut thematic_keys_kept = Vec::<String>::new();

    if occupied_buckets.len() <= 1 {
        if let Some(ix) = occupied_buckets.first() {
            thematic_keys_kept.extend(
                thematic_by_bucket[*ix]
                    .iter()
                    .copied()
                    .map(ToString::to_string),
            );
        }
    } else {
        thematic_keys_kept.extend(
            thematic_by_bucket[chosen_primary_ix]
                .iter()
                .copied()
                .map(ToString::to_string),
        );
    }

    let mut union = thematic_keys_kept;
    union.extend(multi_keys.into_iter().map(|s| s.to_string()));
    union.sort_unstable();
    union.dedup();

    if player.guilds.as_ref() != Some(&union) {
        changed = true;
    }
    player.guilds = if union.is_empty() { None } else { Some(union) };

    changed
}

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
}
