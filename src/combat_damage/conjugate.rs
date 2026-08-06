pub fn conjugate_last_word(word: &str) -> String {
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

pub fn conjugate_verb(verb: &str) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conjugate_single_word_adds_s() {
        assert_eq!(conjugate_verb("boot"), "boots");
        assert_eq!(conjugate_verb("bitchslap"), "bitchslaps");
    }

    #[test]
    fn conjugate_single_word_adds_es() {
        assert_eq!(conjugate_verb("gash"), "gashes");
        assert_eq!(conjugate_verb("scratch"), "scratches");
    }

    #[test]
    fn conjugate_multi_word_conjugates_last_word_only() {
        assert_eq!(conjugate_verb("lightly strike"), "lightly strikes");
        assert_eq!(conjugate_verb("cruelly beat"), "cruelly beats");
    }

    #[test]
    fn conjugate_preserves_caps_on_last_word() {
        assert_eq!(conjugate_verb("BRUTALLY TEAR"), "BRUTALLY TEARS");
    }
}
