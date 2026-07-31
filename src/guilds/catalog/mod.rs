use super::{
    AelenaGuild, AlchemistsGuild, AnimistGuild, ArchersGuild, BarbarianGuild, BardGuild,
    BeastmasterGuild, CavalierGuild, ChannellersGuild, CivilizedFightersGuild, CivilizedGuild,
    CivmageGuild, CrimsonGuild, CurateGuild, DiscipleGuild, DruidsGuild, EvilReligiousGuild,
    ExplorerGuild, FolkloristGuild, GoodReligiousGuild, Guild, InfGuild, InnerCircleGuild,
    KharimGuild, KnightGuild, LiberatorGuild, MageAcidGuild, MageAsphyxiationGuild, MageColdGuild,
    MageElectricityGuild, MageFireGuild, MageGuild, MageMagicalGuild, MagePoisonGuild,
    MagicalGuild, MerchantGuild, MonkGuild, NavigatorGuild, NergalGuild, NomadGuild, NunGuild,
    PsionicistGuild, RangerGuild, ReaverGuild, RiftwalkerGuild, RunemagesGuild, SabresGuild,
    SailorGuild, SeminaryGuild, SpiderGuild, SquireGuild, TarmalenGuild, TemplarGuild, TigerGuild,
    TreenavGuild, TriadGuild, TzarakkGuild,
};

mod browse;
mod selection;

pub use browse::{GuildBrowseRow, GuildDrillSource, browse_labels, drill_rows};
pub use selection::{
    DEFAULT_GUILD_PRIMARY_KEYWORD, GuildBucketClass, GuildSelection, THEMES_UX_ORDER,
    classify_guild_key_typed, thematic_index_for_keyword,
};

pub type GuildFactory = fn() -> Box<dyn Guild>;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GuildKey {
    Aelena,
    Alchemists,
    Animist,
    Archers,
    Barbarian,
    Bard,
    Beastmaster,
    Cavalier,
    Channellers,
    Civmage,
    Civilized,
    CivilizedFighters,
    Crimson,
    Curate,
    Disciple,
    Druids,
    EvilReligious,
    Explorer,
    Folklorist,
    GoodReligious,
    Inf,
    InnerCircle,
    Kharim,
    Knight,
    Liberator,
    Magical,
    Mage,
    MageAcid,
    MageAsphyxiation,
    MageCold,
    MageElectricity,
    MageFire,
    MageMagical,
    MagePoison,
    Merchant,
    Monk,
    Nomad,
    Navigator,
    Nergal,
    Nun,
    Psionicist,
    Ranger,
    Reaver,
    Riftwalker,
    Runemages,
    Sabres,
    Sailor,
    Seminary,
    Spider,
    Squire,
    Tarmalen,
    Templar,
    Tiger,
    Treenav,
    Triad,
    Tzarakk,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GuildGroupingClass {
    Thematic(usize),
    Multi,
}

#[derive(Clone, Copy)]
pub enum GuildPlayability {
    Playable {
        build: GuildFactory,
    },
    /// Buildable for background auto-injection; excluded from drill toggles and selection.
    BackgroundOnly {
        build: GuildFactory,
    },
}

pub struct GuildCatalogEntry {
    pub key: GuildKey,
    pub persisted_key: &'static str,
    pub display_name: &'static str,
    pub grouping: GuildGroupingClass,
    pub playability: GuildPlayability,
}

impl GuildCatalogEntry {
    pub fn is_playable(&self) -> bool {
        matches!(self.playability, GuildPlayability::Playable { .. })
    }

    pub fn build(&self) -> Option<Box<dyn Guild>> {
        match self.playability {
            GuildPlayability::Playable { build } | GuildPlayability::BackgroundOnly { build } => {
                Some(build())
            }
        }
    }
}

const fn playable(
    key: GuildKey,
    persisted_key: &'static str,
    display_name: &'static str,
    grouping: GuildGroupingClass,
    build: GuildFactory,
) -> GuildCatalogEntry {
    GuildCatalogEntry {
        key,
        persisted_key,
        display_name,
        grouping,
        playability: GuildPlayability::Playable { build },
    }
}

const fn background_only(
    key: GuildKey,
    persisted_key: &'static str,
    display_name: &'static str,
    grouping: GuildGroupingClass,
    build: GuildFactory,
) -> GuildCatalogEntry {
    GuildCatalogEntry {
        key,
        persisted_key,
        display_name,
        grouping,
        playability: GuildPlayability::BackgroundOnly { build },
    }
}

macro_rules! guild_factory {
    ($factory_name:ident, $guild_type:ty) => {
        fn $factory_name() -> Box<dyn Guild> {
            Box::new(<$guild_type>::default())
        }
    };
}

guild_factory!(build_aelena, AelenaGuild);
guild_factory!(build_alchemists, AlchemistsGuild);
guild_factory!(build_animist, AnimistGuild);
guild_factory!(build_archers, ArchersGuild);
guild_factory!(build_barbarian, BarbarianGuild);
guild_factory!(build_bard, BardGuild);
guild_factory!(build_beastmaster, BeastmasterGuild);
guild_factory!(build_cavalier, CavalierGuild);
guild_factory!(build_channellers, ChannellersGuild);
guild_factory!(build_civilized, CivilizedGuild);
guild_factory!(build_civilized_fighters, CivilizedFightersGuild);
guild_factory!(build_civmage, CivmageGuild);
guild_factory!(build_crimson, CrimsonGuild);
guild_factory!(build_curate, CurateGuild);
guild_factory!(build_disciple, DiscipleGuild);
guild_factory!(build_druids, DruidsGuild);
guild_factory!(build_evil_religious, EvilReligiousGuild);
guild_factory!(build_explorer, ExplorerGuild);
guild_factory!(build_folklorist, FolkloristGuild);
guild_factory!(build_good_religious, GoodReligiousGuild);
guild_factory!(build_inf, InfGuild);
guild_factory!(build_inner_circle, InnerCircleGuild);
guild_factory!(build_kharim, KharimGuild);
guild_factory!(build_knight, KnightGuild);
guild_factory!(build_liberator, LiberatorGuild);
guild_factory!(build_magical, MagicalGuild);
guild_factory!(build_mage, MageGuild);
guild_factory!(build_mage_acid, MageAcidGuild);
guild_factory!(build_mage_asphyxiation, MageAsphyxiationGuild);
guild_factory!(build_mage_cold, MageColdGuild);
guild_factory!(build_mage_electricity, MageElectricityGuild);
guild_factory!(build_mage_fire, MageFireGuild);
guild_factory!(build_mage_magical, MageMagicalGuild);
guild_factory!(build_mage_poison, MagePoisonGuild);
guild_factory!(build_merchant, MerchantGuild);
guild_factory!(build_monk, MonkGuild);
guild_factory!(build_navigator, NavigatorGuild);
guild_factory!(build_nergal, NergalGuild);
guild_factory!(build_nomad, NomadGuild);
guild_factory!(build_nun, NunGuild);
guild_factory!(build_psionicist, PsionicistGuild);
guild_factory!(build_ranger, RangerGuild);
guild_factory!(build_reaver, ReaverGuild);
guild_factory!(build_riftwalker, RiftwalkerGuild);
guild_factory!(build_runemages, RunemagesGuild);
guild_factory!(build_sabres, SabresGuild);
guild_factory!(build_sailor, SailorGuild);
guild_factory!(build_seminary, SeminaryGuild);
guild_factory!(build_spider, SpiderGuild);
guild_factory!(build_squire, SquireGuild);
guild_factory!(build_tarmalen, TarmalenGuild);
guild_factory!(build_templar, TemplarGuild);
guild_factory!(build_tiger, TigerGuild);
guild_factory!(build_treenav, TreenavGuild);
guild_factory!(build_triad, TriadGuild);
guild_factory!(build_tzarakk, TzarakkGuild);

pub static GUILD_CATALOG: &[GuildCatalogEntry] = &[
    background_only(
        GuildKey::Civilized,
        "civilized",
        "Civilized",
        GuildGroupingClass::Thematic(0),
        build_civilized,
    ),
    playable(
        GuildKey::Alchemists,
        "alchemists",
        "Alchemists",
        GuildGroupingClass::Thematic(0),
        build_alchemists,
    ),
    playable(
        GuildKey::CivilizedFighters,
        "civilized_fighters",
        "Civilized Fighters",
        GuildGroupingClass::Thematic(0),
        build_civilized_fighters,
    ),
    playable(
        GuildKey::Civmage,
        "civmage",
        "Civmage",
        GuildGroupingClass::Thematic(0),
        build_civmage,
    ),
    playable(
        GuildKey::Folklorist,
        "folklorist",
        "Folklorist",
        GuildGroupingClass::Thematic(0),
        build_folklorist,
    ),
    playable(
        GuildKey::Knight,
        "knight",
        "Knight",
        GuildGroupingClass::Thematic(0),
        build_knight,
    ),
    playable(
        GuildKey::Merchant,
        "merchant",
        "Merchant",
        GuildGroupingClass::Thematic(0),
        build_merchant,
    ),
    playable(
        GuildKey::Runemages,
        "runemages",
        "Runemages",
        GuildGroupingClass::Thematic(0),
        build_runemages,
    ),
    playable(
        GuildKey::Sabres,
        "sabres",
        "Sabres",
        GuildGroupingClass::Thematic(0),
        build_sabres,
    ),
    playable(
        GuildKey::Bard,
        "bard",
        "Bard",
        GuildGroupingClass::Thematic(0),
        build_bard,
    ),
    background_only(
        GuildKey::Magical,
        "magical",
        "Magical",
        GuildGroupingClass::Thematic(1),
        build_magical,
    ),
    playable(
        GuildKey::Channellers,
        "channellers",
        "Channeller",
        GuildGroupingClass::Thematic(1),
        build_channellers,
    ),
    playable(
        GuildKey::InnerCircle,
        "inner_circle",
        "Inner Circle",
        GuildGroupingClass::Thematic(1),
        build_inner_circle,
    ),
    playable(
        GuildKey::Mage,
        "mage",
        "Mage",
        GuildGroupingClass::Thematic(1),
        build_mage,
    ),
    playable(
        GuildKey::MageAcid,
        "mage_acid",
        "Mage Acid",
        GuildGroupingClass::Thematic(1),
        build_mage_acid,
    ),
    playable(
        GuildKey::MageAsphyxiation,
        "mage_asphyxiation",
        "Mage Asphyxiation",
        GuildGroupingClass::Thematic(1),
        build_mage_asphyxiation,
    ),
    playable(
        GuildKey::MageCold,
        "mage_cold",
        "Mage Cold",
        GuildGroupingClass::Thematic(1),
        build_mage_cold,
    ),
    playable(
        GuildKey::MageElectricity,
        "mage_electricity",
        "Mage Electricity",
        GuildGroupingClass::Thematic(1),
        build_mage_electricity,
    ),
    playable(
        GuildKey::MageFire,
        "mage_fire",
        "Mage Fire",
        GuildGroupingClass::Thematic(1),
        build_mage_fire,
    ),
    playable(
        GuildKey::MageMagical,
        "mage_magical",
        "Mage Magical",
        GuildGroupingClass::Thematic(1),
        build_mage_magical,
    ),
    playable(
        GuildKey::MagePoison,
        "mage_poison",
        "Mage Poison",
        GuildGroupingClass::Thematic(1),
        build_mage_poison,
    ),
    playable(
        GuildKey::Psionicist,
        "psionicist",
        "Psionicist",
        GuildGroupingClass::Thematic(1),
        build_psionicist,
    ),
    playable(
        GuildKey::Riftwalker,
        "riftwalker",
        "Riftwalker",
        GuildGroupingClass::Thematic(1),
        build_riftwalker,
    ),
    background_only(
        GuildKey::GoodReligious,
        "good_religious",
        "Good Religious",
        GuildGroupingClass::Thematic(2),
        build_good_religious,
    ),
    playable(
        GuildKey::Animist,
        "animist",
        "Animist",
        GuildGroupingClass::Thematic(2),
        build_animist,
    ),
    playable(
        GuildKey::Druids,
        "druids",
        "Druids",
        GuildGroupingClass::Thematic(2),
        build_druids,
    ),
    playable(
        GuildKey::Liberator,
        "liberator",
        "Liberator",
        GuildGroupingClass::Thematic(2),
        build_liberator,
    ),
    playable(
        GuildKey::Monk,
        "monk",
        "Monk",
        GuildGroupingClass::Thematic(2),
        build_monk,
    ),
    playable(
        GuildKey::Nun,
        "nun",
        "Nun",
        GuildGroupingClass::Thematic(2),
        build_nun,
    ),
    playable(
        GuildKey::Tarmalen,
        "tarmalen",
        "Tarmalen",
        GuildGroupingClass::Thematic(2),
        build_tarmalen,
    ),
    playable(
        GuildKey::Templar,
        "templar",
        "Templar",
        GuildGroupingClass::Thematic(2),
        build_templar,
    ),
    background_only(
        GuildKey::EvilReligious,
        "evil_religious",
        "Evil Religious",
        GuildGroupingClass::Thematic(3),
        build_evil_religious,
    ),
    playable(
        GuildKey::Aelena,
        "aelena",
        "Aelena",
        GuildGroupingClass::Thematic(3),
        build_aelena,
    ),
    playable(
        GuildKey::Curate,
        "curate",
        "Curate",
        GuildGroupingClass::Thematic(3),
        build_curate,
    ),
    playable(
        GuildKey::Nergal,
        "nergal",
        "Nergal",
        GuildGroupingClass::Thematic(3),
        build_nergal,
    ),
    playable(
        GuildKey::Reaver,
        "reaver",
        "Reaver",
        GuildGroupingClass::Thematic(3),
        build_reaver,
    ),
    playable(
        GuildKey::Seminary,
        "seminary",
        "Seminary",
        GuildGroupingClass::Thematic(3),
        build_seminary,
    ),
    playable(
        GuildKey::Spider,
        "spider",
        "Spider",
        GuildGroupingClass::Thematic(3),
        build_spider,
    ),
    playable(
        GuildKey::Tiger,
        "tiger",
        "Tiger",
        GuildGroupingClass::Thematic(3),
        build_tiger,
    ),
    playable(
        GuildKey::Triad,
        "triad",
        "Triad",
        GuildGroupingClass::Thematic(3),
        build_triad,
    ),
    playable(
        GuildKey::Tzarakk,
        "tzarakk",
        "Tzarakk",
        GuildGroupingClass::Thematic(3),
        build_tzarakk,
    ),
    background_only(
        GuildKey::Nomad,
        "nomad",
        "Nomad",
        GuildGroupingClass::Thematic(4),
        build_nomad,
    ),
    playable(
        GuildKey::Archers,
        "archers",
        "Archers",
        GuildGroupingClass::Thematic(4),
        build_archers,
    ),
    playable(
        GuildKey::Barbarian,
        "barbarian",
        "Barbarian",
        GuildGroupingClass::Thematic(4),
        build_barbarian,
    ),
    playable(
        GuildKey::Beastmaster,
        "beastmaster",
        "Beastmaster",
        GuildGroupingClass::Thematic(4),
        build_beastmaster,
    ),
    playable(
        GuildKey::Ranger,
        "ranger",
        "Ranger",
        GuildGroupingClass::Thematic(4),
        build_ranger,
    ),
    playable(
        GuildKey::Crimson,
        "crimson",
        "Crimson",
        GuildGroupingClass::Thematic(4),
        build_crimson,
    ),
    playable(
        GuildKey::Cavalier,
        "cavalier",
        "Cavalier",
        GuildGroupingClass::Multi,
        build_cavalier,
    ),
    playable(
        GuildKey::Disciple,
        "disciple",
        "Disciple",
        GuildGroupingClass::Multi,
        build_disciple,
    ),
    playable(
        GuildKey::Explorer,
        "explorer",
        "Explorer",
        GuildGroupingClass::Multi,
        build_explorer,
    ),
    playable(
        GuildKey::Inf,
        "inf",
        "Inf",
        GuildGroupingClass::Multi,
        build_inf,
    ),
    playable(
        GuildKey::Kharim,
        "kharim",
        "Kharim",
        GuildGroupingClass::Multi,
        build_kharim,
    ),
    playable(
        GuildKey::Navigator,
        "navigator",
        "Navigator",
        GuildGroupingClass::Multi,
        build_navigator,
    ),
    playable(
        GuildKey::Sailor,
        "sailor",
        "Sailor",
        GuildGroupingClass::Multi,
        build_sailor,
    ),
    playable(
        GuildKey::Squire,
        "squire",
        "Squire",
        GuildGroupingClass::Multi,
        build_squire,
    ),
    playable(
        GuildKey::Treenav,
        "treenav",
        "Treenav",
        GuildGroupingClass::Multi,
        build_treenav,
    ),
];

pub fn entries() -> &'static [GuildCatalogEntry] {
    GUILD_CATALOG
}

pub fn playable_entries() -> impl Iterator<Item = &'static GuildCatalogEntry> {
    entries().iter().filter(|entry| entry.is_playable())
}

pub fn playable_entries_list() -> Vec<&'static GuildCatalogEntry> {
    playable_entries().collect()
}

pub fn entry_for_key(key: GuildKey) -> Option<&'static GuildCatalogEntry> {
    entries().iter().find(|entry| entry.key == key)
}

pub fn entry_for_persisted_key(key: &str) -> Option<&'static GuildCatalogEntry> {
    entries().iter().find(|entry| entry.persisted_key == key)
}

pub fn playable_entry_for_persisted_key(key: &str) -> Option<&'static GuildCatalogEntry> {
    entry_for_persisted_key(key).filter(|entry| entry.is_playable())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn persisted_keys_are_unique() {
        let mut keys = HashSet::new();
        for entry in entries() {
            assert!(
                keys.insert(entry.persisted_key),
                "duplicate guild key {}",
                entry.persisted_key
            );
        }
    }

    #[test]
    fn display_names_are_not_empty() {
        for entry in entries() {
            assert!(
                !entry.display_name.is_empty(),
                "empty display name for {}",
                entry.persisted_key
            );
        }
    }

    #[test]
    fn every_playable_entry_builds() {
        for entry in playable_entries() {
            assert!(
                entry.build().is_some(),
                "failed to build {}",
                entry.persisted_key
            );
        }
    }

    #[test]
    fn background_only_entries_build_but_are_not_playable() {
        for &(keyword, _) in THEMES_UX_ORDER {
            let entry = entry_for_persisted_key(keyword).expect("background entry");
            assert!(!entry.is_playable(), "{keyword} must not be playable");
            assert!(entry.build().is_some(), "{keyword} must build");
        }
    }

    #[test]
    fn catalog_includes_newly_playable_grouping_keywords() {
        assert!(entry_for_persisted_key("alchemists").is_some());
        assert!(entry_for_persisted_key("navigator").is_some());
        assert!(entry_for_persisted_key("treenav").is_some());
        assert!(playable_entry_for_persisted_key("alchemists").is_some());
    }
}
