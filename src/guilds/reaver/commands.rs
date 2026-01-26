use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::{cast_spell, use_skill, ReaverGuild};
use std::collections::HashMap;

impl ReaverGuild {
    pub fn get_commands(&self) -> HashMap<String, Command> {
        HashMap::from([
            ("rt".to_string(), Self::command_reaver_threaten as Command),
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
            ("uccut".to_string(), Self::use_corrosive_cut as Command),
            ("ubd".to_string(), Self::use_breath_of_doom as Command),
            (
                "upd".to_string(),
                Self::use_prayer_to_destruction as Command,
            ),
            // Spells
            ("cws".to_string(), Self::cast_word_of_spite as Command),
            ("cb".to_string(), Self::cast_word_of_blasting as Command),
            ("cwd".to_string(), Self::cast_word_of_destruction as Command),
            ("cwa".to_string(), Self::cast_word_of_attrition as Command),
            ("cwb".to_string(), Self::cast_word_of_blasting as Command),
            ("cwsl".to_string(), Self::cast_word_of_slaughter as Command),
            ("cwg".to_string(), Self::cast_word_of_genocide as Command),
            ("csf".to_string(), Self::cast_shattered_feast as Command),
            ("cbh".to_string(), Self::cast_black_hole as Command),
            ("cbs".to_string(), Self::cast_blood_seeker as Command),
            ("crb".to_string(), Self::cast_reaping_of_bile as Command),
            ("cca".to_string(), Self::cast_call_armour as Command),
            ("csd".to_string(), Self::cast_spirit_drain as Command),
        ])
    }

    pub fn reaver_threaten(data: &command::Data) -> String {
        if !data.args.is_empty() {
            return format!("@reaver threaten {}", &data.args);
        }
        String::default()
    }

    pub fn command_reaver_threaten(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            Some(format!("@reaver threaten {}", &data.args))
        }
    }

    // SKILLS

    pub fn use_scythe_swipe(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("scythe swipe", data)
        ))
    }

    pub fn use_rampant_cutting(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("rampant cutting", data)
        ))
    }

    pub fn use_reaver_strike(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("reaver strike", data)
        ))
    }

    pub fn use_blood_harvest(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("blood harvest", data)
        ))
    }

    pub fn use_reave_shield(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("reave shield", data))
    }

    pub fn use_reave_weapon(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("reave weapon", data))
    }

    pub fn use_reave_armour(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(use_skill("reave armour", data))
    }

    pub fn use_true_reaving(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("true reaving", data)
        ))
    }

    pub fn use_corrosive_cut(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("corrosive cut", data)
        ))
    }

    pub fn use_breath_of_doom(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            use_skill("breath of doom", data)
        ))
    }

    pub fn use_prayer_to_destruction(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if !data.args.is_empty() {
            return Some(format!("@use prayer to destruction at {}", &data.args));
        }
        None
    }

    // SPELLS

    pub fn cast_word_of_spite(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of spite", data)
        ))
    }

    pub fn cast_word_of_blasting(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of blasting", data)
        ))
    }

    pub fn cast_word_of_destruction(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of destruction", data)
        ))
    }

    pub fn cast_word_of_slaughter(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of slaughter", data)
        ))
    }

    pub fn cast_word_of_genocide(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(format!(
            "{};{}",
            Self::reaver_threaten(data),
            cast_spell("word of genocide", data)
        ))
    }

    pub fn cast_word_of_attrition(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            Some(format!("@cast word of attrition at {}", &data.args))
        }
    }

    pub fn cast_shattered_feast(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some("@cast shattered feast at amount 100".to_string());
        }

        set_prayer_flag(ctx, "cast_shattered_feast");
        Some("@use prayer to destruction at spell".to_string())
    }

    pub fn cast_black_hole(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some("@cast black hole".to_string());
        }

        set_prayer_flag(ctx, "cast_black_hole");
        Some("@use prayer to destruction at spell".to_string())
    }

    pub fn cast_blood_seeker(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some("@cast blood seeker at amount 100".to_string());
        }

        set_prayer_flag(ctx, "cast_blood_seeker");
        Some("@use prayer to destruction at spell".to_string())
    }

    pub fn cast_reaping_of_bile(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some("@cast reaping of bile".to_string())
    }

    pub fn cast_call_armour(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            if ctx.flag("prayer_done") {
                return Some(format!("@cast call armour at amount {}", &data.args));
            }

            ctx.push_action(Action::SetVar(
                "call_armour_amount".to_string(),
                data.args.clone(),
            ));
            set_prayer_flag(ctx, "cast_call_armour");
            Some("@use prayer to destruction at spell".to_string())
        }
    }

    pub fn cast_spirit_drain(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            if ctx.flag("prayer_done") {
                return Some(format!("@cast spirit drain at {} amount 100", &data.args));
            }

            ctx.push_action(Action::SetVar(
                "spirit_drain_target".to_string(),
                data.args.clone(),
            ));
            set_prayer_flag(ctx, "cast_spirit_drain");
            Some("@use prayer to destruction at spell".to_string())
        }
    }
}

fn set_prayer_flag(ctx: &mut command::CommandContext, flag: &str) {
    let flags = [
        "cast_shattered_feast",
        "cast_blood_seeker",
        "cast_call_armour",
        "cast_spirit_drain",
        "cast_black_hole",
    ];

    for existing in flags {
        if existing != flag {
            ctx.push_action(Action::ClearFlag(existing.to_string()));
        }
    }

    ctx.push_action(Action::SetFlag(flag.to_string(), true));
}
