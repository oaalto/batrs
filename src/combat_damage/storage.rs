use rusqlite::{Connection, OpenFlags, OptionalExtension};
use std::fs;
use std::path::Path;

pub const CURRENT_SCHEMA_VERSION: i32 = 2;

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
            weight REAL NOT NULL,
            damage_min INTEGER NOT NULL,
            damage_max INTEGER NOT NULL,
            catalog_rank INTEGER,
            weapon_family TEXT
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
    if from_version == 1 {
        migrate_v1_to_v2(conn)?;
        return Ok(());
    }
    Err(format!(
        "missing migration from schema version {from_version} to {CURRENT_SCHEMA_VERSION}"
    ))
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
    fn fresh_open_creates_v2_schema_and_indexes() {
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
                "weight",
                "damage_min",
                "damage_max",
                "catalog_rank",
                "weapon_family",
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
        assert_eq!(version, 2);
        let row_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM damage_events", [], |row| row.get(0))
            .unwrap();
        assert_eq!(row_count, 0);
        let _ = fs::remove_file(path);
    }

    #[test]
    fn v1_database_migrates_to_v2() {
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
                "weight",
                "damage_min",
                "damage_max",
                "catalog_rank",
                "weapon_family",
            ]
        );
        let version: i32 = conn
            .query_row("SELECT version FROM schema_version", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, 2);
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
