use chrono::{DateTime, Duration, Utc};
use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: i32 = 5;

const SCHEMA_NEWER_THAN_BINARY: &str = "Database schema newer than batrs; upgrade batrs.";
pub const CANNOT_OPEN_DATABASE: &str = "Cannot open combat damage database.";

pub fn open_db(path: &Path) -> Result<Connection, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|err| err.to_string())?;
    }
    let conn = Connection::open(path).map_err(|err| err.to_string())?;
    conn.pragma_update(None, "journal_mode", "WAL")
        .map_err(|err| err.to_string())?;
    migrate(&conn)?;
    Ok(conn)
}

pub fn open_readonly_db(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open_with_flags(
        path,
        OpenFlags::SQLITE_OPEN_READ_ONLY | OpenFlags::SQLITE_OPEN_NO_MUTEX,
    )
    .map_err(|_| CANNOT_OPEN_DATABASE.to_string())?;
    validate_readable_schema(&conn)?;
    Ok(conn)
}

fn validate_readable_schema(conn: &Connection) -> Result<(), String> {
    let version = read_schema_version(conn)?;
    match version {
        None => Err(CANNOT_OPEN_DATABASE.to_string()),
        Some(existing) if existing > CURRENT_SCHEMA_VERSION => {
            Err(SCHEMA_NEWER_THAN_BINARY.to_string())
        }
        Some(_) => Ok(()),
    }
}

fn migrate(conn: &Connection) -> Result<(), String> {
    let version = read_schema_version(conn)?;
    match version {
        None => create_schema(conn, CURRENT_SCHEMA_VERSION)?,
        Some(existing) if existing > CURRENT_SCHEMA_VERSION => {
            return Err(SCHEMA_NEWER_THAN_BINARY.to_string());
        }
        Some(existing) if existing < CURRENT_SCHEMA_VERSION => {
            apply_migrations(conn, existing)?;
        }
        Some(_) => {}
    }
    backfill_melee_catalog_metadata(conn)?;
    backfill_riposte_from_unattributed(conn)?;
    Ok(())
}

const RIPOSTE_BACKFILL_WINDOW: Duration = Duration::seconds(30);

struct PendingParry {
    source: String,
    recorded_at: DateTime<Utc>,
    row_id: i64,
}

pub fn backfill_riposte_from_unattributed(conn: &Connection) -> Result<usize, String> {
    use crate::combat_damage::collector::parse_json_string_array;
    use crate::combat_damage::matcher::{
        enemy_parry_source, is_riposte_follow_up, orphan_riposte_candidate,
        riposte_pair_from_context,
    };

    let mut select = conn
        .prepare(
            "SELECT id, recorded_at, player, hp_delta, hp_before, hp_after, context_lines
             FROM unattributed_hp_events
             ORDER BY id",
        )
        .map_err(|err| err.to_string())?;
    let rows = select
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, i32>(3)?,
                row.get::<_, i32>(4)?,
                row.get::<_, i32>(5)?,
                row.get::<_, String>(6)?,
            ))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;

    if rows.is_empty() {
        return Ok(0);
    }

    let mut next_batch_id: i64 = conn
        .query_row(
            "SELECT COALESCE(MAX(batch_id), 0) + 1 FROM damage_events",
            [],
            |row| row.get(0),
        )
        .unwrap_or(1);
    let mut pending_parries: HashMap<String, Vec<PendingParry>> = HashMap::new();
    let mut converted = 0usize;

    for (row_id, recorded_at, player, hp_delta, hp_before, hp_after, context_json) in rows {
        let recorded_at_dt = DateTime::parse_from_rfc3339(&recorded_at)
            .map(|value| value.with_timezone(&Utc))
            .map_err(|err| err.to_string())?;
        let context_lines = parse_json_string_array(&context_json);
        let line_refs: Vec<&str> = context_lines.iter().map(String::as_str).collect();

        let candidate = riposte_pair_from_context(&line_refs).or_else(|| {
            if line_refs.len() != 1 || !is_riposte_follow_up(line_refs[0]) {
                return None;
            }
            let parry = pending_parries.get(&player)?.iter().rev().find(|entry| {
                entry.row_id < row_id
                    && recorded_at_dt.signed_duration_since(entry.recorded_at)
                        <= RIPOSTE_BACKFILL_WINDOW
            })?;
            orphan_riposte_candidate(&parry.source, line_refs[0])
        });

        if let Some(candidate) = candidate {
            let transaction = conn
                .unchecked_transaction()
                .map_err(|err| err.to_string())?;
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
                        next_batch_id,
                        recorded_at,
                        player,
                        hp_delta,
                        hp_before,
                        hp_after,
                        "skill",
                        candidate.source_name,
                        candidate.message_verb,
                        candidate.message_text,
                        1,
                        1.0,
                        hp_delta,
                        hp_delta,
                        None::<i32>,
                        None::<String>,
                        1.0,
                    ],
                )
                .map_err(|err| err.to_string())?;
            transaction
                .execute("DELETE FROM unattributed_hp_events WHERE id = ?1", [row_id])
                .map_err(|err| err.to_string())?;
            transaction.commit().map_err(|err| err.to_string())?;
            next_batch_id += 1;
            converted += 1;
        }

        if let Some(pending) = pending_parries.get_mut(&player) {
            pending.retain(|entry| {
                recorded_at_dt.signed_duration_since(entry.recorded_at) <= RIPOSTE_BACKFILL_WINDOW
            });
        }
        for line in &context_lines {
            if let Some(source) = enemy_parry_source(line) {
                pending_parries
                    .entry(player.clone())
                    .or_default()
                    .push(PendingParry {
                        source,
                        recorded_at: recorded_at_dt,
                        row_id,
                    });
            }
        }
    }

    Ok(converted)
}

fn backfill_melee_catalog_metadata(conn: &Connection) -> Result<(), String> {
    use crate::combat_damage::catalog::{FAMILY_IDS, unambiguous_melee_catalog_meta};

    let mut select = conn
        .prepare(
            "SELECT id, message_verb FROM damage_events
             WHERE damage_category = 'melee' AND weapon_family IS NULL",
        )
        .map_err(|err| err.to_string())?;
    let rows = select
        .query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;

    if rows.is_empty() {
        return Ok(());
    }

    let mut update = conn
        .prepare("UPDATE damage_events SET catalog_rank = ?1, weapon_family = ?2 WHERE id = ?3")
        .map_err(|err| err.to_string())?;
    for (id, verb) in rows {
        let Some(meta) = unambiguous_melee_catalog_meta(&verb) else {
            continue;
        };
        update
            .execute(rusqlite::params![
                i32::from(meta.rank),
                FAMILY_IDS[meta.family_index],
                id,
            ])
            .map_err(|err| err.to_string())?;
    }
    Ok(())
}

fn read_schema_version(conn: &Connection) -> Result<Option<i32>, String> {
    let table_exists = conn
        .prepare("SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'schema_version'")
        .map_err(|err| err.to_string())?
        .exists([])
        .map_err(|err| err.to_string())?;
    if !table_exists {
        return Ok(None);
    }
    conn.query_row("SELECT version FROM schema_version LIMIT 1", [], |row| {
        row.get(0)
    })
    .optional()
    .map_err(|err| err.to_string())
}

fn create_schema(conn: &Connection, version: i32) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE schema_version (
            version INTEGER NOT NULL
        );
        CREATE TABLE damage_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            batch_id INTEGER NOT NULL,
            recorded_at TEXT NOT NULL,
            player TEXT NOT NULL,
            hp_delta INTEGER NOT NULL,
            hp_before INTEGER NOT NULL,
            hp_after INTEGER NOT NULL,
            damage_category TEXT NOT NULL,
            source_name TEXT NOT NULL,
            message_verb TEXT NOT NULL,
            message_text TEXT NOT NULL,
            candidate_count INTEGER NOT NULL,
            confidence REAL NOT NULL,
            damage_min INTEGER NOT NULL,
            damage_max INTEGER NOT NULL,
            catalog_rank INTEGER,
            weapon_family TEXT,
            weight REAL NOT NULL
        );
        CREATE INDEX idx_damage_events_batch_id ON damage_events (batch_id);
        CREATE INDEX idx_damage_events_recorded_at ON damage_events (recorded_at);
        CREATE INDEX idx_damage_events_category_verb ON damage_events (damage_category, message_verb);
        CREATE INDEX idx_damage_events_candidate_count ON damage_events (candidate_count);
        CREATE TABLE unattributed_hp_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recorded_at TEXT NOT NULL,
            player TEXT NOT NULL,
            hp_delta INTEGER NOT NULL,
            hp_before INTEGER NOT NULL,
            hp_after INTEGER NOT NULL,
            h_line_text TEXT NOT NULL,
            context_lines TEXT NOT NULL,
            reviewed_at TEXT
        );
        CREATE INDEX idx_unattributed_hp_events_recorded_at ON unattributed_hp_events (recorded_at);
        CREATE INDEX idx_unattributed_hp_events_player ON unattributed_hp_events (player);
        ",
    )
    .map_err(|err| err.to_string())?;
    conn.execute(
        "INSERT INTO schema_version (version) VALUES (?1)",
        [version],
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn apply_migrations(conn: &Connection, from_version: i32) -> Result<(), String> {
    let mut version = from_version;
    if version == 1 {
        migrate_v1_to_v2(conn)?;
        version = 2;
    }
    if version == 2 {
        migrate_v2_to_v3(conn)?;
        version = 3;
    }
    if version == 3 {
        migrate_v3_to_v4(conn)?;
        version = 4;
    }
    if version == 4 {
        migrate_v4_to_v5(conn)?;
        version = 5;
    }
    if version < CURRENT_SCHEMA_VERSION {
        return Err(format!(
            "missing migration from schema version {version} to {CURRENT_SCHEMA_VERSION}"
        ));
    }
    Ok(())
}

fn migrate_v1_to_v2(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        ALTER TABLE damage_events ADD COLUMN catalog_rank INTEGER;
        ALTER TABLE damage_events ADD COLUMN weapon_family TEXT;
        UPDATE schema_version SET version = 2;
        ",
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn migrate_v4_to_v5(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        ALTER TABLE unattributed_hp_events ADD COLUMN reviewed_at TEXT;
        UPDATE schema_version SET version = 5;
        ",
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn migrate_v3_to_v4(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        CREATE TABLE unattributed_hp_events (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            recorded_at TEXT NOT NULL,
            player TEXT NOT NULL,
            hp_delta INTEGER NOT NULL,
            hp_before INTEGER NOT NULL,
            hp_after INTEGER NOT NULL,
            h_line_text TEXT NOT NULL,
            context_lines TEXT NOT NULL
        );
        CREATE INDEX idx_unattributed_hp_events_recorded_at ON unattributed_hp_events (recorded_at);
        CREATE INDEX idx_unattributed_hp_events_player ON unattributed_hp_events (player);
        UPDATE schema_version SET version = 4;
        ",
    )
    .map_err(|err| err.to_string())?;
    Ok(())
}

fn migrate_v2_to_v3(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "
        ALTER TABLE damage_events RENAME COLUMN weight TO confidence;
        ALTER TABLE damage_events ADD COLUMN weight REAL NOT NULL DEFAULT 1.0;
        UPDATE schema_version SET version = 3;
        ",
    )
    .map_err(|err| err.to_string())?;
    backfill_catalog_weights(conn)?;
    Ok(())
}

fn backfill_catalog_weights(conn: &Connection) -> Result<(), String> {
    let mut statement = conn
        .prepare("SELECT id, batch_id, catalog_rank FROM damage_events ORDER BY batch_id, id")
        .map_err(|err| err.to_string())?;
    let rows = statement
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, Option<i32>>(2)?,
            ))
        })
        .map_err(|err| err.to_string())?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| err.to_string())?;

    let mut batches: std::collections::BTreeMap<i64, Vec<(i64, Option<i32>)>> =
        std::collections::BTreeMap::new();
    for (id, batch_id, catalog_rank) in rows {
        batches
            .entry(batch_id)
            .or_default()
            .push((id, catalog_rank));
    }

    let mut update = conn
        .prepare("UPDATE damage_events SET weight = ?1 WHERE id = ?2")
        .map_err(|err| err.to_string())?;
    for (_batch_id, members) in batches {
        let ranks: Vec<Option<i32>> = members.iter().map(|(_, rank)| *rank).collect();
        let weights = crate::combat_damage::attribution::catalog_weights(&ranks);
        for ((id, _), weight) in members.iter().zip(weights) {
            update
                .execute(rusqlite::params![weight, id])
                .map_err(|err| err.to_string())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn temp_db_path(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!(
            "batrs-combat-damage-{}-{}",
            name,
            std::process::id()
        ))
    }

    fn table_names(conn: &Connection) -> Vec<String> {
        conn.prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
            .unwrap()
            .query_map([], |row| row.get(0))
            .unwrap()
            .map(|name| name.unwrap())
            .collect()
    }

    fn index_names(conn: &Connection) -> Vec<String> {
        conn.prepare(
            "SELECT name FROM sqlite_master WHERE type = 'index' AND name LIKE 'idx_damage_events_%' ORDER BY name",
        )
        .unwrap()
        .query_map([], |row| row.get(0))
        .unwrap()
        .map(|name| name.unwrap())
        .collect()
    }

    fn damage_event_columns(conn: &Connection) -> Vec<String> {
        conn.prepare("PRAGMA table_info(damage_events)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(|name| name.unwrap())
            .collect()
    }

    #[test]
    fn fresh_open_creates_v5_schema_and_indexes() {
        let path = temp_db_path("fresh");
        let _ = fs::remove_file(&path);
        let conn = open_db(&path).expect("open fresh db");
        assert_eq!(
            table_names(&conn)
                .into_iter()
                .filter(|name| name != "sqlite_sequence")
                .collect::<Vec<_>>(),
            ["damage_events", "schema_version", "unattributed_hp_events"]
        );
        assert_eq!(
            damage_event_columns(&conn),
            [
                "id",
                "batch_id",
                "recorded_at",
                "player",
                "hp_delta",
                "hp_before",
                "hp_after",
                "damage_category",
                "source_name",
                "message_verb",
                "message_text",
                "candidate_count",
                "confidence",
                "damage_min",
                "damage_max",
                "catalog_rank",
                "weapon_family",
                "weight",
            ]
        );
        assert_eq!(
            index_names(&conn),
            [
                "idx_damage_events_batch_id",
                "idx_damage_events_candidate_count",
                "idx_damage_events_category_verb",
                "idx_damage_events_recorded_at",
            ]
        );
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM damage_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(row_count, 0);
        let unattributed_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM unattributed_hp_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(unattributed_count, 0);
        let unattributed_columns: Vec<String> = conn
            .prepare("PRAGMA table_info(unattributed_hp_events)")
            .unwrap()
            .query_map([], |row| row.get::<_, String>(1))
            .unwrap()
            .map(|name| name.unwrap())
            .collect();
        assert!(unattributed_columns.contains(&"reviewed_at".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v4_database_migrates_to_v5() {
        let path = temp_db_path("migrate-v4");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                confidence REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT,
                weight REAL NOT NULL
            );
            CREATE TABLE unattributed_hp_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                h_line_text TEXT NOT NULL,
                context_lines TEXT NOT NULL
            );
            INSERT INTO schema_version (version) VALUES (4);
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("migrate v4 db");
        let conn = Connection::open(&path).unwrap();
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        let reviewed_at_exists: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('unattributed_hp_events') WHERE name = 'reviewed_at'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(reviewed_at_exists, 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v3_database_migrates_to_v5() {
        let path = temp_db_path("migrate-v3");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                confidence REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT,
                weight REAL NOT NULL
            );
            INSERT INTO schema_version (version) VALUES (3);
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("migrate v3 db");
        let conn = Connection::open(&path).unwrap();
        assert!(table_names(&conn).contains(&"unattributed_hp_events".to_string()));
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v1_database_migrates_to_v5() {
        let path = temp_db_path("migrate-v1");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                weight REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL
            );
            INSERT INTO schema_version (version) VALUES (1);
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("migrate v1 db");
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            damage_event_columns(&conn),
            [
                "id",
                "batch_id",
                "recorded_at",
                "player",
                "hp_delta",
                "hp_before",
                "hp_after",
                "damage_category",
                "source_name",
                "message_verb",
                "message_text",
                "candidate_count",
                "confidence",
                "damage_min",
                "damage_max",
                "catalog_rank",
                "weapon_family",
                "weight",
            ]
        );
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v2_database_migrates_to_v5() {
        let path = temp_db_path("migrate-v2");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                weight REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT
            );
            INSERT INTO schema_version (version) VALUES (2);
            INSERT INTO damage_events (
                batch_id, recorded_at, player, hp_delta, hp_before, hp_after,
                damage_category, source_name, message_verb, message_text,
                candidate_count, weight, damage_min, damage_max, catalog_rank, weapon_family
            ) VALUES
                (101, '2026-08-06T12:00:00Z', 'Odefu', 35, 135, 100, 'melee', 'Orc', 'barely scrape',
                 'Orc barely scrapes you.', 2, 0.5, 0, 35, 3, 'claw'),
                (101, '2026-08-06T12:00:00Z', 'Odefu', 35, 135, 100, 'melee', 'Orc', 'prick',
                 'Orc pricks you.', 2, 0.5, 0, 35, 5, 'stab');
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("migrate v2 db");
        let conn = Connection::open(&path).unwrap();
        assert_eq!(
            damage_event_columns(&conn),
            [
                "id",
                "batch_id",
                "recorded_at",
                "player",
                "hp_delta",
                "hp_before",
                "hp_after",
                "damage_category",
                "source_name",
                "message_verb",
                "message_text",
                "candidate_count",
                "confidence",
                "damage_min",
                "damage_max",
                "catalog_rank",
                "weapon_family",
                "weight",
            ]
        );
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 5);
        let barely_scrape: (f64, f64) = conn
            .query_row(
                "SELECT confidence, weight FROM damage_events WHERE message_verb = 'barely scrape'",
                [],
                |row| Ok((row.get::<_, f64>(0)?, row.get::<_, f64>(1)?)),
            )
            .unwrap();
        assert!((barely_scrape.0 - 0.5).abs() < 1e-9);
        assert!((barely_scrape.1 - 0.375).abs() < 1e-9);
        let prick: f64 = conn
            .query_row(
                "SELECT weight FROM damage_events WHERE message_verb = 'prick'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!((prick - 0.625).abs() < 1e-9);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn null_melee_weapon_family_backfilled_from_catalog() {
        let path = temp_db_path("backfill-family");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                confidence REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT,
                weight REAL NOT NULL
            );
            INSERT INTO schema_version (version) VALUES (3);
            INSERT INTO damage_events (
                batch_id, recorded_at, player, hp_delta, hp_before, hp_after,
                damage_category, source_name, message_verb, message_text,
                candidate_count, confidence, damage_min, damage_max, catalog_rank, weapon_family, weight
            ) VALUES
                (1, '2026-08-06T07:18:00Z', 'Odefu', 10, 100, 90, 'melee', 'Bunny', 'scrape',
                 'Bunny scrapes you.', 1, 1.0, 10, 10, NULL, NULL, 1.0),
                (2, '2026-08-06T07:19:00Z', 'Odefu', 12, 90, 78, 'melee', 'Bunny', 'savagely strike',
                 'Bunny savagely strikes you.', 1, 1.0, 12, 12, NULL, NULL, 1.0);
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("backfill on open");
        let conn = Connection::open(&path).unwrap();
        let scrape: (Option<i32>, Option<String>) = conn
            .query_row(
                "SELECT catalog_rank, weapon_family FROM damage_events WHERE message_verb = 'scrape'",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();
        assert_eq!(scrape, (Some(4), Some("claw".to_string())));
        let savagely: Option<String> = conn
            .query_row(
                "SELECT weapon_family FROM damage_events WHERE message_verb = 'savagely strike'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(savagely, None);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn second_open_on_same_file_succeeds() {
        let path = temp_db_path("reopen");
        let _ = fs::remove_file(&path);
        open_db(&path).expect("first open");
        open_db(&path).expect("second open");
        let _ = fs::remove_file(path);
    }

    #[test]
    fn newer_schema_version_returns_clear_error() {
        let path = temp_db_path("newer");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_version (version INTEGER NOT NULL);
             INSERT INTO schema_version (version) VALUES (99);",
        )
        .unwrap();
        let err = open_db(&path).unwrap_err();
        assert_eq!(err, SCHEMA_NEWER_THAN_BINARY);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn open_db_creates_parent_directory() {
        let parent = temp_db_path("parent");
        let _ = fs::remove_dir_all(&parent);
        let path = parent.join("nested").join("combat_damage.db");
        open_db(&path).expect("open with missing parent");
        assert!(path.exists());
        let _ = fs::remove_dir_all(parent);
    }

    #[test]
    fn backfill_riposte_from_unattributed_same_context_row() {
        let path = temp_db_path("backfill-riposte-same-context");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            INSERT INTO schema_version (version) VALUES (5);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                confidence REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT,
                weight REAL NOT NULL
            );
            CREATE TABLE unattributed_hp_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                h_line_text TEXT NOT NULL,
                context_lines TEXT NOT NULL,
                reviewed_at TEXT
            );
            INSERT INTO unattributed_hp_events (
                recorded_at, player, hp_delta, hp_before, hp_after, h_line_text, context_lines
            ) VALUES (
                '2026-08-06T12:00:01Z', 'Fueryon', 10, 100, 90,
                'H:90/100 [-10] S:100/100 [] E:100/100 [] $:100 [] exp:100 []',
                '[\"Gaward parries.\",\"..AND ripostes.\",\"Gaward misses.\"]'
            );
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("open and backfill");
        let conn = Connection::open(&path).unwrap();
        let damage_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM damage_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(damage_count, 1);
        let verb: String = conn
            .query_row("SELECT message_verb FROM damage_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(verb, "riposte");
        let unattributed_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM unattributed_hp_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(unattributed_count, 0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn backfill_riposte_from_unattributed_pairs_orphan_follow_up_with_prior_parry_row() {
        let path = temp_db_path("backfill-riposte-orphan");
        let _ = fs::remove_file(&path);
        let conn = Connection::open(&path).unwrap();
        conn.execute_batch(
            "
            CREATE TABLE schema_version (version INTEGER NOT NULL);
            INSERT INTO schema_version (version) VALUES (5);
            CREATE TABLE damage_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                batch_id INTEGER NOT NULL,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                damage_category TEXT NOT NULL,
                source_name TEXT NOT NULL,
                message_verb TEXT NOT NULL,
                message_text TEXT NOT NULL,
                candidate_count INTEGER NOT NULL,
                confidence REAL NOT NULL,
                damage_min INTEGER NOT NULL,
                damage_max INTEGER NOT NULL,
                catalog_rank INTEGER,
                weapon_family TEXT,
                weight REAL NOT NULL
            );
            CREATE TABLE unattributed_hp_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                recorded_at TEXT NOT NULL,
                player TEXT NOT NULL,
                hp_delta INTEGER NOT NULL,
                hp_before INTEGER NOT NULL,
                hp_after INTEGER NOT NULL,
                h_line_text TEXT NOT NULL,
                context_lines TEXT NOT NULL,
                reviewed_at TEXT
            );
            INSERT INTO unattributed_hp_events (
                recorded_at, player, hp_delta, hp_before, hp_after, h_line_text, context_lines
            ) VALUES
                ('2026-08-06T12:00:00Z', 'Fueryon', 5, 100, 95,
                 'H:95/100 [-5] S:100/100 [] E:100/100 [] $:100 [] exp:100 []',
                 '[\"Barney parries.\"]'),
                ('2026-08-06T12:00:01Z', 'Fueryon', 10, 95, 85,
                 'H:85/100 [-10] S:100/100 [] E:100/100 [] $:100 [] exp:100 []',
                 '[\" ...AND counterattacks.\"]');
            ",
        )
        .unwrap();
        drop(conn);
        open_db(&path).expect("open and backfill");
        let conn = Connection::open(&path).unwrap();
        let riposte_rows: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM damage_events WHERE message_verb = 'riposte'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(riposte_rows, 1);
        let source: String = conn
            .query_row(
                "SELECT source_name FROM damage_events WHERE message_verb = 'riposte'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(source, "Barney");
        let unattributed_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM unattributed_hp_events", [], |row| {
                row.get(0)
            })
            .unwrap();
        assert_eq!(unattributed_count, 1);
        let _ = fs::remove_file(path);
    }

    #[test]
    #[ignore = "manual: BATRS_DAMAGE_DB=/path/to/combat_damage.db cargo test backfill_user_damage_db -- --ignored"]
    fn backfill_user_damage_db() {
        let path = std::env::var("BATRS_DAMAGE_DB").expect("set BATRS_DAMAGE_DB");
        let path = std::path::Path::new(&path);
        let before: i64 = Connection::open(path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM damage_events WHERE message_verb = 'riposte'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        open_db(path).expect("open and backfill user db");
        let after: i64 = Connection::open(path)
            .unwrap()
            .query_row(
                "SELECT COUNT(*) FROM damage_events WHERE message_verb = 'riposte'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        eprintln!(
            "riposte backfill for {}: {} -> {} riposte events",
            path.display(),
            before,
            after
        );
    }
}
