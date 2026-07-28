use crate::triggers::rule_engine::{
    HiliteTarget, Rule, RuleAction, RuleMatcher, push_rule, sort_rules, tf_hilite,
};
use log::warn;
use regex::Regex;
use std::sync::{Arc, Mutex};

static PLAYER_COMBAT_RULES_CACHE: Mutex<Option<(String, Arc<Vec<Rule>>)>> = Mutex::new(None);
// ponytail: global mutex + single-entry cache; per-name LRU if contention shows up

pub(crate) fn player_combat_rules_arc(name: &str) -> Arc<Vec<Rule>> {
    let Some(name) = player_combat_rule_name(name) else {
        return Arc::new(Vec::new());
    };

    let mut guard = PLAYER_COMBAT_RULES_CACHE.lock().unwrap_or_else(|poisoned| {
        warn!("player combat rules cache mutex was poisoned; clearing cache");
        let mut guard = poisoned.into_inner();
        *guard = None;
        guard
    });
    if guard
        .as_ref()
        .is_some_and(|(stored, _)| stored.as_str() == name)
    {
        return Arc::clone(&guard.as_ref().unwrap().1);
    }
    let built = Arc::new(build_player_combat_rules(&name));
    *guard = Some((name, Arc::clone(&built)));
    built
}

fn player_combat_rule_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let mut chars = trimmed.chars();
    let first = chars.next()?;

    let mut normalized = first.to_uppercase().collect::<String>();
    normalized.push_str(&chars.as_str().to_lowercase());
    Some(normalized)
}

fn push_player_combat_regex_rule(
    rules: &mut Vec<Rule>,
    order: &mut usize,
    pattern: String,
    actions: Vec<RuleAction>,
) {
    let Some(regex) = Regex::new(&pattern).ok() else {
        warn!("failed to compile player combat trigger regex; pattern={pattern:?}");
        return;
    };
    push_rule(rules, order, RuleMatcher::Regex(regex), 1000, None, actions);
}

/// Player avatar combat hit lines keyed by configured player name.
pub(crate) fn build_player_combat_rules(name: &str) -> Vec<Rule> {
    let escaped = regex::escape(name);
    let mut rules = Vec::new();
    let mut order = 0usize;

    // "{name} hits <other> …" — attacker is the player character (green), count is group 2.
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(
            r"^{} hits (.+) (?:once|twice|thrice|\d+ times) (.+)\.$",
            escaped
        ),
        vec![tf_hilite("Cgreen", HiliteTarget::Whole)],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^{} hits (.+) (once) (.+)\.$", escaped),
        vec![tf_hilite("Cblue", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^{} hits (.+) (twice) (.+)\.$", escaped),
        vec![tf_hilite("Cmagenta", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^{} hits (.+) (thrice) (.+)\.$", escaped),
        vec![tf_hilite("BCred", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^{} hits (.+) (\d+ times) (.+)\.$", escaped),
        vec![tf_hilite("Cred", HiliteTarget::Group(2))],
    );

    // "<other> hits {name} …" — player is the target (magenta), count is group 2.
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(
            r"^(.+) hits {} (?:once|twice|thrice|\d+ times) (.+)\.$",
            escaped
        ),
        vec![tf_hilite("Cmagenta", HiliteTarget::Whole)],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^(.+) hits {} (once) (.+)\.$", escaped),
        vec![tf_hilite("Cblue", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^(.+) hits {} (twice) (.+)\.$", escaped),
        vec![tf_hilite("BCmagenta", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^(.+) hits {} (thrice) (.+)\.$", escaped),
        vec![tf_hilite("BCred", HiliteTarget::Group(2))],
    );
    push_player_combat_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^(.+) hits {} (\d+ times) (.+)\.$", escaped),
        vec![tf_hilite("Cred", HiliteTarget::Group(2))],
    );

    sort_rules(&mut rules);
    rules
}
