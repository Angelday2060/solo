use rusqlite::Connection;
use std::path::PathBuf;

const SCHEMA_SQL: &str = include_str!("../sql/schema.sql");

fn resolve_db_path() -> Result<PathBuf, String> {
    let base = dirs::data_dir().ok_or_else(|| "无法解析应用数据目录".to_string())?;
    let dir = base.join("com.solo");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    Ok(dir.join("solo.db"))
}

/// 打开数据库、启用外键并应用 schema。
pub fn open_database() -> Result<(Connection, PathBuf), String> {
    let path = resolve_db_path()?;
    let conn = Connection::open(&path).map_err(|e| e.to_string())?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA journal_mode = WAL;",
    )
    .map_err(|e| e.to_string())?;
    conn.execute_batch(SCHEMA_SQL).map_err(|e| e.to_string())?;
    Ok((conn, path))
}
