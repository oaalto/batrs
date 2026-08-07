use crate::combat_damage::attribution::{catalog_weights, confidence};
use crate::combat_damage::matcher::{DamageCandidate, DamageCategory, Matcher};
use crate::combat_damage::storage::open_db;
use crate::triggers::SC_REGEX;
use chrono::Utc;
use log::warn;
use rusqlite::Connection;
use std::path::Path;

struct PendingBatch {
    batch_id: i64,
    recorded_at: String,
    player: String,
    hp_delta: i32,
    hp_before: i32,
    hp_after: i32,
    candidate_count: usize,
    damage_min: i32,
    damage_max: i32,
    candidates: Vec<DamageCandidate>,
}

struct PendingUnattributed {
    recorded_at: String,
    player: String,
    hp_delta: i32,
    hp_before: i32,
    hp_after: i32,
    h_line_text: String,
    context_lines: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Clone, PartialEq)]
pub struct DamageEventRow {
    pub batch_id: i64,
    pub recorded_at: String,
    pub player: String,
    pub hp_delta: i32,
    pub hp_before: i32,
    pub hp_after: i32,
    pub damage_category: String,
    pub source_name: String,
    pub message_verb: String,
    pub message_text: String,
    pub candidate_count: i32,
    pub confidence: f64,
    pub weight: f64,
    pub damage_min: i32,
    pub damage_max: i32,
    pub catalog_rank: Option<i32>,
    pub weapon_family: Option<String>,
}

pub struct DamageCollector {
    matcher: Matcher,
    buffer: Vec<DamageCandidate>,
    context_window: Vec<String>,
    conn: Option<Connection>,
    next_batch_id: i64,
}

impl DamageCollector {
    pub fn open(path: &Path) -> Result<Self, String> {
        let conn = open_db(path)?;
        Ok(Self::new(conn))
    }

    pub fn new(conn: Connection) -> Self {
        let next_batch_id = conn
            .query_row(
                "SELECT COALESCE(MAX(batch_id), 0) + 1 FROM damage_events",
                [],
                |row| row.get(0),
            )
            .unwrap_or(1);
        Self {
            matcher: Matcher::new(),
            buffer: Vec::new(),
            context_window: Vec::new(),
            conn: Some(conn),
            next_batch_id,
        }
    }

    pub fn inert() -> Self {
        Self {
            matcher: Matcher::new(),
            buffer: Vec::new(),
            context_window: Vec::new(),
            conn: None,
            next_batch_id: 1,
        }
    }

    pub fn reset_buffer(&mut self) {
        self.buffer.clear();
        self.context_window.clear();
    }

    #[cfg(test)]
    pub fn buffer_len(&self) -> usize {
        self.buffer.len()
    }

    #[cfg(test)]
    pub fn context_window_len(&self) -> usize {
        self.context_window.len()
    }

    pub fn handle_line(&mut self, line: &str, player: &str) {
        if let Some(captures) = SC_REGEX.captures(line) {
            self.flush_on_h_line(&captures, player, line);
            return;
        }
        self.context_window.push(line.to_string());
        if let Some(candidate) = self.matcher.match_incoming(line) {
            self.buffer.push(candidate);
        }
    }

    fn flush_on_h_line(&mut self, captures: &regex::Captures<'_>, player: &str, h_line_text: &str) {
        let hp_current = captures
            .get(1)
            .and_then(|value| value.as_str().parse::<i32>().ok())
            .unwrap_or_default();
        let diff_hp = captures
            .get(3)
            .and_then(|value| value.as_str().parse::<i32>().ok())
            .unwrap_or_default();
        let candidates: Vec<DamageCandidate> = self.buffer.drain(..).collect();
        let context_lines: Vec<String> = self.context_window.drain(..).collect();

        if diff_hp >= 0 {
            return;
        }

        let hp_delta = -diff_hp;
        let hp_after = hp_current;
        let hp_before = hp_current - diff_hp;
        let recorded_at = Utc::now().to_rfc3339();

        if candidates.is_empty() {
            if let Err(err) = self.write_unattributed(PendingUnattributed {
                recorded_at,
                player: player.to_string(),
                hp_delta,
                hp_before,
                hp_after,
                h_line_text: h_line_text.to_string(),
                context_lines,
            }) {
                warn!("failed to write unattributed hp event: {err}");
            }
            return;
        }

        let candidate_count = candidates.len();
        let (damage_min, damage_max) = if candidate_count == 1 {
            (hp_delta, hp_delta)
        } else {
            (0, hp_delta)
        };
        let batch_id = self.next_batch_id;
        self.next_batch_id += 1;

        if let Err(err) = self.write_batch(PendingBatch {
            batch_id,
            recorded_at,
            player: player.to_string(),
            hp_delta,
            hp_before,
            hp_after,
            candidate_count,
            damage_min,
            damage_max,
            candidates,
        }) {
            warn!("failed to write combat damage batch: {err}");
        }
    }

    fn write_unattributed(&mut self, event: PendingUnattributed) -> Result<(), String> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| "combat damage database unavailable".to_string())?;
        let transaction = conn.transaction().map_err(|err| err.to_string())?;
        let context_json = json_string_array(&event.context_lines);
        transaction
            .execute(
                "
                INSERT INTO unattributed_hp_events (
                    recorded_at, player, hp_delta, hp_before, hp_after,
                    h_line_text, context_lines
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                ",
                rusqlite::params![
                    event.recorded_at,
                    event.player,
                    event.hp_delta,
                    event.hp_before,
                    event.hp_after,
                    event.h_line_text,
                    context_json,
                ],
            )
            .map_err(|err| err.to_string())?;
        transaction.commit().map_err(|err| err.to_string())?;
        Ok(())
    }

    fn write_batch(&mut self, batch: PendingBatch) -> Result<(), String> {
        let conn = self
            .conn
            .as_mut()
            .ok_or_else(|| "combat damage database unavailable".to_string())?;
        let transaction = conn.transaction().map_err(|err| err.to_string())?;
        let batch_confidence = confidence(batch.candidate_count);
        let ranks: Vec<Option<i32>> = batch
            .candidates
            .iter()
            .map(|candidate| candidate.catalog_rank.map(i32::from))
            .collect();
        let weights = catalog_weights(&ranks);
        for (candidate, weight) in batch.candidates.iter().zip(weights) {
            transaction
                .execute(
                    "
                    INSERT INTO damage_events (
                        batch_id, recorded_at, player, hp_delta, hp_before, hp_after,
                        damage_category, source_name, message_verb, message_text,
                        candidate_count, confidence, damage_min, damage_max,
                        catalog_rank, weapon_family, weight
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16, ?17)
                    ",
                    rusqlite::params![
                        batch.batch_id,
                        batch.recorded_at,
                        batch.player,
                        batch.hp_delta,
                        batch.hp_before,
                        batch.hp_after,
                        category_label(candidate.category),
                        candidate.source_name,
                        candidate.message_verb,
                        candidate.message_text,
                        batch.candidate_count as i32,
                        batch_confidence,
                        batch.damage_min,
                        batch.damage_max,
                        candidate.catalog_rank.map(i32::from),
                        candidate.weapon_family,
                        weight,
                    ],
                )
                .map_err(|err| err.to_string())?;
        }
        transaction.commit().map_err(|err| err.to_string())?;
        Ok(())
    }

    #[cfg(test)]
    pub fn query_all_events(conn: &Connection) -> Result<Vec<DamageEventRow>, String> {
        let mut statement = conn
            .prepare(
                "
                SELECT batch_id, recorded_at, player, hp_delta, hp_before, hp_after,
                       damage_category, source_name, message_verb, message_text,
                       candidate_count, confidence, damage_min, damage_max,
                       catalog_rank, weapon_family, weight
                FROM damage_events
                ORDER BY id
                ",
            )
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| {
                Ok(DamageEventRow {
                    batch_id: row.get(0)?,
                    recorded_at: row.get(1)?,
                    player: row.get(2)?,
                    hp_delta: row.get(3)?,
                    hp_before: row.get(4)?,
                    hp_after: row.get(5)?,
                    damage_category: row.get(6)?,
                    source_name: row.get(7)?,
                    message_verb: row.get(8)?,
                    message_text: row.get(9)?,
                    candidate_count: row.get(10)?,
                    confidence: row.get(11)?,
                    damage_min: row.get(12)?,
                    damage_max: row.get(13)?,
                    catalog_rank: row.get(14)?,
                    weapon_family: row.get(15)?,
                    weight: row.get(16)?,
                })
            })
            .map_err(|err| err.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())
    }

    #[cfg(test)]
    pub fn query_all_unattributed(
        conn: &Connection,
    ) -> Result<Vec<UnattributedHpEventRow>, String> {
        let mut statement = conn
            .prepare(
                "
                SELECT recorded_at, player, hp_delta, hp_before, hp_after,
                       h_line_text, context_lines
                FROM unattributed_hp_events
                ORDER BY id
                ",
            )
            .map_err(|err| err.to_string())?;
        let rows = statement
            .query_map([], |row| {
                let context_json: String = row.get(6)?;
                Ok(UnattributedHpEventRow {
                    recorded_at: row.get(0)?,
                    player: row.get(1)?,
                    hp_delta: row.get(2)?,
                    hp_before: row.get(3)?,
                    hp_after: row.get(4)?,
                    h_line_text: row.get(5)?,
                    context_lines: parse_json_string_array(&context_json),
                })
            })
            .map_err(|err| err.to_string())?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())
    }

    #[cfg(test)]
    pub fn connection(&self) -> Option<&Connection> {
        self.conn.as_ref()
    }
}

fn category_label(category: DamageCategory) -> &'static str {
    match category {
        DamageCategory::Melee => "melee",
        DamageCategory::Skill => "skill",
        DamageCategory::Spell => "spell",
    }
}

fn json_string_array(lines: &[String]) -> String {
    json_string_array_from_slice(lines)
}

pub fn json_string_array_from_slice(lines: &[String]) -> String {
    let mut out = String::from("[");
    for (index, line) in lines.iter().enumerate() {
        if index > 0 {
            out.push(',');
        }
        out.push('"');
        for ch in line.chars() {
            match ch {
                '"' => out.push_str("\\\""),
                '\\' => out.push_str("\\\\"),
                '\n' => out.push_str("\\n"),
                '\r' => out.push_str("\\r"),
                '\t' => out.push_str("\\t"),
                c if c.is_control() => {
                    use std::fmt::Write;
                    let _ = write!(out, "\\u{:04x}", c as u32);
                }
                c => out.push(c),
            }
        }
        out.push('"');
    }
    out.push(']');
    out
}

#[cfg(test)]
#[allow(dead_code)]
pub struct UnattributedHpEventRow {
    pub recorded_at: String,
    pub player: String,
    pub hp_delta: i32,
    pub hp_before: i32,
    pub hp_after: i32,
    pub h_line_text: String,
    pub context_lines: Vec<String>,
}

pub fn parse_json_string_array(json: &str) -> Vec<String> {
    let trimmed = json.trim();
    if trimmed == "[]" {
        return Vec::new();
    }
    let inner = trimmed
        .strip_prefix('[')
        .and_then(|value| value.strip_suffix(']'))
        .unwrap_or(trimmed)
        .trim();
    if inner.is_empty() {
        return Vec::new();
    }
    let mut lines = Vec::new();
    let mut current = String::new();
    let mut chars = inner.chars().peekable();
    while chars.peek().is_some() {
        match chars.next() {
            Some('"') => {
                while let Some(ch) = chars.next() {
                    match ch {
                        '"' => break,
                        '\\' => match chars.next() {
                            Some('n') => current.push('\n'),
                            Some('r') => current.push('\r'),
                            Some('t') => current.push('\t'),
                            Some('"') => current.push('"'),
                            Some('\\') => current.push('\\'),
                            Some('u') => {
                                let hex: String = chars.by_ref().take(4).collect();
                                if let Ok(code) = u32::from_str_radix(&hex, 16)
                                    && let Some(decoded) = char::from_u32(code)
                                {
                                    current.push(decoded);
                                }
                            }
                            Some(other) => current.push(other),
                            None => break,
                        },
                        other => current.push(other),
                    }
                }
                lines.push(current.clone());
                current.clear();
            }
            Some(',') => {}
            Some(_) => {}
            None => break,
        }
    }
    lines
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::combat_damage::matcher::DamageCategory;

    fn temp_db_path(name: &str) -> std::path::PathBuf {
        std::env::temp_dir().join(format!(
            "batrs-damage-collector-{}-{}",
            name,
            std::process::id()
        ))
    }

    fn collector_with_temp_db(name: &str) -> (DamageCollector, std::path::PathBuf) {
        let path = temp_db_path(name);
        let _ = std::fs::remove_file(&path);
        let collector = DamageCollector::open(&path).expect("open temp db");
        (collector, path)
    }

    fn h_line(current: i32, max: i32, bracket: &str) -> String {
        format!("H:{current}/{max} [{bracket}] S:100/100 [] E:100/100 [] $:100 [] exp:100 []")
    }

    fn assert_row(row: &DamageEventRow, expected: &DamageEventRow) {
        assert_eq!(row.player, expected.player);
        assert_eq!(row.hp_delta, expected.hp_delta);
        assert_eq!(row.hp_before, expected.hp_before);
        assert_eq!(row.hp_after, expected.hp_after);
        assert_eq!(row.damage_category, expected.damage_category);
        assert_eq!(row.source_name, expected.source_name);
        assert_eq!(row.message_verb, expected.message_verb);
        assert_eq!(row.message_text, expected.message_text);
        assert_eq!(row.candidate_count, expected.candidate_count);
        assert!((row.confidence - expected.confidence).abs() < 1e-9);
        assert!((row.weight - expected.weight).abs() < 1e-9);
        assert_eq!(row.damage_min, expected.damage_min);
        assert_eq!(row.damage_max, expected.damage_max);
        if let Some(expected_rank) = expected.catalog_rank {
            assert_eq!(row.catalog_rank, Some(expected_rank));
        }
        if let Some(expected_family) = &expected.weapon_family {
            assert_eq!(row.weapon_family.as_deref(), Some(expected_family.as_str()));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn expected_row(
        player: &str,
        hp_delta: i32,
        hp_before: i32,
        hp_after: i32,
        category: &str,
        source: &str,
        verb: &str,
        text: &str,
        candidate_count: i32,
        confidence: f64,
        weight: f64,
        damage_min: i32,
        damage_max: i32,
    ) -> DamageEventRow {
        DamageEventRow {
            batch_id: 0,
            recorded_at: String::new(),
            player: player.to_string(),
            hp_delta,
            hp_before,
            hp_after,
            damage_category: category.to_string(),
            source_name: source.to_string(),
            message_verb: verb.to_string(),
            message_text: text.to_string(),
            candidate_count,
            confidence,
            weight,
            damage_min,
            damage_max,
            catalog_rank: None,
            weapon_family: None,
        }
    }

    #[test]
    fn json_string_array_round_trips_context_lines() {
        let lines = vec![
            "Holy man misses.".to_string(),
            "You say \"hello\".".to_string(),
            "Path\\to\\file".to_string(),
            "line\twith\ttabs".to_string(),
        ];
        let json = json_string_array_from_slice(&lines);
        assert_eq!(parse_json_string_array(&json), lines);
    }

    #[test]
    fn isolated_melee_hit_writes_one_row() {
        let (mut collector, path) = collector_with_temp_db("isolated-melee");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_row(
            &rows[0],
            &expected_row(
                "Fueryon",
                22,
                782,
                760,
                "melee",
                "Holy man",
                "bitchslap",
                "Holy man bitchslaps you.",
                1,
                1.0,
                1.0,
                22,
                22,
            ),
        );
        assert_eq!(rows[0].catalog_rank, Some(4));
        assert_eq!(rows[0].weapon_family.as_deref(), Some("unarmed"));
        assert_eq!(collector.buffer_len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn isolated_skill_variants_write_expected_verbs() {
        let cases = [
            (
                "Holy man's bash sends you sprawling.",
                "bash",
                DamageCategory::Skill,
            ),
            ("Holy man pushes you.", "push", DamageCategory::Skill),
            (
                "Salvatore kicks you in the groin very hard. You gasp with pain and double up.",
                "kick",
                DamageCategory::Skill,
            ),
            (
                "With a quick flick, Akeem knocks your weapon aside and stabs your stomach!",
                "stab",
                DamageCategory::Skill,
            ),
            (
                "Reaver slashes a ragged wound across your chest.",
                "scythe swipe",
                DamageCategory::Skill,
            ),
        ];
        for (line, verb, category) in cases {
            let (mut collector, path) = collector_with_temp_db(verb);
            collector.handle_line(line, "Fueryon");
            collector.handle_line(&h_line(90, 100, "-10"), "Fueryon");
            let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
            assert_eq!(rows.len(), 1, "line: {line}");
            assert_eq!(rows[0].message_verb, verb);
            assert_eq!(rows[0].damage_category, category_label(category));
            let _ = std::fs::remove_file(path);
        }
    }

    #[test]
    fn isolated_spell_writes_spell_name_with_empty_source() {
        let (mut collector, path) = collector_with_temp_db("spell");
        collector.handle_line("A magic missile hits you.", "Fueryon");
        collector.handle_line(&h_line(50, 100, "-5"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].message_verb, "magic missile");
        assert_eq!(rows[0].source_name, "");
        assert_eq!(rows[0].damage_category, "spell");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ambiguous_two_candidates_share_batch_id_and_fractional_weight() {
        let (mut collector, path) = collector_with_temp_db("ambiguous-2");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line("Holy man lightly strikes you.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].batch_id, rows[1].batch_id);
        for row in &rows {
            assert_row(
                row,
                &expected_row(
                    "Fueryon",
                    22,
                    782,
                    760,
                    "melee",
                    "Holy man",
                    row.message_verb.as_str(),
                    row.message_text.as_str(),
                    2,
                    0.5,
                    row.weight,
                    0,
                    22,
                ),
            );
            assert!((row.confidence - 0.5).abs() < 1e-9);
        }
        let weights: Vec<f64> = rows.iter().map(|row| row.weight).collect();
        assert!(
            weights
                .iter()
                .any(|weight| (*weight - 4.0 / 9.0).abs() < 1e-9)
        );
        assert!(
            weights
                .iter()
                .any(|weight| (*weight - 5.0 / 9.0).abs() < 1e-9)
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ambiguous_three_candidates_use_one_third_weight() {
        let (mut collector, path) = collector_with_temp_db("ambiguous-3");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line("Holy man lightly strikes you.", "Fueryon");
        collector.handle_line("Holy man boots you.", "Fueryon");
        collector.handle_line(&h_line(700, 782, "-30"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 3);
        for row in &rows {
            assert!((row.confidence - (1.0 / 3.0)).abs() < 1e-9);
            assert_eq!(row.candidate_count, 3);
            assert_eq!(row.damage_min, 0);
            assert_eq!(row.damage_max, 30);
        }
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn negative_h_with_zero_candidates_writes_unattributed_row() {
        let (mut collector, path) = collector_with_temp_db("zero-candidates");
        collector.handle_line("Holy man misses.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert_eq!(unattributed.len(), 1);
        assert_eq!(unattributed[0].hp_delta, 22);
        assert_eq!(unattributed[0].context_lines, ["Holy man misses."]);
        assert_eq!(collector.buffer_len(), 0);
        assert_eq!(collector.context_window_len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn empty_buffer_negative_h_writes_unattributed_row() {
        let (mut collector, path) = collector_with_temp_db("empty-buffer");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert_eq!(unattributed.len(), 1);
        assert!(unattributed[0].context_lines.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn positive_hp_bracket_writes_no_rows_and_clears_buffer() {
        let (mut collector, path) = collector_with_temp_db("healing");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "+20"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        assert_eq!(collector.buffer_len(), 0);
        assert_eq!(collector.context_window_len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn sp_only_negative_bracket_writes_no_rows() {
        let (mut collector, path) = collector_with_temp_db("sp-only");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line(
            "H:760/782 [] S:100/100 [-40] E:100/100 [] $:100 [] exp:100 []",
            "Fueryon",
        );
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        assert_eq!(collector.buffer_len(), 0);
        assert_eq!(collector.context_window_len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn mixed_hp_and_sp_loss_uses_hp_delta_only() {
        let (mut collector, path) = collector_with_temp_db("mixed-stats");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line(
            "H:760/782 [-22] S:100/100 [-40] E:100/100 [] $:100 [] exp:100 []",
            "Fueryon",
        );
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].hp_delta, 22);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn miss_outgoing_and_dodge_lines_write_unattributed_with_context() {
        let (mut collector, path) = collector_with_temp_db("non-candidates");
        for line in [
            "Holy man misses.",
            "You miss.",
            "You puncture Holy man.",
            "You tumble Holy man's dodge.",
            "Holy man dodges.",
        ] {
            collector.handle_line(line, "Fueryon");
        }
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert_eq!(unattributed.len(), 1);
        assert_eq!(
            unattributed[0].context_lines,
            [
                "Holy man misses.",
                "You miss.",
                "You puncture Holy man.",
                "You tumble Holy man's dodge.",
                "Holy man dodges.",
            ]
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn gagged_scan_line_in_context_when_zero_candidates() {
        let (mut collector, path) = collector_with_temp_db("gagged-scan");
        collector.handle_line("Guard is slightly hurt (70%).", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert_eq!(unattributed.len(), 1);
        assert_eq!(
            unattributed[0].context_lines,
            ["Guard is slightly hurt (70%)."]
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn recognized_hit_writes_damage_events_not_unattributed() {
        let (mut collector, path) = collector_with_temp_db("recognized-hit");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert!(unattributed.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn ambiguous_batch_writes_damage_events_not_unattributed() {
        let (mut collector, path) = collector_with_temp_db("ambiguous-unattributed");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line("Holy man lightly strikes you.", "Fueryon");
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert!(unattributed.is_empty());
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn kick_partial_deflect_is_candidate() {
        let (mut collector, path) = collector_with_temp_db("kick-deflect");
        collector.handle_line(
            "Salvatore's kick lashes at you with speed, but you manage to partly deflect it in time.",
            "Fueryon",
        );
        collector.handle_line(&h_line(90, 100, "-8"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].message_verb, "kick");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn between_round_bash_still_writes_row() {
        let (mut collector, path) = collector_with_temp_db("between-round");
        collector.handle_line("Holy man's bash sends you sprawling.", "Fueryon");
        collector.handle_line(&h_line(95, 100, "-5"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].message_verb, "bash");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn hp_prompt_line_does_not_flush_or_corrupt_buffer() {
        let (mut collector, path) = collector_with_temp_db("hp-prompt");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line("Hp:760/782 Sp:100/100 Ep:100/100 Exp:100 >", "Fueryon");
        assert_eq!(collector.buffer_len(), 1);
        collector.handle_line(&h_line(738, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn reset_buffer_discards_pending_candidates_and_context() {
        let (mut collector, path) = collector_with_temp_db("reset-buffer");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.handle_line("Holy man misses.", "Fueryon");
        collector.reset_buffer();
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert!(rows.is_empty());
        let unattributed =
            DamageCollector::query_all_unattributed(collector.connection().unwrap()).unwrap();
        assert_eq!(unattributed.len(), 1);
        assert!(unattributed[0].context_lines.is_empty());
        collector.handle_line("Holy man boots you.", "Fueryon");
        collector.handle_line(&h_line(750, 782, "-10"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 1);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn player_column_matches_flush_argument() {
        let (mut collector, path) = collector_with_temp_db("players");
        collector.handle_line("Holy man bitchslaps you.", "Alpha");
        collector.handle_line(&h_line(760, 782, "-22"), "Alpha");
        collector.handle_line("Holy man boots you.", "Beta");
        collector.handle_line(&h_line(750, 782, "-10"), "Beta");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].player, "Alpha");
        assert_eq!(rows[1].player, "Beta");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn holy_man_fight_replay_writes_expected_rows() {
        let (mut collector, path) = collector_with_temp_db("holy-man-fight");
        let sequence = [
            "Holy man bitchslaps you.",
            &h_line(760, 782, "-22"),
            "Holy man lightly strikes you.",
            &h_line(738, 782, "-22"),
            "Holy man boots you.",
            &h_line(716, 782, "-22"),
            "Holy man's bash sends you sprawling.",
            &h_line(694, 782, "-22"),
            "Holy man pushes you.",
            &h_line(672, 782, "-22"),
            "Holy man misses.",
            &h_line(672, 782, ""),
            "You puncture Holy man.",
            &h_line(672, 782, ""),
        ];
        for line in sequence {
            collector.handle_line(line, "Fueryon");
        }
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 5);
        let verbs: Vec<&str> = rows.iter().map(|row| row.message_verb.as_str()).collect();
        assert_eq!(
            verbs,
            ["bitchslap", "lightly strike", "boot", "bash", "push"]
        );
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn kick_skill_replay_writes_kick_rows() {
        let (mut collector, path) = collector_with_temp_db("kick-replay");
        collector.handle_line(
            "Salvatore kicks you in the groin very hard. You gasp with pain and double up.",
            "Fueryon",
        );
        collector.handle_line(&h_line(90, 100, "-10"), "Fueryon");
        collector.handle_line(
            "Salvatore performs a quick kick to your stomach, almost making you lose your breakfast.",
            "Fueryon",
        );
        collector.handle_line(&h_line(80, 100, "-10"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert!(rows.iter().all(|row| row.message_verb == "kick"));
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn spell_replay_writes_spell_rows() {
        let (mut collector, path) = collector_with_temp_db("spell-replay");
        collector.handle_line("A magic missile hits you.", "Fueryon");
        collector.handle_line(&h_line(95, 100, "-5"), "Fueryon");
        collector.handle_line("An icebolt hits you.", "Fueryon");
        collector.handle_line(&h_line(90, 100, "-5"), "Fueryon");
        let rows = DamageCollector::query_all_events(collector.connection().unwrap()).unwrap();
        assert_eq!(rows.len(), 2);
        assert_eq!(rows[0].message_verb, "magic missile");
        assert_eq!(rows[1].message_verb, "icebolt");
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn write_failure_logs_warning_and_clears_buffers() {
        let (mut collector, path) = collector_with_temp_db("write-failure");
        collector.handle_line("Holy man bitchslaps you.", "Fueryon");
        collector.conn = None;
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        assert_eq!(collector.buffer_len(), 0);
        assert_eq!(collector.context_window_len(), 0);
        let _ = std::fs::remove_file(path);
    }

    #[test]
    fn unattributed_write_failure_clears_buffers() {
        let (mut collector, path) = collector_with_temp_db("unattributed-write-failure");
        collector.handle_line("Holy man misses.", "Fueryon");
        collector.conn = None;
        collector.handle_line(&h_line(760, 782, "-22"), "Fueryon");
        assert_eq!(collector.buffer_len(), 0);
        assert_eq!(collector.context_window_len(), 0);
        let _ = std::fs::remove_file(path);
    }
}
