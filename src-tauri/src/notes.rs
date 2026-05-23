use crate::DbState;
use chrono::Local;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteNotebookDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteTagDto {
    pub id: String,
    pub name: String,
    pub created_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteAttachmentDto {
    pub id: String,
    pub note_id: String,
    pub file_path: String,
    pub display_name: Option<String>,
    pub sort_order: i32,
    pub created_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteListItem {
    pub id: String,
    pub notebook_id: Option<String>,
    pub title: Option<String>,
    pub preview: String,
    pub is_pinned: bool,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteDto {
    pub id: String,
    pub notebook_id: Option<String>,
    pub title: Option<String>,
    pub body: String,
    pub is_pinned: bool,
    pub sort_order: i32,
    pub tags: Vec<NoteTagDto>,
    pub attachments: Vec<NoteAttachmentDto>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteListFilter {
    /// `Some(Some(id))` = 指定笔记本；`Some(None)` = 未分类；`None` = 全部
    pub notebook_id: Option<Option<String>>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteCreateNotebook {
    pub name: String,
    pub parent_id: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUpdateNotebook {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub is_pinned: Option<bool>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteCreateNote {
    pub notebook_id: Option<String>,
    pub title: Option<String>,
    pub body: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteUpdateNote {
    pub id: String,
    pub notebook_id: Option<String>,
    pub title: Option<String>,
    pub body: String,
    pub is_pinned: Option<bool>,
    pub tag_ids: Option<Vec<String>>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteCreateTag {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct NoteAddAttachment {
    pub note_id: String,
    pub file_path: String,
    pub display_name: Option<String>,
}

fn conn<'a>(
    state: &'a tauri::State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    (*state).0.lock().map_err(|e| e.to_string())
}

fn now_iso() -> String {
    Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn norm_optional_text(raw: &Option<String>) -> Option<String> {
    raw.as_ref()
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
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

fn bool_from_sql(v: i32) -> bool {
    v != 0
}

fn bool_to_sql(v: bool) -> i32 {
    if v { 1 } else { 0 }
}

fn notebook_exists(conn: &Connection, id: &str) -> Result<bool, String> {
    let n: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM note_notebook WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

fn note_exists(conn: &Connection, id: &str) -> Result<bool, String> {
    let n: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM note WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    Ok(n > 0)
}

fn load_tags_for_note(conn: &Connection, note_id: &str) -> Result<Vec<NoteTagDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.name, t.created_at
             FROM note_tag t
             INNER JOIN note_tag_link l ON l.tag_id = t.id
             WHERE l.note_id = ?1
             ORDER BY t.name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![note_id], |row| {
            Ok(NoteTagDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

fn load_attachments_for_note(conn: &Connection, note_id: &str) -> Result<Vec<NoteAttachmentDto>, String> {
    let mut stmt = conn
        .prepare(
            "SELECT id, note_id, file_path, display_name, sort_order, created_at
             FROM note_attachment WHERE note_id = ?1
             ORDER BY sort_order ASC, created_at ASC, id ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![note_id], |row| {
            Ok(NoteAttachmentDto {
                id: row.get(0)?,
                note_id: row.get(1)?,
                file_path: row.get(2)?,
                display_name: row.get(3)?,
                sort_order: row.get(4)?,
                created_at: row.get(5)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

fn sync_note_tags(conn: &Connection, note_id: &str, tag_ids: &[String]) -> Result<(), String> {
    conn.execute("DELETE FROM note_tag_link WHERE note_id = ?1", params![note_id])
        .map_err(|e| e.to_string())?;
    for tag_id in tag_ids {
        if tag_id.is_empty() {
            continue;
        }
        let n: i32 = conn
            .query_row(
                "SELECT COUNT(*) FROM note_tag WHERE id = ?1",
                params![tag_id],
                |row| row.get(0),
            )
            .map_err(|e| e.to_string())?;
        if n == 0 {
            return Err("标签不存在".into());
        }
        conn.execute(
            "INSERT OR IGNORE INTO note_tag_link (note_id, tag_id) VALUES (?1, ?2)",
            params![note_id, tag_id],
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(())
}

#[tauri::command]
pub fn note_list_notebooks(state: tauri::State<'_, DbState>) -> Result<Vec<NoteNotebookDto>, String> {
    let conn = conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, sort_order, is_pinned, created_at, updated_at
             FROM note_notebook
             ORDER BY is_pinned DESC, sort_order ASC, name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(NoteNotebookDto {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get(3)?,
                is_pinned: bool_from_sql(row.get(4)?),
                created_at: row.get(5)?,
                updated_at: row.get(6)?,
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
pub fn note_create_notebook(
    state: tauri::State<'_, DbState>,
    payload: NoteCreateNotebook,
) -> Result<NoteNotebookDto, String> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("笔记本名称不能为空".into());
    }
    if name.len() > 64 {
        return Err("笔记本名称过长".into());
    }
    let conn = conn(&state)?;
    let parent_id = norm_optional_text(&payload.parent_id);
    if let Some(ref pid) = parent_id {
        if !notebook_exists(&conn, pid)? {
            return Err("父笔记本不存在".into());
        }
    }
    let max_so: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM note_notebook WHERE IFNULL(parent_id, '') = IFNULL(?1, '')",
            params![parent_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO note_notebook (id, name, parent_id, sort_order, is_pinned, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6)",
        params![id, name, parent_id, max_so + 1, ts, ts],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    note_list_notebooks(state)?
        .into_iter()
        .find(|n| n.id == id)
        .ok_or_else(|| "写入后读取失败".into())
}

#[tauri::command]
pub fn note_update_notebook(
    state: tauri::State<'_, DbState>,
    payload: NoteUpdateNotebook,
) -> Result<NoteNotebookDto, String> {
    if payload.id.is_empty() {
        return Err("笔记本 id 无效".into());
    }
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("笔记本名称不能为空".into());
    }
    if name.len() > 64 {
        return Err("笔记本名称过长".into());
    }
    let conn = conn(&state)?;
    if !notebook_exists(&conn, &payload.id)? {
        return Err("笔记本不存在".into());
    }
    let parent_id = norm_optional_text(&payload.parent_id);
    if parent_id.as_deref() == Some(payload.id.as_str()) {
        return Err("不能将笔记本设为自己的子级".into());
    }
    if let Some(ref pid) = parent_id {
        if !notebook_exists(&conn, pid)? {
            return Err("父笔记本不存在".into());
        }
    }
    let is_pinned = payload.is_pinned.unwrap_or(false);
    let ts = now_iso();
    conn.execute(
        "UPDATE note_notebook SET name = ?1, parent_id = ?2, is_pinned = ?3, updated_at = ?4 WHERE id = ?5",
        params![name, parent_id, bool_to_sql(is_pinned), ts, payload.id],
    )
    .map_err(|e| e.to_string())?;
    let rid = payload.id.clone();
    drop(conn);
    note_list_notebooks(state)?
        .into_iter()
        .find(|n| n.id == rid)
        .ok_or_else(|| "笔记本不存在".into())
}

#[tauri::command]
pub fn note_delete_notebook(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    if id.is_empty() {
        return Err("笔记本 id 无效".into());
    }
    let conn = conn(&state)?;
    let parent_id: Option<String> = conn
        .query_row(
            "SELECT parent_id FROM note_notebook WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| "笔记本不存在".to_string())?;
    conn.execute(
        "UPDATE note_notebook SET parent_id = ?1 WHERE parent_id = ?2",
        params![parent_id, id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute(
        "UPDATE note SET notebook_id = NULL WHERE notebook_id = ?1",
        params![id],
    )
    .map_err(|e| e.to_string())?;
    conn.execute("DELETE FROM note_notebook WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn note_list_notes(
    state: tauri::State<'_, DbState>,
    filter: NoteListFilter,
) -> Result<Vec<NoteListItem>, String> {
    let conn = conn(&state)?;
    let mut sql = String::from(
        "SELECT id, notebook_id, title, body, is_pinned, updated_at
         FROM note WHERE 1=1",
    );
    let mut pvec: Vec<String> = Vec::new();

    match filter.notebook_id {
        None => {}
        Some(None) => {
            sql.push_str(" AND notebook_id IS NULL");
        }
        Some(Some(ref nid)) if !nid.is_empty() => {
            sql.push_str(" AND notebook_id = ?");
            pvec.push(nid.clone());
        }
        Some(Some(_)) => {}
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

    sql.push_str(" ORDER BY is_pinned DESC, updated_at DESC, sort_order ASC, created_at DESC, id DESC");

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
            Ok(NoteListItem {
                id: row.get(0)?,
                notebook_id: row.get(1)?,
                title,
                preview,
                is_pinned: bool_from_sql(row.get(4)?),
                updated_at: row.get(5)?,
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
pub fn note_get_note(state: tauri::State<'_, DbState>, id: String) -> Result<Option<NoteDto>, String> {
    if id.is_empty() {
        return Ok(None);
    }
    let conn = conn(&state)?;
    let row = conn.query_row(
        "SELECT id, notebook_id, title, body, is_pinned, sort_order, created_at, updated_at
         FROM note WHERE id = ?1",
        params![id],
        |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<String>>(2)?,
                row.get::<_, String>(3)?,
                bool_from_sql(row.get(4)?),
                row.get::<_, i32>(5)?,
                row.get::<_, Option<String>>(6)?,
                row.get::<_, Option<String>>(7)?,
            ))
        },
    );
    match row {
        Ok((id, notebook_id, title, body, is_pinned, sort_order, created_at, updated_at)) => {
            let tags = load_tags_for_note(&conn, &id)?;
            let attachments = load_attachments_for_note(&conn, &id)?;
            Ok(Some(NoteDto {
                id,
                notebook_id,
                title,
                body,
                is_pinned,
                sort_order,
                tags,
                attachments,
                created_at,
                updated_at,
            }))
        }
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(e.to_string()),
    }
}

#[tauri::command]
pub fn note_create_note(
    state: tauri::State<'_, DbState>,
    payload: NoteCreateNote,
) -> Result<NoteDto, String> {
    let conn = conn(&state)?;
    let notebook_id = norm_optional_text(&payload.notebook_id);
    if let Some(ref nid) = notebook_id {
        if !notebook_exists(&conn, nid)? {
            return Err("笔记本不存在".into());
        }
    }
    let title = norm_optional_text(&payload.title);
    let body = payload.body.unwrap_or_default();
    let max_so: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM note WHERE IFNULL(notebook_id, '') = IFNULL(?1, '')",
            params![notebook_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO note (id, notebook_id, title, body, is_pinned, sort_order, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, 0, ?5, ?6, ?7)",
        params![id, notebook_id, title, body, max_so + 1, ts, ts],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    note_get_note(state, id)?.ok_or_else(|| "写入后读取失败".into())
}

#[tauri::command]
pub fn note_update_note(
    state: tauri::State<'_, DbState>,
    payload: NoteUpdateNote,
) -> Result<NoteDto, String> {
    if payload.id.is_empty() {
        return Err("笔记 id 无效".into());
    }
    let conn = conn(&state)?;
    if !note_exists(&conn, &payload.id)? {
        return Err("笔记不存在".into());
    }
    let notebook_id = norm_optional_text(&payload.notebook_id);
    if let Some(ref nid) = notebook_id {
        if !notebook_exists(&conn, nid)? {
            return Err("笔记本不存在".into());
        }
    }
    let title = norm_optional_text(&payload.title);
    let is_pinned = match payload.is_pinned {
        Some(v) => v,
        None => {
            let current: i32 = conn
                .query_row(
                    "SELECT is_pinned FROM note WHERE id = ?1",
                    params![payload.id],
                    |row| row.get(0),
                )
                .map_err(|e| e.to_string())?;
            bool_from_sql(current)
        }
    };
    let ts = now_iso();
    conn.execute(
        "UPDATE note SET notebook_id = ?1, title = ?2, body = ?3, is_pinned = ?4, updated_at = ?5 WHERE id = ?6",
        params![
            notebook_id,
            title,
            payload.body,
            bool_to_sql(is_pinned),
            ts,
            payload.id
        ],
    )
    .map_err(|e| e.to_string())?;
    if let Some(tag_ids) = payload.tag_ids {
        sync_note_tags(&conn, &payload.id, &tag_ids)?;
    }
    let rid = payload.id.clone();
    drop(conn);
    note_get_note(state, rid)?.ok_or_else(|| "笔记不存在".into())
}

#[tauri::command]
pub fn note_delete_note(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    if id.is_empty() {
        return Err("笔记 id 无效".into());
    }
    let conn = conn(&state)?;
    conn.execute("DELETE FROM note WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

#[tauri::command]
pub fn note_list_tags(state: tauri::State<'_, DbState>) -> Result<Vec<NoteTagDto>, String> {
    let conn = conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, created_at FROM note_tag ORDER BY name COLLATE NOCASE ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(NoteTagDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
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
pub fn note_create_tag(
    state: tauri::State<'_, DbState>,
    payload: NoteCreateTag,
) -> Result<NoteTagDto, String> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("标签名称不能为空".into());
    }
    if name.len() > 32 {
        return Err("标签名称过长".into());
    }
    let conn = conn(&state)?;
    if let Ok(existing) = conn.query_row(
        "SELECT id, name, created_at FROM note_tag WHERE name = ?1 COLLATE NOCASE",
        params![name],
        |row| {
            Ok(NoteTagDto {
                id: row.get(0)?,
                name: row.get(1)?,
                created_at: row.get(2)?,
            })
        },
    ) {
        return Ok(existing);
    }
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO note_tag (id, name, created_at) VALUES (?1, ?2, ?3)",
        params![id, name, ts],
    )
    .map_err(|e| e.to_string())?;
    Ok(NoteTagDto {
        id,
        name: name.to_string(),
        created_at: Some(ts),
    })
}

#[tauri::command]
pub fn note_add_attachment(
    state: tauri::State<'_, DbState>,
    payload: NoteAddAttachment,
) -> Result<NoteAttachmentDto, String> {
    if payload.note_id.is_empty() {
        return Err("笔记 id 无效".into());
    }
    let file_path = payload.file_path.trim();
    if file_path.is_empty() {
        return Err("文件路径不能为空".into());
    }
    let conn = conn(&state)?;
    if !note_exists(&conn, &payload.note_id)? {
        return Err("笔记不存在".into());
    }
    let display_name = norm_optional_text(&payload.display_name);
    let max_so: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM note_attachment WHERE note_id = ?1",
            params![payload.note_id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO note_attachment (id, note_id, file_path, display_name, sort_order, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![id, payload.note_id, file_path, display_name, max_so + 1, ts],
    )
    .map_err(|e| e.to_string())?;
    Ok(NoteAttachmentDto {
        id,
        note_id: payload.note_id,
        file_path: file_path.to_string(),
        display_name,
        sort_order: max_so + 1,
        created_at: Some(ts),
    })
}

#[tauri::command]
pub fn note_delete_attachment(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    if id.is_empty() {
        return Err("附件 id 无效".into());
    }
    let conn = conn(&state)?;
    conn.execute("DELETE FROM note_attachment WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}
