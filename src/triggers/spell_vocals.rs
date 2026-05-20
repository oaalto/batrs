//! Annotates spell chant lines using static vocal → spell-name tables in `spell_vocal_data.rs`.
//! After a quoted vocal `'…'`, inserts ` (Spell name)` (or ` (A, B, …)` when several spells share the
//! same vocal).

#[cfg(test)]
use crate::ansi::StyledLine;
use crate::triggers::{TriggerEffects, TriggerFacts, TriggerLine};
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

fn has_quoted_vocal_boundary_after(plain: &str, quote_end_byte_idx: usize) -> bool {
    plain[quote_end_byte_idx..]
        .chars()
        .next()
        .is_none_or(|next| !is_word_continuation(next))
}

fn is_word_continuation(ch: char) -> bool {
    ch.is_alphanumeric() || ch == '_'
}

pub fn trigger(line: &TriggerLine<'_>, _facts: &TriggerFacts) -> TriggerEffects {
    let plain = line.plain_line;

    for rule in CONTEXTUAL_RULES.iter() {
        if let Some(found) = rule.regex.find(plain) {
            let label = format!(" ({})", rule.spell);
            return TriggerEffects::none().insert_plain_after_plain_byte_idx(found.end(), label);
        }
    }

    for rule in QUOTED_VOCAL_RULES.iter() {
        if let Some(found) = rule
            .regex
            .find_iter(plain)
            .find(|found| has_quoted_vocal_boundary_after(plain, found.end()))
        {
            return TriggerEffects::none()
                .insert_plain_after_plain_byte_idx(found.end(), format_label(&rule.spells));
        }
    }

    TriggerEffects::none()
}

#[cfg(test)]
pub fn annotate(styled_line: &mut StyledLine) {
    let effects = trigger(
        &TriggerLine::new(&styled_line.plain_line),
        &TriggerFacts::default(),
    );
    for edit in effects.original.edits {
        edit.apply_to(styled_line);
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

    #[test]
    fn does_not_annotate_single_letter_vocal_inside_contraction() {
        let original = "You tell Farliss (*<|   W & T   |>*) 'I'd like a reinc'";
        let mut line = StyledLine::new(original);
        annotate(&mut line);
        assert_eq!(line.plain_line, original);
    }

    #[test]
    fn annotates_standalone_single_letter_vocal() {
        let mut line = StyledLine::new("You chant 'I' loudly.");
        annotate(&mut line);
        assert_eq!(
            line.plain_line,
            "You chant 'I' (Blessing of intoxication, Spirit drain) loudly."
        );
    }
}
