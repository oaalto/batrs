use crate::command;
use crate::command::Command;
use crate::guilds::{cast_spell, use_skill, ReaverGuild};
use egui::Context;
use std::collections::HashMap;

impl ReaverGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            // Skills
            ("uss".to_string(), Self::use_scythe_swipe as Command),
            ("urc".to_string(), Self::use_rampant_cutting as Command),
            ("urs".to_string(), Self::use_reaver_strike as Command),
            ("ubh".to_string(), Self::use_blood_harvest as Command),
            ("res".to_string(), Self::use_reave_shield as Command),
            ("rew".to_string(), Self::use_reave_weapon as Command),
            ("rea".to_string(), Self::use_reave_armour as Command),
            ("utr".to_string(), Self::use_true_reaving as Command),
            ("ucc".to_string(), Self::use_corrosive_cut as Command),
            ("ubd".to_string(), Self::use_breath_of_doom as Command),
            (
                "upd".to_string(),
                Self::use_prayer_to_destruction as Command,
            ),
            // Spells
            ("cws".to_string(), Self::cast_word_of_spite as Command),
            ("cwd".to_string(), Self::cast_word_of_destruction as Command),
            ("cwa".to_string(), Self::cast_word_of_attrition as Command),
            ("cwb".to_string(), Self::cast_word_of_blasting as Command),
            ("cwsl".to_string(), Self::cast_word_of_slaughter as Command),
            ("cwg".to_string(), Self::cast_word_of_genocide as Command),
        ])
    }

    pub fn reaver_threaten(data: &command::Data) -> String {
        if !data.args.is_empty() {
            return format!("reaver threaten {}", &data.args);
        }
        String::default()
    }

    // SKILLS

    pub fn use_scythe_swipe(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("scythe swipe", data)
        ))
    }

    pub fn use_rampant_cutting(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("rampant cutting", data)
        ))
    }

    pub fn use_reaver_strike(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("reaver strike", data)
        ))
    }

    pub fn use_blood_harvest(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("blood harvest", data)
        ))
    }

    pub fn use_reave_shield(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("reave shield", data)
        ))
    }

    pub fn use_reave_weapon(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("reave weapon", data)
        ))
    }

    pub fn use_reave_armour(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("reave armour", data)
        ))
    }

    pub fn use_true_reaving(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("true reaving", data)
        ))
    }

    pub fn use_corrosive_cut(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("corrosive cut", data)
        ))
    }

    pub fn use_breath_of_doom(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("breath of doom", data)
        ))
    }

    pub fn use_prayer_to_destruction(data: &command::Data, _ctx: &Context) -> Option<String> {
        if !data.args.is_empty() {
            return Some(format!("use prayer to destruction at {}", &data.args));
        }
        None
    }

    // SPELLS

    pub fn cast_word_of_spite(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of spite", data)
        ))
    }

    pub fn cast_word_of_blasting(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of blasting", data)
        ))
    }

    pub fn cast_word_of_destruction(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("word of destruction", data)
        ))
    }

    pub fn cast_word_of_slaughter(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("word of slaughter", data)
        ))
    }

    pub fn cast_word_of_genocide(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("word of genocide", data)
        ))
    }

    pub fn cast_word_of_attrition(data: &command::Data, _ctx: &Context) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("word of attrition", data)
        ))
    }
}
