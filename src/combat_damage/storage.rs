use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::fs;
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: i32 = 3;

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
    Ok(())
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
        return Ok(());
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
    fn fresh_open_creates_v3_schema_and_indexes() {
        let path = temp_db_path("fresh");
        let _ = fs::remove_file(&path);
        let conn = open_db(&path).expect("open fresh db");
        assert_eq!(
            table_names(&conn)
                .into_iter()
                .filter(|name| name != "sqlite_sequence")
                .collect::<Vec<_>>(),
            ["damage_events", "schema_version"]
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
        assert_eq!(version, 3);
        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM damage_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(row_count, 0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v1_database_migrates_to_v3() {
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
        assert_eq!(version, 3);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v2_database_migrates_to_v3() {
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
        assert_eq!(version, 3);
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
}
