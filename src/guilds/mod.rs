mod aelena;
mod animist;
mod barbarian;
pub mod catalog;
mod channellers;
mod civmage;
mod curate;
mod disciple;
mod folklorist;
pub mod grouping;
mod inner_circle;
mod kharim;
mod liberator;
mod mage;
mod mage_acid;
mod mage_asphyxiation;
mod mage_cold;
mod mage_electricity;
mod mage_fire;
mod mage_magical;
mod mage_poison;
mod magic_lore_analysis;
mod monk;
mod nergal;
mod psionicist;
mod ranger;
mod reaver;
mod riftwalker;
mod sabres;
mod sects_triggers;
mod seminary;
mod spider;
mod tiger;
mod triad;
mod tzarakk;

pub use aelena::AelenaGuild;
pub use animist::AnimistGuild;
pub use barbarian::BarbarianGuild;
pub use channellers::ChannellersGuild;
pub use civmage::CivmageGuild;
pub use curate::CurateGuild;
pub use disciple::DiscipleGuild;
pub use folklorist::FolkloristGuild;
pub use inner_circle::InnerCircleGuild;
pub use kharim::KharimGuild;
pub use liberator::LiberatorGuild;
pub use mage::MageGuild;
pub use mage_acid::MageAcidGuild;
pub use mage_asphyxiation::MageAsphyxiationGuild;
pub use mage_cold::MageColdGuild;
pub use mage_electricity::MageElectricityGuild;
pub use mage_fire::MageFireGuild;
pub use mage_magical::MageMagicalGuild;
pub use mage_poison::MagePoisonGuild;
pub use monk::MonkGuild;
pub use nergal::NergalGuild;
pub use psionicist::PsionicistGuild;
pub use ranger::RangerGuild;
pub use reaver::ReaverGuild;
pub use riftwalker::RiftwalkerGuild;
pub use sabres::SabresGuild;
pub use seminary::SeminaryGuild;
pub use spider::SpiderGuild;
pub use tiger::TigerGuild;
pub use triad::TriadGuild;
pub use tzarakk::TzarakkGuild;

use crate::automation::Automation;
use crate::command::{Command, Data};
use crate::triggers::Trigger;
use std::collections::HashMap;

pub trait Guild {
    fn commands(&self) -> HashMap<String, Command>;
    fn triggers(&self) -> Vec<Trigger>;
    fn register_automation(&self, _automation: &mut Automation) {}
}

pub fn use_skill(skill_name: &str, data: &Data) -> String {
    crate::abilities::use_skill(skill_name, data)
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    crate::abilities::cast_spell(spell_name, data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn use_skill_builds_targeted_commands() {
        let empty = Data {
            cmd: "use".to_string(),
            args: "".to_string(),
        };
        let with_args = Data {
            cmd: "use".to_string(),
            args: "orc".to_string(),
        };

        assert_eq!(use_skill("scythe swipe", &empty), "@use 'scythe swipe'");
        assert_eq!(
            use_skill("scythe swipe", &with_args),
            "@target orc;use 'scythe swipe' orc"
        );
    }

    #[test]
    fn cast_spell_builds_targeted_commands() {
        let empty = Data {
            cmd: "cast".to_string(),
            args: "".to_string(),
        };
        let with_args = Data {
            cmd: "cast".to_string(),
            args: "goblin".to_string(),
        };

        assert_eq!(cast_spell("word of spite", &empty), "@cast 'word of spite'");
        assert_eq!(
            cast_spell("word of spite", &with_args),
            "@target goblin;cast 'word of spite' goblin"
        );
    }
}
