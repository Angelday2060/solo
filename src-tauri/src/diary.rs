use crate::DbState;
use chrono::{Local, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaryEntryListItem {
    pub id: String,
    pub entry_date: String,
    pub title: Option<String>,
    pub preview: String,
    pub sort_order: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaryEntryDto {
    pub id: String,
    pub entry_date: String,
    pub title: Option<String>,
    pub body: String,
    pub weather: Option<String>,
    pub mood: Option<String>,
    pub sort_order: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaryListFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaryCreateEntry {
    pub entry_date: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
    pub weather: Option<String>,
    pub mood: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiaryUpdateEntry {
    pub id: String,
    pub entry_date: String,
    pub title: Option<String>,
    pub body: String,
    pub weather: Option<String>,
    pub mood: Option<String>,
}

const WEATHER_VALUES: &[&str] = &["sunny", "cloudy", "overcast", "rain", "snow", "fog", "wind"];
const MOOD_VALUES: &[&str] = &["calm", "happy", "low", "anxious", "excited", "tired"];

fn conn<'a>(
    state: &'a tauri::State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    (*state).0.lock().map_err(|e| e.to_string())
}

fn now_iso() -> String {
    Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn today_ymd() -> String {
    Local::now().format("%Y-%m-%d").to_string()
}

fn parse_iso_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| "日期须为 YYYY-MM-DD".into())
}

fn norm_optional_text(raw: &Option<String>) -> Option<String> {
    raw.as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

fn norm_weather(raw: &Option<String>) -> Result<Option<String>, String> {
    match norm_optional_text(raw) {
        None => Ok(None),
        Some(v) if WEATHER_VALUES.contains(&v.as_str()) => Ok(Some(v)),
        _ => Err("天气选项无效".into()),
    }
}

fn norm_mood(raw: &Option<String>) -> Result<Option<String>, String> {
    match norm_optional_text(raw) {
        None => Ok(None),
        Some(v) if MOOD_VALUES.contains(&v.as_str()) => Ok(Some(v)),
        _ => Err("心情选项无效".into()),
    }
}

fn body_preview(body: &str) -> String {
    let line = body
        .lines()
        .find(|l| !l.trim().is_empty())
        .map(str::trim)
        .unwrap_or("");
    if line.is_empty() {
        return String::new();
    }
    let mut chars = line.chars();
    let preview: String = chars.by_ref().take(48).collect();
    if chars.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}

fn map_entry_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<DiaryEntryDto> {
    Ok(DiaryEntryDto {
        id: row.get(0)?,
        entry_date: row.get(1)?,
        title: row.get(2)?,
        body: row.get(3)?,
        weather: row.get(4)?,
        mood: row.get(5)?,
        sort_order: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
    })
}

fn next_sort_order(conn: &Connection, entry_date: &str) -> Result<i32, String> {
    let min_so: Option<i32> = conn
        .query_row(
            "SELECT MIN(sort_order) FROM diary_entry WHERE entry_date = ?1",
            params![entry_date],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(min_so.map(|v| v - 1).unwrap_or(0))
}

#[tauri::command]
pub fn diary_list_entries(
    state: tauri::State<'_, DbState>,
    filter: DiaryListFilter,
) -> Result<Vec<DiaryEntryListItem>, String> {
    let conn = conn(&state)?;
    let mut sql = String::from(
        "SELECT id, entry_date, title, body, sort_order
         FROM diary_entry
         WHERE 1=1",
    );
    let mut pvec: Vec<String> = Vec::new();

    if let Some(ref d) = filter.date_from {
        if !d.is_empty() {
            sql.push_str(" AND entry_date >= ?");
            pvec.push(d.clone());
        }
    }
    if let Some(ref d) = filter.date_to {
        if !d.is_empty() {
            sql.push_str(" AND entry_date <= ?");
            pvec.push(d.clone());
        }
    }
    if let Some(ref s) = filter.search {
        let t = s.trim();
        if !t.is_empty() {
            let pat = format!("%{}%", t.replace('%', "\\%").replace('_', "\\_"));
            sql.push_str(" AND (IFNULL(title, '') LIKE ? ESCAPE '\\' OR body LIKE ? ESCAPE '\\')");
            pvec.push(pat.clone());
            pvec.push(pat);
        }
    }

    sql.push_str(" ORDER BY entry_date DESC, sort_order ASC, created_at DESC, id DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(pvec.iter()), |row| {
            let body: String = row.get(3)?;
            let title: Option<String> = row.get(2)?;
            let preview = if let Some(ref t) = title {
                if !t.trim().is_empty() {
                    t.trim().to_string()
                } else {
                    body_preview(&body)
                }
            } else {
                body_preview(&body)
            };
            Ok(DiaryEntryListItem {
                id: row.get(0)?,
                entry_date: row.get(1)?,
                title,
                preview,
                sort_order: row.get(4)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn diary_get_entry(
    state: tauri::State<'_, DbState>,
    id: String,
) -> Result<Option<DiaryEntryDto>, String> {
    if id.is_empty() {
        return Ok(None);
    }
    let conn = conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, entry_date, title, body, weather, mood, sort_order, created_at, updated_at
             FROM diary_entry WHERE id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt
        .query_map(params![id], map_entry_row)
        .map_err(|e| e.to_string())?;
    match rows.next() {
        Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn diary_create_entry(
    state: tauri::State<'_, DbState>,
    payload: DiaryCreateEntry,
) -> Result<DiaryEntryDto, String> {
    let entry_date = match payload.entry_date {
        Some(ref d) if !d.trim().is_empty() => {
            parse_iso_date(d)?;
            d.trim().to_string()
        }
        _ => today_ymd(),
    };
    let weather = norm_weather(&payload.weather)?;
    let mood = norm_mood(&payload.mood)?;
    let title = norm_optional_text(&payload.title);
    let body = payload.body.unwrap_or_default();
    let conn = conn(&state)?;
    let sort_order = next_sort_order(&conn, &entry_date)?;
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO diary_entry (id, entry_date, title, body, weather, mood, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![id, entry_date, title, body, weather, mood, sort_order, ts, ts],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    diary_get_entry(state, id)?.ok_or_else(|| "写入后读取失败".into())
}

#[tauri::command]
pub fn diary_update_entry(
    state: tauri::State<'_, DbState>,
    payload: DiaryUpdateEntry,
) -> Result<DiaryEntryDto, String> {
    if payload.id.is_empty() {
        return Err("条目 id 无效".into());
    }
    parse_iso_date(&payload.entry_date)?;
    let weather = norm_weather(&payload.weather)?;
    let mood = norm_mood(&payload.mood)?;
    let title = norm_optional_text(&payload.title);
    let conn = conn(&state)?;
    let n: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM diary_entry WHERE id = ?1",
            params![payload.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("日记不存在".into());
    }
    let ts = now_iso();
    conn.execute(
        "UPDATE diary_entry SET entry_date = ?1, title = ?2, body = ?3, weather = ?4, mood = ?5, updated_at = ?6
         WHERE id = ?7",
        params![
            payload.entry_date.trim(),
            title,
            payload.body,
            weather,
            mood,
            ts,
            payload.id
        ],
    )
    .map_err(|e| e.to_string())?;
    let rid = payload.id.clone();
    drop(conn);
    diary_get_entry(state, rid)?.ok_or_else(|| "日记不存在".into())
}

#[tauri::command]
pub fn diary_delete_entry(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    if id.is_empty() {
        return Err("条目 id 无效".into());
    }
    let conn = conn(&state)?;
    conn.execute("DELETE FROM diary_entry WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
