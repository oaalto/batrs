use crate::combat_damage::catalog::{CATALOG, CatalogEntry};
use crate::combat_damage::conjugate::conjugate_verb;
use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DamageCategory {
    Melee,
    Skill,
    Spell,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DamageCandidate {
    pub category: DamageCategory,
    pub source_name: String,
    pub message_verb: String,
    pub message_text: String,
}

#[derive(Debug, Default)]
pub struct Matcher {
    last_family: Option<usize>,
}

struct SkillPattern {
    verb: &'static str,
    regex: &'static str,
}

static SKILL_PATTERNS: &[SkillPattern] = &[
    SkillPattern {
        verb: "kick",
        regex: r"^(.+) kicks you in the groin very hard\. You gasp with pain and double up\.$",
    },
    SkillPattern {
        verb: "kick",
        regex: r"^(.+) performs a quick kick to your stomach, almost making you lose your breakfast\.$",
    },
    SkillPattern {
        verb: "kick",
        regex: r"^(.+)'s kick lashes at you with speed, but you manage to partly deflect it in time\.$",
    },
    SkillPattern {
        verb: "stab",
        regex: r"^With a quick flick, (.+) knocks your weapon aside and stabs your stomach!$",
    },
    SkillPattern {
        verb: "stab",
        regex: r"^You watch helplessly as (.+) smashes your kneecap!$",
    },
    SkillPattern {
        verb: "stab",
        regex: r"^OOF!\s+(.+) feints, throwing you offguard as he PUMMELS your midriff!$",
    },
    SkillPattern {
        verb: "scythe swipe",
        regex: r"^(.+) slashes a ragged wound across your chest\.$",
    },
    SkillPattern {
        verb: "bash",
        regex: r"^(.+)'s bash sends you sprawling\.$",
    },
    SkillPattern {
        verb: "push",
        regex: r"^(.+) pushes you\.$",
    },
];

static SPELL_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^An? (.+) hits you\.$").expect("spell regex"));

static SKILL_REGEXES: LazyLock<Vec<(Regex, &'static str)>> = LazyLock::new(|| {
    SKILL_PATTERNS
        .iter()
        .map(|pattern| {
            (
                Regex::new(pattern.regex).expect("skill regex"),
                pattern.verb,
            )
        })
        .collect()
});

impl Matcher {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn reset(&mut self) {
        self.last_family = None;
    }

    pub fn match_incoming(&mut self, line: &str) -> Option<DamageCandidate> {
        if let Some(candidate) = match_skill(line) {
            return Some(candidate);
        }
        if let Some(candidate) = match_spell(line) {
            return Some(candidate);
        }
        self.match_melee(line)
    }

    pub fn match_outgoing_sanity(&self, line: &str) -> Option<(String, String)> {
        let rest = line.strip_prefix("You ")?;
        for entry in CATALOG {
            if let Some(target) = rest
                .strip_prefix(entry.canonical)
                .and_then(|s| s.strip_prefix(' '))
                && target.ends_with('.')
            {
                return Some((
                    entry.canonical.to_string(),
                    target.trim_end_matches('.').to_string(),
                ));
            }
            let conjugated = conjugate_verb(entry.canonical);
            if let Some(target) = rest
                .strip_prefix(&conjugated)
                .and_then(|s| s.strip_prefix(' '))
                && target.ends_with('.')
            {
                return Some((
                    entry.canonical.to_string(),
                    target.trim_end_matches('.').to_string(),
                ));
            }
        }
        None
    }

    fn match_melee(&mut self, line: &str) -> Option<DamageCandidate> {
        if !line_ends_with_you(line) {
            return None;
        }

        let mut best: Option<(usize, String)> = None;
        for idx in self.melee_search_order() {
            let entry = &CATALOG[idx];
            if let Some(source) = match_catalog_entry(line, entry) {
                let len = entry.canonical.len();
                if best.as_ref().is_none_or(|(best_len, _)| len > *best_len) {
                    best = Some((idx, source));
                }
            }
        }

        let (idx, source) = best?;
        let entry = &CATALOG[idx];
        self.last_family = Some(entry.family);
        Some(DamageCandidate {
            category: DamageCategory::Melee,
            source_name: source,
            message_verb: entry.canonical.to_string(),
            message_text: line.to_string(),
        })
    }

    fn melee_search_order(&self) -> Vec<usize> {
        let mut recency: Vec<usize> = Vec::new();
        let mut rest: Vec<usize> = Vec::new();

        for (idx, entry) in CATALOG.iter().enumerate() {
            if Some(entry.family) == self.last_family {
                recency.push(idx);
            } else {
                rest.push(idx);
            }
        }

        let by_len = |a: &usize, b: &usize| {
            CATALOG[*b]
                .canonical
                .len()
                .cmp(&CATALOG[*a].canonical.len())
        };
        recency.sort_by(by_len);
        rest.sort_by(by_len);
        recency.extend(rest);
        recency
    }

    #[doc(hidden)]
    pub fn match_melee_for_test(&mut self, line: &str) -> Option<DamageCandidate> {
        self.match_melee(line)
    }
}

fn line_ends_with_you(line: &str) -> bool {
    line.trim_end().to_ascii_lowercase().ends_with(" you.")
}

fn match_catalog_entry(line: &str, entry: &CatalogEntry) -> Option<String> {
    if let Some(source) = strip_suffix_case_insensitive(line, entry.conjugated_suffix) {
        return Some(source);
    }
    strip_suffix_case_insensitive(line, entry.bare_suffix)
}

fn strip_suffix_case_insensitive(line: &str, suffix: &str) -> Option<String> {
    let line_lower = line.to_ascii_lowercase();
    let suffix_lower = suffix.to_ascii_lowercase();
    let source = line_lower.strip_suffix(&suffix_lower)?;
    let source = source.trim_end();
    if source.is_empty() {
        return None;
    }
    let end = source.len();
    Some(line[..end].to_string())
}

fn match_skill(line: &str) -> Option<DamageCandidate> {
    for (regex, verb) in SKILL_REGEXES.iter() {
        if let Some(captures) = regex.captures(line) {
            let source_name = captures.get(1)?.as_str().to_string();
            return Some(DamageCandidate {
                category: DamageCategory::Skill,
                source_name,
                message_verb: (*verb).to_string(),
                message_text: line.to_string(),
            });
        }
    }
    None
}

fn match_spell(line: &str) -> Option<DamageCandidate> {
    let captures = SPELL_REGEX.captures(line)?;
    let spell_name = captures.get(1)?.as_str().to_string();
    Some(DamageCandidate {
        category: DamageCategory::Spell,
        source_name: String::new(),
        message_verb: spell_name,
        message_text: line.to_string(),
    })
}

pub fn format_incoming_line(source: &str, canonical_verb: &str) -> String {
    format!("{source} {} you.", conjugate_verb(canonical_verb))
}

pub fn format_incoming_line_bare(source: &str, canonical_verb: &str) -> String {
    format!("{source} {canonical_verb} you.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_damage::catalog::{CATALOG, FAMILY_IDS};

    fn assert_match(
        matcher: &mut Matcher,
        line: &str,
        category: DamageCategory,
        source: &str,
        verb: &str,
    ) {
        let candidate = matcher
            .match_incoming(line)
            .unwrap_or_else(|| panic!("expected match for {line:?}"));
        assert_eq!(candidate.category, category, "line: {line}");
        assert_eq!(candidate.source_name, source, "line: {line}");
        assert_eq!(candidate.message_verb, verb, "line: {line}");
        assert_eq!(candidate.message_text, line);
    }

    fn assert_no_match(matcher: &mut Matcher, line: &str) {
        assert!(
            matcher.match_incoming(line).is_none(),
            "expected no match for {line:?}"
        );
    }

    #[test]
    fn catalog_has_eleven_families_and_two_hundred_eighty_six_verbs() {
        assert_eq!(FAMILY_IDS.len(), 11);
        assert_eq!(CATALOG.len(), 286);
    }

    #[test]
    fn catalog_families_are_unique_and_ordered() {
        assert_eq!(
            FAMILY_IDS,
            &[
                "slash", "bash", "pierce", "shield", "whip", "unarmed", "tiger", "monk", "bite",
                "claw", "breath"
            ]
        );
    }

    #[test]
    fn every_catalog_verb_matches_synthetic_incoming_line() {
        for entry in CATALOG {
            let mut matcher = Matcher::new();
            let conjugated = format_incoming_line("Orc", entry.canonical);
            let bare = format_incoming_line_bare("Orc", entry.canonical);
            let candidate = matcher
                .match_melee_for_test(&conjugated)
                .or_else(|| {
                    let mut matcher = Matcher::new();
                    matcher.match_melee_for_test(&bare)
                })
                .unwrap_or_else(|| panic!("no melee match for verb {}", entry.canonical));
            assert_eq!(candidate.category, DamageCategory::Melee);
            assert_eq!(candidate.source_name, "Orc");
            assert!(
                candidate.message_verb.eq_ignore_ascii_case(entry.canonical),
                "verb {:?} matched as {:?}",
                entry.canonical,
                candidate.message_verb
            );
        }
    }

    #[test]
    fn one_incoming_line_per_weapon_family() {
        let samples = [
            ("slash", "gash"),
            ("bash", "bash"),
            ("pierce", "puncture"),
            ("shield", "heavily bash"),
            ("whip", "lash"),
            ("unarmed", "bitchslap"),
            ("tiger", "toe-kick"),
            ("monk", "snap-kick"),
            ("bite", "bite"),
            ("claw", "claw"),
            ("breath", "breath lightly"),
        ];
        for (family, verb) in samples {
            let mut matcher = Matcher::new();
            let line = format_incoming_line("Orc", verb);
            let candidate = matcher
                .match_incoming(&line)
                .or_else(|| matcher.match_incoming(&format_incoming_line_bare("Orc", verb)))
                .unwrap();
            assert_eq!(candidate.message_verb, verb);
            let entry = CATALOG
                .iter()
                .find(|entry| entry.canonical == verb)
                .unwrap_or_else(|| panic!("verb {verb} missing from catalog"));
            assert_eq!(FAMILY_IDS[entry.family], family);
        }
    }

    #[test]
    fn conjugation_edge_cases_match_incoming() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Holy man bitchslaps you.",
            DamageCategory::Melee,
            "Holy man",
            "bitchslap",
        );
        assert_match(
            &mut matcher,
            "Holy man lightly strikes you.",
            DamageCategory::Melee,
            "Holy man",
            "lightly strike",
        );
        assert_match(
            &mut matcher,
            "Holy man gashes you.",
            DamageCategory::Melee,
            "Holy man",
            "gash",
        );
        assert_match(
            &mut matcher,
            "Dragon breath lightly you.",
            DamageCategory::Melee,
            "Dragon",
            "breath lightly",
        );
        assert_match(
            &mut matcher,
            "Dragon BRUTALLY TEARS you.",
            DamageCategory::Melee,
            "Dragon",
            "BRUTALLY TEAR",
        );
    }

    #[test]
    fn example_fight_incoming_melee_lines() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Holy man bitchslaps you.",
            DamageCategory::Melee,
            "Holy man",
            "bitchslap",
        );
        assert_match(
            &mut matcher,
            "Holy man lightly strikes you.",
            DamageCategory::Melee,
            "Holy man",
            "lightly strike",
        );
        assert_match(
            &mut matcher,
            "Holy man boots you.",
            DamageCategory::Melee,
            "Holy man",
            "boot",
        );
    }

    #[test]
    fn example_fight_incoming_skill_lines() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Holy man's bash sends you sprawling.",
            DamageCategory::Skill,
            "Holy man",
            "bash",
        );
        assert_match(
            &mut matcher,
            "Holy man pushes you.",
            DamageCategory::Skill,
            "Holy man",
            "push",
        );
    }

    #[test]
    fn example_fight_non_damage_lines_do_not_match() {
        let mut matcher = Matcher::new();
        for line in [
            "Holy man misses.",
            "You miss.",
            "You puncture Holy man.",
            "You tumble Holy man's dodge.",
            "Holy man dodges.",
            "You poke Holy man in the ribs with two fingers going in.",
            "Holy man gasps painfully for air.",
            "You are prepared to do the skill.",
            "********************** Round 1 **********************",
            "H:760/782 [-22] S:434/474 [-40] E:489/494 [] $:2510 [] exp:170412 []",
        ] {
            assert_no_match(&mut matcher, line);
        }
    }

    #[test]
    fn kick_skill_examples() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Salvatore kicks you in the groin very hard. You gasp with pain and double up.",
            DamageCategory::Skill,
            "Salvatore",
            "kick",
        );
        assert_match(
            &mut matcher,
            "Salvatore performs a quick kick to your stomach, almost making you lose your breakfast.",
            DamageCategory::Skill,
            "Salvatore",
            "kick",
        );
        assert_match(
            &mut matcher,
            "Salvatore's kick lashes at you with speed, but you manage to partly deflect it in time.",
            DamageCategory::Skill,
            "Salvatore",
            "kick",
        );
    }

    #[test]
    fn stab_skill_examples() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "With a quick flick, Akeem knocks your weapon aside and stabs your stomach!",
            DamageCategory::Skill,
            "Akeem",
            "stab",
        );
        assert_match(
            &mut matcher,
            "You watch helplessly as Akeem smashes your kneecap!",
            DamageCategory::Skill,
            "Akeem",
            "stab",
        );
        assert_match(
            &mut matcher,
            "OOF!  Akeem feints, throwing you offguard as he PUMMELS your midriff!",
            DamageCategory::Skill,
            "Akeem",
            "stab",
        );
    }

    #[test]
    fn scythe_swipe_skill_example() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Reaver slashes a ragged wound across your chest.",
            DamageCategory::Skill,
            "Reaver",
            "scythe swipe",
        );
    }

    #[test]
    fn spell_examples_from_fixture() {
        let mut matcher = Matcher::new();
        for (line, spell) in [
            ("A magic missile hits you.", "magic missile"),
            ("A chill touch hits you.", "chill touch"),
            ("A firebolt hits you.", "firebolt"),
            ("A suffocation hits you.", "suffocation"),
            ("A thorn spray hits you.", "thorn spray"),
            ("A chaos bolt hits you.", "chaos bolt"),
            ("A venom strike hits you.", "venom strike"),
            ("A psi blast hits you.", "psi blast"),
            ("A fire blast hits you.", "fire blast"),
            ("A blast lightning hits you.", "blast lightning"),
            ("A shocking grasp hits you.", "shocking grasp"),
            ("An icebolt hits you.", "icebolt"),
        ] {
            assert_match(&mut matcher, line, DamageCategory::Spell, "", spell);
        }
    }

    #[test]
    fn skill_bash_wins_over_melee_ambiguity() {
        let mut matcher = Matcher::new();
        let candidate = matcher
            .match_incoming("Holy man's bash sends you sprawling.")
            .unwrap();
        assert_eq!(candidate.category, DamageCategory::Skill);
        assert_eq!(candidate.message_verb, "bash");
    }

    #[test]
    fn skill_push_wins_over_monk_melee_push() {
        let mut matcher = Matcher::new();
        let candidate = matcher.match_incoming("Holy man pushes you.").unwrap();
        assert_eq!(candidate.category, DamageCategory::Skill);
        assert_eq!(candidate.message_verb, "push");
    }

    #[test]
    fn stab_skill_beats_outgoing_you_prefix() {
        let mut matcher = Matcher::new();
        let candidate = matcher
            .match_incoming("You watch helplessly as Akeem smashes your kneecap!")
            .unwrap();
        assert_eq!(candidate.category, DamageCategory::Skill);
    }

    #[test]
    fn family_recency_prefers_last_family() {
        let mut matcher = Matcher::new();
        matcher.match_incoming("Holy man bitchslaps you.").unwrap();
        assert_eq!(matcher.last_family, Some(5)); // unarmed

        let line = format_incoming_line("Holy man", "boot");
        matcher.match_incoming(&line).unwrap();
        assert_eq!(matcher.last_family, Some(5));
    }

    #[test]
    fn reset_clears_family_recency() {
        let mut matcher = Matcher::new();
        matcher.match_incoming("Holy man bitchslaps you.").unwrap();
        matcher.reset();
        assert_eq!(matcher.last_family, None);
    }

    #[test]
    fn outgoing_sanity_checks_from_example_fight() {
        let matcher = Matcher::new();
        let (verb, target) = matcher
            .match_outgoing_sanity("You puncture Holy man.")
            .unwrap();
        assert_eq!(verb, "puncture");
        assert_eq!(target, "Holy man");

        let (verb, target) = matcher
            .match_outgoing_sanity("You lightly cut Holy man.")
            .unwrap();
        assert_eq!(verb, "lightly cut");
        assert_eq!(target, "Holy man");
    }

    #[test]
    fn outgoing_lines_are_not_incoming_candidates() {
        let mut matcher = Matcher::new();
        assert_no_match(&mut matcher, "You puncture Holy man.");
        assert_no_match(&mut matcher, "You pierce Holy man.");
        assert_no_match(&mut matcher, "You cut Holy man.");
    }

    #[test]
    fn longest_verb_wins_within_family() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Orc lightly strikes you.",
            DamageCategory::Melee,
            "Orc",
            "lightly strike",
        );
        assert_match(
            &mut matcher,
            "Orc savagely triple-kicks you.",
            DamageCategory::Melee,
            "Orc",
            "savagely triple-kick",
        );
    }

    #[test]
    fn case_insensitive_incoming_match() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "holy man BITCHSLAPS you.",
            DamageCategory::Melee,
            "holy man",
            "bitchslap",
        );
    }

    #[test]
    fn monk_push_catalog_verb_matches_melee_when_bare() {
        let mut matcher = Matcher::new();
        let candidate = matcher
            .match_melee_for_test("Orc push you.")
            .expect("monk push melee verb");
        assert_eq!(candidate.message_verb, "push");
        assert_eq!(candidate.category, DamageCategory::Melee);
    }

    #[test]
    fn monk_push_conjugated_line_is_skill_not_melee() {
        let mut matcher = Matcher::new();
        let candidate = matcher.match_incoming("Holy man pushes you.").unwrap();
        assert_eq!(candidate.category, DamageCategory::Skill);
        assert_eq!(candidate.message_verb, "push");
    }

    #[test]
    fn each_family_has_twenty_six_verbs() {
        let mut counts = [0usize; 11];
        for entry in CATALOG {
            counts[entry.family] += 1;
        }
        for (family, count) in FAMILY_IDS.iter().zip(counts) {
            assert_eq!(count, 26, "family {family} verb count");
        }
    }

    #[test]
    fn outgoing_sanity_for_every_catalog_verb() {
        let matcher = Matcher::new();
        for entry in CATALOG {
            let line = format!("You {} Holy man.", entry.canonical);
            let (verb, target) = matcher
                .match_outgoing_sanity(&line)
                .unwrap_or_else(|| panic!("outgoing sanity failed for {}", entry.canonical));
            assert_eq!(verb, entry.canonical);
            assert_eq!(target, "Holy man");
        }
    }

    #[test]
    fn all_caps_catalog_verbs_match_incoming() {
        let mut matcher = Matcher::new();
        for verb in [
            "BRUTALLY TEAR",
            "SAVAGELY SHRED",
            "REALLY WHACK",
            "BARBARICALLY BASH",
            "DEVASTATINGLY HEAD-THROW",
        ] {
            let line = format_incoming_line("Dragon", verb);
            let candidate = matcher.match_melee_for_test(&line).unwrap();
            assert_eq!(candidate.message_verb, verb);
        }
    }

    #[test]
    fn comma_phrase_verbs_match_bare_incoming() {
        let mut matcher = Matcher::new();
        for (verb, line) in [
            (
                "evade, and then reverse",
                "Monk evade, and then reverse you.",
            ),
            (
                "pummel, with dozens of chain punches,",
                "Monk pummel, with dozens of chain punches, you.",
            ),
        ] {
            let candidate = matcher.match_melee_for_test(line).unwrap();
            assert_eq!(candidate.message_verb, verb);
        }
    }

    #[test]
    fn spell_cast_lines_do_not_match() {
        let mut matcher = Matcher::new();
        for line in [
            "Ghost hound utters the magic words 'ghht zur sanc'",
            "Good pixie claps his hands and says 'cah zur fehh'",
            "Knight fills up his cheeks with air and exhales 'zot zur fehh'",
        ] {
            assert_no_match(&mut matcher, line);
        }
    }

    #[test]
    fn longest_match_beats_embedded_shorter_verb() {
        let mut matcher = Matcher::new();
        assert_match(
            &mut matcher,
            "Orc heavily bashes you.",
            DamageCategory::Melee,
            "Orc",
            "heavily bash",
        );
        assert_match(
            &mut matcher,
            "Orc BARBARICALLY BASHES you.",
            DamageCategory::Melee,
            "Orc",
            "BARBARICALLY BASH",
        );
    }
}
