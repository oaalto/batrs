use crate::triggers::rule_engine::{
    HiliteTarget, Rule, RuleAction, RuleMatcher, push_rule, sort_rules, tf_hilite,
};
use log::warn;
use regex::Regex;
use std::sync::{Arc, Mutex};

static COMPANION_RULES_CACHE: Mutex<Option<(String, Arc<Vec<Rule>>)>> = Mutex::new(None);
// ponytail: global mutex + single-entry cache; per-name LRU if contention shows up

pub(crate) fn companion_rules_arc(name: &str) -> Arc<Vec<Rule>> {
    let Some(name) = companion_rule_name(name) else {
        return Arc::new(Vec::new());
    };

    let mut guard = COMPANION_RULES_CACHE.lock().unwrap_or_else(|poisoned| {
        warn!("companion rules cache mutex was poisoned; clearing cache");
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
    let built = Arc::new(build_companion_rules(&name));
    *guard = Some((name, Arc::clone(&built)));
    built
}

fn companion_rule_name(name: &str) -> Option<String> {
    let trimmed = name.trim();
    let mut chars = trimmed.chars();
    let first = chars.next()?;

    let mut normalized = first.to_uppercase().collect::<String>();
    normalized.push_str(&chars.as_str().to_lowercase());
    Some(normalized)
}

fn push_companion_regex_rule(
    rules: &mut Vec<Rule>,
    order: &mut usize,
    pattern: String,
    actions: Vec<RuleAction>,
) {
    let Some(regex) = Regex::new(&pattern).ok() else {
        warn!("failed to compile companion trigger regex; pattern={pattern:?}");
        return;
    };
    push_rule(rules, order, RuleMatcher::Regex(regex), 1000, None, actions);
}

/// Animist soul-companion announcement lines keyed by configured player name.
pub(crate) fn build_companion_rules(name: &str) -> Vec<Rule> {
    let escaped = regex::escape(name);
    let mut rules = Vec::new();
    let mut order = 0usize;

    push_companion_regex_rule(
        &mut rules,
        &mut order,
        format!(r"^A blue-glowing soul companion \[{}\]\.?$", escaped),
        vec![tf_hilite("Cblue", HiliteTarget::Whole)],
    );

    sort_rules(&mut rules);
    rules
}
