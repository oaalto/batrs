use super::{
    AelenaGuild, AnimistGuild, BarbarianGuild, ChannellersGuild, CivmageGuild, CurateGuild,
    DiscipleGuild, FolkloristGuild, Guild, InnerCircleGuild, KharimGuild, LiberatorGuild,
    MageAcidGuild, MageAsphyxiationGuild, MageColdGuild, MageElectricityGuild, MageFireGuild,
    MageGuild, MageMagicalGuild, MagePoisonGuild, MonkGuild, NergalGuild, PsionicistGuild,
    RangerGuild, ReaverGuild, RiftwalkerGuild, SabresGuild, SeminaryGuild, SpiderGuild, TigerGuild,
    TriadGuild, TzarakkGuild,
};

mod selection;

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
    CivilizedFighters,
    Crimson,
    Curate,
    Disciple,
    Druids,
    Explorer,
    Folklorist,
    Inf,
    InnerCircle,
    Kharim,
    Knight,
    Liberator,
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
    Playable { build: GuildFactory },
    Unimplemented,
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
            GuildPlayability::Playable { build } => Some(build()),
            GuildPlayability::Unimplemented => None,
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

const fn unimplemented(
    key: GuildKey,
    persisted_key: &'static str,
    display_name: &'static str,
    grouping: GuildGroupingClass,
) -> GuildCatalogEntry {
    GuildCatalogEntry {
        key,
        persisted_key,
        display_name,
        grouping,
        playability: GuildPlayability::Unimplemented,
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
guild_factory!(build_animist, AnimistGuild);
guild_factory!(build_barbarian, BarbarianGuild);
guild_factory!(build_channellers, ChannellersGuild);
guild_factory!(build_civmage, CivmageGuild);
guild_factory!(build_curate, CurateGuild);
guild_factory!(build_disciple, DiscipleGuild);
guild_factory!(build_folklorist, FolkloristGuild);
guild_factory!(build_inner_circle, InnerCircleGuild);
guild_factory!(build_kharim, KharimGuild);
guild_factory!(build_liberator, LiberatorGuild);
guild_factory!(build_mage, MageGuild);
guild_factory!(build_mage_acid, MageAcidGuild);
guild_factory!(build_mage_asphyxiation, MageAsphyxiationGuild);
guild_factory!(build_mage_cold, MageColdGuild);
guild_factory!(build_mage_electricity, MageElectricityGuild);
guild_factory!(build_mage_fire, MageFireGuild);
guild_factory!(build_mage_magical, MageMagicalGuild);
guild_factory!(build_mage_poison, MagePoisonGuild);
guild_factory!(build_monk, MonkGuild);
guild_factory!(build_nergal, NergalGuild);
guild_factory!(build_psionicist, PsionicistGuild);
guild_factory!(build_ranger, RangerGuild);
guild_factory!(build_reaver, ReaverGuild);
guild_factory!(build_riftwalker, RiftwalkerGuild);
guild_factory!(build_sabres, SabresGuild);
guild_factory!(build_seminary, SeminaryGuild);
guild_factory!(build_spider, SpiderGuild);
guild_factory!(build_tiger, TigerGuild);
guild_factory!(build_triad, TriadGuild);
guild_factory!(build_tzarakk, TzarakkGuild);

pub static GUILD_CATALOG: &[GuildCatalogEntry] = &[
    unimplemented(
        GuildKey::Alchemists,
        "alchemists",
        "Alchemists",
        GuildGroupingClass::Thematic(0),
    ),
    unimplemented(
        GuildKey::CivilizedFighters,
        "civilized_fighters",
        "Civilized Fighters",
        GuildGroupingClass::Thematic(0),
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
    unimplemented(
        GuildKey::Knight,
        "knight",
        "Knight",
        GuildGroupingClass::Thematic(0),
    ),
    unimplemented(
        GuildKey::Merchant,
        "merchant",
        "Merchant",
        GuildGroupingClass::Thematic(0),
    ),
    unimplemented(
        GuildKey::Runemages,
        "runemages",
        "Runemages",
        GuildGroupingClass::Thematic(0),
    ),
    playable(
        GuildKey::Sabres,
        "sabres",
        "Sabres",
        GuildGroupingClass::Thematic(0),
        build_sabres,
    ),
    unimplemented(
        GuildKey::Bard,
        "bard",
        "Bard",
        GuildGroupingClass::Thematic(0),
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
    playable(
        GuildKey::Animist,
        "animist",
        "Animist",
        GuildGroupingClass::Thematic(2),
        build_animist,
    ),
    unimplemented(
        GuildKey::Druids,
        "druids",
        "Druids",
        GuildGroupingClass::Thematic(2),
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
    unimplemented(GuildKey::Nun, "nun", "Nun", GuildGroupingClass::Thematic(2)),
    unimplemented(
        GuildKey::Tarmalen,
        "tarmalen",
        "Tarmalen",
        GuildGroupingClass::Thematic(2),
    ),
    unimplemented(
        GuildKey::Templar,
        "templar",
        "Templar",
        GuildGroupingClass::Thematic(2),
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
    unimplemented(
        GuildKey::Archers,
        "archers",
        "Archers",
        GuildGroupingClass::Thematic(4),
    ),
    playable(
        GuildKey::Barbarian,
        "barbarian",
        "Barbarian",
        GuildGroupingClass::Thematic(4),
        build_barbarian,
    ),
    unimplemented(
        GuildKey::Beastmaster,
        "beastmaster",
        "Beastmaster",
        GuildGroupingClass::Thematic(4),
    ),
    playable(
        GuildKey::Ranger,
        "ranger",
        "Ranger",
        GuildGroupingClass::Thematic(4),
        build_ranger,
    ),
    unimplemented(
        GuildKey::Crimson,
        "crimson",
        "Crimson",
        GuildGroupingClass::Thematic(4),
    ),
    unimplemented(
        GuildKey::Cavalier,
        "cavalier",
        "Cavalier",
        GuildGroupingClass::Multi,
    ),
    playable(
        GuildKey::Disciple,
        "disciple",
        "Disciple",
        GuildGroupingClass::Multi,
        build_disciple,
    ),
    unimplemented(
        GuildKey::Explorer,
        "explorer",
        "Explorer",
        GuildGroupingClass::Multi,
    ),
    unimplemented(GuildKey::Inf, "inf", "Inf", GuildGroupingClass::Multi),
    playable(
        GuildKey::Kharim,
        "kharim",
        "Kharim",
        GuildGroupingClass::Multi,
        build_kharim,
    ),
    unimplemented(
        GuildKey::Navigator,
        "navigator",
        "Navigator",
        GuildGroupingClass::Multi,
    ),
    unimplemented(
        GuildKey::Sailor,
        "sailor",
        "Sailor",
        GuildGroupingClass::Multi,
    ),
    unimplemented(
        GuildKey::Squire,
        "squire",
        "Squire",
        GuildGroupingClass::Multi,
    ),
    unimplemented(
        GuildKey::Treenav,
        "treenav",
        "Treenav",
        GuildGroupingClass::Multi,
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
    fn unimplemented_entries_do_not_build() {
        let unimplemented = entries().iter().filter(|entry| !entry.is_playable());
        for entry in unimplemented {
            assert!(
                entry.build().is_none(),
                "unimplemented entry built {}",
                entry.persisted_key
            );
        }
    }

    #[test]
    fn catalog_includes_unimplemented_grouping_keywords() {
        assert!(entry_for_persisted_key("alchemists").is_some());
        assert!(entry_for_persisted_key("navigator").is_some());
        assert!(entry_for_persisted_key("treenav").is_some());
    }
}
