mod generated {
    #![allow(dead_code)]
    include!(concat!(env!("OUT_DIR"), "/combat_damage_catalog.rs"));
}

use std::collections::HashMap;
use std::sync::LazyLock;

pub use generated::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeleeCatalogMeta {
    pub family_index: usize,
    pub rank: u8,
}

static UNAMBIGUOUS_MELEE_VERBS: LazyLock<HashMap<String, MeleeCatalogMeta>> = LazyLock::new(|| {
    let mut by_verb: HashMap<String, Vec<&CatalogEntry>> = HashMap::new();
    for entry in CATALOG {
        by_verb
            .entry(entry.canonical.to_ascii_lowercase())
            .or_default()
            .push(entry);
    }
    by_verb
        .into_iter()
        .filter_map(|(verb, entries)| {
            if entries.len() != 1 {
                return None;
            }
            let entry = entries[0];
            Some((
                verb,
                MeleeCatalogMeta {
                    family_index: entry.family,
                    rank: entry.rank,
                },
            ))
        })
        .collect()
});

/// Resolves catalog rank and family for a melee verb when it maps to exactly one catalog entry.
pub fn unambiguous_melee_catalog_meta(verb: &str) -> Option<MeleeCatalogMeta> {
    UNAMBIGUOUS_MELEE_VERBS
        .get(&verb.to_ascii_lowercase())
        .copied()
}
