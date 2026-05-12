//! Annotates spell chant lines using static vocal → spell-name tables in `spell_vocal_data.rs`.
//! After a quoted vocal `'…'`, inserts ` (Spell name)` (or ` (A, B, …)` when several spells share the
//! same vocal).

use crate::ansi::StyledLine;
use lazy_static::lazy_static;
use regex::Regex;

struct ContextualRule {
    regex: Regex,
    spell: &'static str,
}

struct QuotedVocalRule {
    regex: Regex,
    spells: Vec<String>,
}

lazy_static! {
    /// Tiger spells share `'(Haii!)'`; disambiguate using surrounding line context.
    static ref CONTEXTUAL_RULES: Vec<ContextualRule> = vec![
        ContextualRule {
            regex: Regex::new(r"(?i)large\s+circle\s+to\s+the\s+air.*?'\(Haii!\)'").unwrap(),
            spell: "Shadow leap",
        },
        ContextualRule {
            regex: Regex::new(r"(?i)shred.*?air.*?'\(Haii!\)'").unwrap(),
            spell: "Tiger claw",
        },
    ];

    static ref QUOTED_VOCAL_RULES: Vec<QuotedVocalRule> = super::spell_vocal_data::VOCAL_SPELL_GROUPS
        .iter()
        .filter_map(|(inner, spells)| {
            let regex = Regex::new(&format!("(?i)'{}'", regex::escape(inner))).ok()?;
            Some(QuotedVocalRule {
                regex,
                spells: spells.iter().map(|spell| (*spell).to_string()).collect(),
            })
        })
        .collect();
}

fn format_label(spells: &[String]) -> String {
    format!(" ({})", spells.join(", "))
}

pub fn annotate(styled_line: &mut StyledLine) {
    let plain = styled_line.plain_line.clone();

    for rule in CONTEXTUAL_RULES.iter() {
        if let Some(found) = rule.regex.find(&plain) {
            let label = format!(" ({})", rule.spell);
            styled_line.insert_plain_after_plain_byte_idx(found.end(), &label);
            return;
        }
    }

    for rule in QUOTED_VOCAL_RULES.iter() {
        if let Some(found) = rule.regex.find(&plain) {
            styled_line.insert_plain_after_plain_byte_idx(found.end(), &format_label(&rule.spells));
            return;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inserts_after_quote_water_walking_example() {
        let mut line =
            StyledLine::new("You rub wax in your feet and chant 'Jeeeeeeeeeeeesuuuuuuuus'");
        annotate(&mut line);
        assert_eq!(
            line.plain_line,
            "You rub wax in your feet and chant 'Jeeeeeeeeeeeesuuuuuuuus' (Water walking)"
        );
    }

    #[test]
    fn inserts_before_trailing_text() {
        let mut line = StyledLine::new(
            "You rub wax in your feet and chant 'Jeeeeeeeeeeeesuuuuuuuus' then stop.",
        );
        annotate(&mut line);
        assert_eq!(
            line.plain_line,
            "You rub wax in your feet and chant 'Jeeeeeeeeeeeesuuuuuuuus' (Water walking) then stop."
        );
    }

    #[test]
    fn ambiguous_ner_lists_all_spells_sorted() {
        let mut line = StyledLine::new("Some chant 'ner' here.");
        annotate(&mut line);
        assert!(
            line.plain_line.starts_with("Some chant 'ner' ("),
            "{}",
            line.plain_line
        );
        assert!(line.plain_line.ends_with(") here."));
        assert!(line.plain_line.contains("Awake enthralled"));
        assert!(line.plain_line.contains("Suppressing misery"));
    }

    #[test]
    fn haii_shadow_leap_context_single_spell() {
        let mut line = StyledLine::new(
            "You draw a large circle to the air with your fingers and chant '(Haii!)'",
        );
        annotate(&mut line);
        assert!(
            line.plain_line.contains("(Shadow leap)"),
            "{}",
            line.plain_line
        );
        assert!(!line.plain_line.contains("Tiger claw"));
    }

    #[test]
    fn haii_without_context_lists_both_spells() {
        let mut line = StyledLine::new("You chant '(Haii!)' softly.");
        annotate(&mut line);
        assert!(
            line.plain_line.contains("Shadow leap"),
            "{}",
            line.plain_line
        );
        assert!(
            line.plain_line.contains("Tiger claw"),
            "{}",
            line.plain_line
        );
    }
}
