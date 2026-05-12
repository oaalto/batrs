use crate::abilities;
use crate::ansi::StyledLine;
use crate::automation::Action;
use crate::command;
use crate::command::Command;
use crate::guilds::{ReaverGuild, use_skill};
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

    pub fn reaver_threaten_logical(data: &command::Data) -> String {
        if !data.args.is_empty() {
            format!("reaver threaten {}", &data.args)
        } else {
            String::default()
        }
    }

    fn compound_threaten_use(data: &command::Data, skill: &str) -> String {
        let threaten = Self::reaver_threaten_logical(data);
        let use_part = abilities::targeted_use(skill, &data.args);
        if threaten.is_empty() {
            abilities::client_send_line(&use_part)
        } else {
            abilities::client_send_line(&format!("{threaten};{use_part}"))
        }
    }

    fn compound_threaten_cast(data: &command::Data, spell: &str) -> String {
        let threaten = Self::reaver_threaten_logical(data);
        let cast_part = abilities::targeted_cast(spell, &data.args);
        if threaten.is_empty() {
            abilities::client_send_line(&cast_part)
        } else {
            abilities::client_send_line(&format!("{threaten};{cast_part}"))
        }
    }

    pub fn command_reaver_threaten(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            Some(abilities::client_send_line(&format!(
                "reaver threaten {}",
                &data.args
            )))
        }
    }

    // SKILLS

    pub fn use_scythe_swipe(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "scythe swipe"))
    }

    pub fn use_rampant_cutting(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "rampant cutting"))
    }

    pub fn use_reaver_strike(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "reaver strike"))
    }

    pub fn use_blood_harvest(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "blood harvest"))
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
        Some(Self::compound_threaten_use(data, "true reaving"))
    }

    pub fn use_corrosive_cut(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "corrosive cut"))
    }

    pub fn use_breath_of_doom(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_use(data, "breath of doom"))
    }

    pub fn use_prayer_to_destruction(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if !data.args.is_empty() {
            return Some(abilities::client_send_line(&format!(
                "use 'prayer to destruction' at {}",
                &data.args
            )));
        }
        ctx.push_output_line(StyledLine::new("No target!"));
        None
    }

    // SPELLS

    pub fn cast_word_of_spite(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_cast(data, "word of spite"))
    }

    pub fn cast_word_of_blasting(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_cast(data, "word of blasting"))
    }

    pub fn cast_word_of_destruction(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_cast(data, "word of destruction"))
    }

    pub fn cast_word_of_slaughter(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_cast(data, "word of slaughter"))
    }

    pub fn cast_word_of_genocide(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(Self::compound_threaten_cast(data, "word of genocide"))
    }

    pub fn cast_word_of_attrition(
        data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            Some(abilities::client_send_line(&format!(
                "cast 'word of attrition' at {}",
                &data.args
            )))
        }
    }

    pub fn cast_shattered_feast(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some(abilities::client_send_line(
                "cast 'shattered feast' at amount 100",
            ));
        }

        set_prayer_flag(ctx, "cast_shattered_feast");
        Some(abilities::client_send_line(
            "use 'prayer to destruction' at spell",
        ))
    }

    pub fn cast_black_hole(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some(abilities::client_send_line("cast 'black hole'"));
        }

        set_prayer_flag(ctx, "cast_black_hole");
        Some(abilities::client_send_line(
            "use 'prayer to destruction' at spell",
        ))
    }

    pub fn cast_blood_seeker(
        _data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if ctx.flag("prayer_done") {
            return Some(abilities::client_send_line(
                "cast 'blood seeker' at amount 100",
            ));
        }

        set_prayer_flag(ctx, "cast_blood_seeker");
        Some(abilities::client_send_line(
            "use 'prayer to destruction' at spell",
        ))
    }

    pub fn cast_reaping_of_bile(
        _data: &command::Data,
        _ctx: &mut command::CommandContext,
    ) -> Option<String> {
        Some(abilities::client_send_line("cast 'reaping of bile'"))
    }

    pub fn cast_call_armour(
        data: &command::Data,
        ctx: &mut command::CommandContext,
    ) -> Option<String> {
        if data.args.is_empty() {
            None
        } else {
            if ctx.flag("prayer_done") {
                return Some(abilities::client_send_line(&format!(
                    "cast 'call armour' at amount {}",
                    &data.args
                )));
            }

            ctx.push_action(Action::SetVar(
                "call_armour_amount".to_string(),
                data.args.clone(),
            ));
            set_prayer_flag(ctx, "cast_call_armour");
            Some(abilities::client_send_line(
                "use 'prayer to destruction' at spell",
            ))
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
                return Some(abilities::client_send_line(&format!(
                    "cast 'spirit drain' at {} amount 100",
                    &data.args
                )));
            }

            ctx.push_action(Action::SetVar(
                "spirit_drain_target".to_string(),
                data.args.clone(),
            ));
            set_prayer_flag(ctx, "cast_spirit_drain");
            Some(abilities::client_send_line(
                "use 'prayer to destruction' at spell",
            ))
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn prayer_to_destruction_requires_target() {
        let data = command::Data {
            cmd: "upd".to_string(),
            args: "".to_string(),
        };
        let mut ctx = command::CommandContext::new(HashMap::new(), true, String::new());

        let result = ReaverGuild::use_prayer_to_destruction(&data, &mut ctx);

        assert!(result.is_none());
        assert_eq!(ctx.output_lines.len(), 1);
        assert_eq!(ctx.output_lines[0].plain_line, "No target!");
    }
}
