use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR");
    let hit_messages = Path::new(&manifest_dir).join("docs/hit_messages.md");
    println!("cargo:rerun-if-changed={}", hit_messages.display());

    let content = fs::read_to_string(&hit_messages)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", hit_messages.display()));

    let families = parse_hit_messages(&content);
    let out_dir = env::var("OUT_DIR").expect("OUT_DIR");
    let out_path = Path::new(&out_dir).join("combat_damage_catalog.rs");
    fs::write(&out_path, generate_catalog_rs(&families))
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out_path.display()));
}

#[derive(Clone, Debug)]
struct Family {
    id: String,
    title: String,
    verbs: Vec<String>,
}

fn parse_hit_messages(content: &str) -> Vec<Family> {
    let mut families = Vec::new();
    let mut current: Option<Family> = None;

    for line in content.lines() {
        if let Some((title, family_id)) = parse_family_header(line) {
            if let Some(family) = current.take() {
                families.push(family);
            }
            current = Some(Family {
                id: family_id,
                title,
                verbs: Vec::new(),
            });
            continue;
        }

        if let Some(verb) = parse_verb_line(line)
            && let Some(family) = current.as_mut()
        {
            family.verbs.push(verb);
        }
    }

    if let Some(family) = current {
        families.push(family);
    }

    if families.is_empty() {
        panic!("no weapon families parsed from hit_messages.md");
    }

    for family in &families {
        if family.verbs.is_empty() {
            panic!("family '{}' has no verbs", family.id);
        }
        if family.verbs.len() != 26 {
            panic!(
                "family '{}' expected 26 verbs, got {}",
                family.id,
                family.verbs.len()
            );
        }
    }

    families
}

fn parse_family_header(line: &str) -> Option<(String, String)> {
    let line = line.trim();
    if !line.starts_with('#') {
        return None;
    }
    let open = line.rfind('(')?;
    let close = line.rfind(')')?;
    if close <= open {
        return None;
    }
    let id = line[open + 1..close].trim().to_string();
    let for_marker = " for ";
    let title_start = line.find(for_marker)? + for_marker.len();
    let title = capitalize_title(line[title_start..open].trim());
    Some((title, id))
}

fn capitalize_title(title: &str) -> String {
    let mut chars = title.chars();
    match chars.next() {
        None => String::new(),
        Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
    }
}

fn parse_verb_line(line: &str) -> Option<String> {
    let trimmed = line.trim();
    let (num, verb) = trimmed.split_once(':')?;
    if num.trim().parse::<u32>().is_err() {
        return None;
    }
    let verb = verb.trim();
    if verb.is_empty() {
        return None;
    }
    Some(verb.to_string())
}

fn conjugate_last_word(word: &str) -> String {
    let lower = word.to_lowercase();
    let suffix = if lower.ends_with('s')
        || lower.ends_with('x')
        || lower.ends_with('z')
        || lower.ends_with("ch")
        || lower.ends_with("sh")
    {
        "es"
    } else {
        "s"
    };

    if word.bytes().all(|b| b.is_ascii_uppercase() || b == b'-')
        && word.contains(|c: char| c.is_ascii_alphabetic())
    {
        format!("{word}{}", suffix.to_ascii_uppercase())
    } else {
        format!("{word}{suffix}")
    }
}

fn conjugate_verb(verb: &str) -> String {
    let parts: Vec<&str> = verb.split_whitespace().collect();
    if parts.is_empty() {
        return String::new();
    }
    if parts.len() == 1 {
        return conjugate_last_word(parts[0]);
    }
    let mut out: Vec<String> = parts[..parts.len() - 1]
        .iter()
        .map(|part| (*part).to_string())
        .collect();
    out.push(conjugate_last_word(parts[parts.len() - 1]));
    out.join(" ")
}

fn escape_rs(s: &str) -> String {
    s.replace('\\', "\\\\").replace('\"', "\\\"")
}

fn generate_catalog_rs(families: &[Family]) -> String {
    let mut entries: Vec<(usize, usize, String, String, String, u8)> = Vec::new();

    for (family_idx, family) in families.iter().enumerate() {
        for (rank, verb) in family.verbs.iter().enumerate() {
            let rank = u8::try_from(rank + 1).expect("catalog rank fits u8");
            let conjugated = conjugate_verb(verb);
            let conjugated_suffix = format!("{conjugated} you.");
            let bare_suffix = format!("{verb} you.");
            entries.push((
                verb.len(),
                family_idx,
                verb.clone(),
                conjugated_suffix,
                bare_suffix,
                rank,
            ));
        }
    }

    entries.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.2.cmp(&b.2)));

    let family_ids: Vec<String> = families.iter().map(|f| f.id.clone()).collect();
    let family_id_literals = family_ids
        .iter()
        .map(|id| format!("\"{}\"", escape_rs(id)))
        .collect::<Vec<_>>()
        .join(", ");
    let family_title_literals = families
        .iter()
        .map(|family| format!("\"{}\"", escape_rs(&family.title)))
        .collect::<Vec<_>>()
        .join(", ");

    let mut family_indices: Vec<Vec<usize>> = vec![Vec::new(); families.len()];
    let mut entry_literals = String::new();

    for (idx, (_, family_idx, verb, conjugated_suffix, bare_suffix, rank)) in
        entries.iter().enumerate()
    {
        family_indices[*family_idx].push(idx);
        entry_literals.push_str(&format!(
            "    CatalogEntry {{ canonical: \"{}\", family: {}, rank: {rank}, conjugated_suffix: \"{}\", bare_suffix: \"{}\" }},\n",
            escape_rs(verb),
            family_idx,
            escape_rs(conjugated_suffix),
            escape_rs(bare_suffix),
        ));
    }

    format!(
        r"// @generated by build.rs from docs/hit_messages.md — do not edit

pub struct CatalogEntry {{
    pub canonical: &'static str,
    pub family: usize,
    pub rank: u8,
    pub conjugated_suffix: &'static str,
    pub bare_suffix: &'static str,
}}

pub const FAMILY_IDS: &[&str] = &[{family_id_literals}];

pub const FAMILY_TITLES: &[&str] = &[{family_title_literals}];

pub const CATALOG: &[CatalogEntry] = &[
{entry_literals}];
"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conjugate_examples() {
        assert_eq!(conjugate_verb("bitchslap"), "bitchslaps");
        assert_eq!(conjugate_verb("lightly strike"), "lightly strikes");
        assert_eq!(conjugate_verb("gash"), "gashes");
        assert_eq!(conjugate_verb("breath lightly"), "breath lightlys");
    }

    #[test]
    fn parse_hit_messages_has_eleven_families() {
        let content = include_str!("docs/hit_messages.md");
        let families = parse_hit_messages(content);
        assert_eq!(families.len(), 11);
        assert_eq!(families[0].id, "slash");
        assert_eq!(families[10].id, "breath");
    }
}
