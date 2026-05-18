//! Canonical `use` / `cast` command lines and client send wrapping.
//!
//! Logical lines omit the leading `@`; [`client_send_line`] adds at most one `@` for the whole line.

pub mod floating_disc;

use crate::command::Data;

/// Add the client send prefix when the logical line does not already have one.
pub fn client_send_line(logical_line: &str) -> String {
    if logical_line.is_empty() {
        return String::new();
    }
    if logical_line.starts_with('@') {
        logical_line.to_owned()
    } else {
        format!("@{logical_line}")
    }
}

/// `use '<skill>'`, or `target <t>;use '<skill>' <t>` when `target_args` is non-empty.
pub fn targeted_use(skill: &str, target_args: &str) -> String {
    let target_args = target_args.trim();
    if target_args.is_empty() {
        format!("use '{skill}'")
    } else {
        format!(
            "target {target_args};use '{skill}' {target_args}",
            target_args = target_args,
            skill = skill
        )
    }
}

/// `cast '<spell>'`, or `target <t>;cast '<spell>' <t>` when `target_args` is non-empty.
pub fn targeted_cast(spell: &str, target_args: &str) -> String {
    let target_args = target_args.trim();
    if target_args.is_empty() {
        format!("cast '{spell}'")
    } else {
        format!(
            "target {target_args};cast '{spell}' {target_args}",
            target_args = target_args,
            spell = spell
        )
    }
}

/// Cast with an extra tail after the quoted name (`cast 'spark birth' troll`).
pub fn cast_quoted_with_suffix(spell: &str, suffix: &str) -> String {
    let suffix = suffix.trim();
    let logical = if suffix.is_empty() {
        format!("cast '{spell}'")
    } else {
        format!("cast '{spell}' {suffix}")
    };
    client_send_line(&logical)
}

pub fn use_skill(skill_name: &str, data: &Data) -> String {
    client_send_line(&targeted_use(skill_name, &data.args))
}

pub fn cast_spell(spell_name: &str, data: &Data) -> String {
    client_send_line(&targeted_cast(spell_name, &data.args))
}

/// Join logical fragments with `;`, then add a single client prefix.
pub fn compound_send(parts: &[&str]) -> String {
    client_send_line(&parts.join(";"))
}

/// `repeat inf cast heal self` (Civmage / Mage / Psionicist `chf`).
pub fn repeat_inf_cast_heal_self() -> String {
    client_send_line("repeat inf cast heal self")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn client_prefix_applies_once() {
        assert_eq!(client_send_line("use 'x'"), "@use 'x'");
        assert_eq!(client_send_line("@use 'x'"), "@use 'x'");
    }

    #[test]
    fn targeted_use_and_cast() {
        assert_eq!(targeted_use("iron palm", ""), "use 'iron palm'");
        assert_eq!(
            targeted_use("iron palm", "orc"),
            "target orc;use 'iron palm' orc"
        );
        assert_eq!(targeted_cast("tiger claw", ""), "cast 'tiger claw'");
        assert_eq!(
            targeted_cast("tiger claw", "orc"),
            "target orc;cast 'tiger claw' orc"
        );
    }

    #[test]
    fn use_skill_matches_targeted_form() {
        let empty = Data {
            cmd: String::new(),
            args: String::new(),
        };
        let with_args = Data {
            cmd: String::new(),
            args: "orc".to_string(),
        };
        assert_eq!(use_skill("scythe swipe", &empty), "@use 'scythe swipe'");
        assert_eq!(
            use_skill("scythe swipe", &with_args),
            "@target orc;use 'scythe swipe' orc"
        );
    }

    #[test]
    fn compound_send_one_prefix() {
        assert_eq!(
            compound_send(&["dismount", "use 'meditation'"]),
            "@dismount;use 'meditation'"
        );
    }

    #[test]
    fn repeat_inf_cast_heal_self_line() {
        assert_eq!(repeat_inf_cast_heal_self(), "@repeat inf cast heal self");
    }
}
