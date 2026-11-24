use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use biu_library::{backup_path, Database, LibraryError, SCHEMA_VERSION};

fn temp_db(name: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!("biu-migrate-{name}-{stamp}"));
    fs::create_dir_all(&dir).unwrap();
    dir.join("library.db")
}

#[test]
fn fresh_database_is_version_one_and_idempotent() {
    let path = temp_db("fresh");
    {
        let db = Database::open(&path).unwrap();
        let version: i64 = db
            .connection()
            .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))
            .unwrap();
        assert_eq!(version, SCHEMA_VERSION);
        let tracks: i64 = db
            .connection()
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap();
        assert_eq!(tracks, 0);
    }
    assert!(!backup_path(&path).exists(), "fresh db must not be backed up");

    // Re-open must not re-run the migration or create a backup.
    let db = Database::open(&path).unwrap();
    let migrations: i64 = db
        .connection()
        .query_row("SELECT COUNT(*) FROM schema_migrations", [], |row| row.get(0))
        .unwrap();
    assert_eq!(migrations, 1);
    drop(db);
    assert!(!backup_path(&path).exists());
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn legacy_database_is_backed_up_before_migration() {
    let path = temp_db("legacy");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch("CREATE TABLE legacy (id INTEGER);").unwrap();
    }

    let db = Database::open(&path).unwrap();
    let version: i64 = db
        .connection()
        .query_row("SELECT MAX(version) FROM schema_migrations", [], |row| row.get(0))
        .unwrap();
    assert_eq!(version, SCHEMA_VERSION);
    drop(db);

    let backup = backup_path(&path);
    assert!(backup.exists(), "legacy db must be copied to .bak");
    let backup_conn = rusqlite::Connection::open(&backup).unwrap();
    let has_legacy: bool = backup_conn
        .query_row(
            "SELECT EXISTS(SELECT 1 FROM sqlite_master WHERE type='table' AND name='legacy')",
            [],
            |row| row.get(0),
        )
        .unwrap();
    assert!(has_legacy, "backup must preserve pre-migration schema");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn newer_schema_is_rejected_not_downgraded() {
    let path = temp_db("newer");
    {
        let conn = rusqlite::Connection::open(&path).unwrap();
        conn.execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY, applied_at TEXT NOT NULL);
             INSERT INTO schema_migrations VALUES (99, datetime('now'));",
        )
        .unwrap();
    }

    match Database::open(&path) {
        Err(LibraryError::SchemaTooNew { found, supported }) => {
            assert_eq!(found, 99);
            assert_eq!(supported, SCHEMA_VERSION);
        }
        Err(other) => panic!("expected SchemaTooNew, got {other}"),
        Ok(_) => panic!("expected SchemaTooNew, got a database"),
    }
    assert!(!backup_path(&path).exists(), "must not touch a newer database");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn corrupt_file_fails_without_panic() {
    let path = temp_db("corrupt");
    fs::write(&path, b"not a sqlite database").unwrap();

    let result = Database::open(&path);
    assert!(result.is_err(), "corrupt db must be reported");
    let _ = fs::remove_dir_all(path.parent().unwrap());
}
