use crate::combat_damage::storage::open_db;
use rusqlite::Connection;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

static FIXTURE_COUNTER: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Clone)]
pub struct FixtureRow {
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
    pub weight: f64,
    pub damage_min: i32,
    pub damage_max: i32,
    pub catalog_rank: Option<i32>,
    pub weapon_family: Option<String>,
}

impl FixtureRow {
    pub fn isolated(
        category: &str,
        verb: &str,
        player: &str,
        hp_delta: i32,
        recorded_at: &str,
    ) -> Self {
        Self {
            batch_id: 1,
            recorded_at: recorded_at.to_string(),
            player: player.to_string(),
            hp_delta,
            hp_before: hp_delta + 100,
            hp_after: 100,
            damage_category: category.to_string(),
            source_name: "Holy man".to_string(),
            message_verb: verb.to_string(),
            message_text: format!("Holy man {verb}s you."),
            candidate_count: 1,
            weight: 1.0,
            damage_min: hp_delta,
            damage_max: hp_delta,
            catalog_rank: None,
            weapon_family: None,
        }
    }

    pub fn ambiguous(
        category: &str,
        verb: &str,
        player: &str,
        hp_delta: i32,
        recorded_at: &str,
        batch_id: i64,
        candidate_count: i32,
    ) -> Self {
        Self {
            batch_id,
            recorded_at: recorded_at.to_string(),
            player: player.to_string(),
            hp_delta,
            hp_before: hp_delta + 100,
            hp_after: 100,
            damage_category: category.to_string(),
            source_name: "Holy man".to_string(),
            message_verb: verb.to_string(),
            message_text: format!("Holy man {verb}s you."),
            candidate_count,
            weight: 1.0 / f64::from(candidate_count),
            damage_min: 0,
            damage_max: hp_delta,
            catalog_rank: None,
            weapon_family: None,
        }
    }

    pub fn with_rank(mut self, rank: i32, weapon_family: &str) -> Self {
        self.catalog_rank = Some(rank);
        self.weapon_family = Some(weapon_family.to_string());
        self
    }

    pub fn with_text(mut self, text: &str) -> Self {
        self.message_text = text.to_string();
        self
    }
}

pub fn temp_db_path(name: &str) -> PathBuf {
    let id = FIXTURE_COUNTER.fetch_add(1, Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "batrs-damage-viewer-{}-{}-{}",
        name,
        id,
        std::process::id()
    ))
}

pub fn remove_db_files(path: &PathBuf) {
    let _ = std::fs::remove_file(path);
    let _ = std::fs::remove_file(format!("{}-wal", path.display()));
    let _ = std::fs::remove_file(format!("{}-shm", path.display()));
}

pub fn open_fixture_db(rows: &[FixtureRow]) -> PathBuf {
    let path = temp_db_path("fixture");
    remove_db_files(&path);
    let conn = open_db(&path).expect("open fixture db");
    insert_rows(&conn, rows);
    path
}

pub fn insert_rows(conn: &Connection, rows: &[FixtureRow]) {
    for row in rows {
        conn.execute(
            "INSERT INTO damage_events (
                batch_id, recorded_at, player, hp_delta, hp_before, hp_after,
                damage_category, source_name, message_verb, message_text,
                candidate_count, weight, damage_min, damage_max,
                catalog_rank, weapon_family
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)",
            rusqlite::params![
                row.batch_id,
                row.recorded_at,
                row.player,
                row.hp_delta,
                row.hp_before,
                row.hp_after,
                row.damage_category,
                row.source_name,
                row.message_verb,
                row.message_text,
                row.candidate_count,
                row.weight,
                row.damage_min,
                row.damage_max,
                row.catalog_rank,
                row.weapon_family,
            ],
        )
        .expect("insert fixture row");
    }
}

pub fn standard_fixture_rows() -> Vec<FixtureRow> {
    vec![
        FixtureRow::isolated("melee", "bitchslap", "Odefu", 22, "2026-08-06T14:32:00Z")
            .with_text("Holy man bitchslaps you."),
        FixtureRow::isolated("skill", "bash", "Odefu", 10, "2026-08-06T14:33:00Z")
            .with_text("Holy man's bash sends you sprawling."),
        FixtureRow::isolated("spell", "magic missile", "Odefu", 5, "2026-08-06T14:34:00Z")
            .with_text("A magic missile hits you."),
        FixtureRow::ambiguous(
            "melee",
            "bitchslap",
            "Odefu",
            22,
            "2026-08-06T14:35:00Z",
            2,
            2,
        )
        .with_text("Holy man bitchslaps you."),
        FixtureRow::ambiguous("melee", "boot", "Odefu", 22, "2026-08-06T14:35:00Z", 2, 2)
            .with_text("Holy man boots you."),
    ]
}
