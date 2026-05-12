//! Guild **keyword** lists for thematic grouping in the UX.
//! batrs loads these from Rust only; keep the slices aligned when guild keywords change.

/// Row order aligns with [`super::THEMES_UX_ORDER`]: civilized, magical, good_religious, evil_religious, nomad.
pub(crate) static THEMATIC_GUILD_KEYWORDS: [&[&str]; 5] = [
    &[
        "alchemists",
        "civilized_fighters",
        "civmage",
        "folklorist",
        "knight",
        "merchant",
        "runemages",
        "sabres",
        "bard",
    ],
    &[
        "channellers",
        "inner_circle",
        "mage",
        "mage_acid",
        "mage_asphyxiation",
        "mage_cold",
        "mage_electricity",
        "mage_fire",
        "mage_magical",
        "mage_poison",
        "psionicist",
        "riftwalker",
    ],
    &[
        "animist",
        "druids",
        "liberator",
        "monk",
        "nun",
        "tarmalen",
        "templar",
    ],
    &[
        "aelena", "curate", "nergal", "reaver", "seminary", "spider", "tiger", "triad", "tzarakk",
    ],
    &["archers", "barbarian", "beastmaster", "ranger", "crimson"],
];

/// CSV `background_multi` guild keywords (`type == guild` after `background_multi` header).
pub(crate) static MULTI_BACKGROUND_GUILD_KEYWORDS: &[&str] = &[
    "cavalier",
    "disciple",
    "explorer",
    "inf",
    "kharim",
    "navigator",
    "sailor",
    "squire",
    "treenav",
];
