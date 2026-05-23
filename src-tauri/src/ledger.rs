use crate::DbState;
use chrono::{Datelike, Duration, Local, NaiveDate};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerCategoryDto {
    pub id: String,
    pub name: String,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerTransactionDto {
    pub id: String,
    pub amount: String,
    pub currency: String,
    pub direction: String,
    pub category_id: Option<String>,
    pub category_name: Option<String>,
    pub occurred_on: String,
    pub note: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerListFilter {
    pub date_from: Option<String>,
    pub date_to: Option<String>,
    /// `all` | `in` | `out`
    pub direction: Option<String>,
    pub category_id: Option<String>,
    /// `all` | `cny` | `jpy`
    pub currency: Option<String>,
    pub search: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerCreateCategory {
    pub name: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerCreateTransaction {
    pub amount: String,
    pub currency: String,
    pub direction: String,
    pub category_id: Option<String>,
    pub occurred_on: String,
    pub note: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerUpdateTransaction {
    pub id: String,
    pub amount: String,
    pub currency: String,
    pub direction: String,
    pub category_id: Option<String>,
    pub occurred_on: String,
    pub note: Option<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerStatQuery {
    pub date_from: String,
    pub date_to: String,
    pub direction: String,
    pub category_id: Option<String>,
    /// 图表用；`all` 时返回空序列，由前端提示选币种
    pub currency: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DailySeriesRow {
    pub date: String,
    pub out_sum: f64,
    pub in_sum: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CategoryExpenseRow {
    pub name: String,
    pub total: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerStatistics {
    pub daily: Vec<DailySeriesRow>,
    pub category_expense: Vec<CategoryExpenseRow>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerRollupCard {
    pub period: String,
    pub label: String,
    pub cny_income: String,
    pub cny_expense: String,
    pub jpy_income: String,
    pub jpy_expense: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LedgerRollups {
    pub cards: Vec<LedgerRollupCard>,
}

fn conn<'a>(
    state: &'a tauri::State<'_, DbState>,
) -> Result<std::sync::MutexGuard<'a, Connection>, String> {
    (*state).0.lock().map_err(|e| e.to_string())
}

fn now_iso() -> String {
    Local::now().to_rfc3339_opts(chrono::SecondsFormat::Millis, true)
}

fn norm_currency(raw: &str) -> Result<String, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "cny" => Ok("cny".into()),
        "jpy" => Ok("jpy".into()),
        _ => Err("币种仅支持 CNY / JPY".into()),
    }
}

fn norm_direction(raw: &str) -> Result<String, String> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "in" => Ok("in".into()),
        "out" => Ok("out".into()),
        _ => Err("方向须为 in（收入）或 out（支出）".into()),
    }
}

fn normalize_amount(raw: &str) -> Result<String, String> {
    let t = raw.trim();
    if t.is_empty() {
        return Err("金额不能为空".into());
    }
    let v: f64 = t.parse().map_err(|_| "金额须为数字".to_string())?;
    if v < 0.0 {
        return Err("金额不能为负数".into());
    }
    Ok(format!("{:.2}", v))
}

fn parse_iso_date(s: &str) -> Result<NaiveDate, String> {
    NaiveDate::parse_from_str(s.trim(), "%Y-%m-%d").map_err(|_| "发生日须为 YYYY-MM-DD".into())
}

fn verify_category(conn: &Connection, id: &str) -> Result<(), String> {
    let n: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ledger_category WHERE id = ?1",
            params![id],
            |row| row.get(0),
        )
        .map_err(|_| "分类不存在".to_string())?;
    if n == 0 {
        return Err("分类不存在".into());
    }
    Ok(())
}

#[tauri::command]
pub fn ledger_list_categories(state: tauri::State<'_, DbState>) -> Result<Vec<LedgerCategoryDto>, String> {
    let conn = conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT id, name, parent_id, sort_order, created_at, updated_at
             FROM ledger_category ORDER BY sort_order ASC, name ASC",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map([], |row| {
            Ok(LedgerCategoryDto {
                id: row.get(0)?,
                name: row.get(1)?,
                parent_id: row.get(2)?,
                sort_order: row.get(3)?,
                created_at: row.get(4)?,
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
pub fn ledger_create_category(
    state: tauri::State<'_, DbState>,
    payload: LedgerCreateCategory,
) -> Result<LedgerCategoryDto, String> {
    let name = payload.name.trim();
    if name.is_empty() {
        return Err("分类名称不能为空".into());
    }
    if name.len() > 64 {
        return Err("分类名称过长".into());
    }
    let conn = conn(&state)?;
    let dup: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ledger_category WHERE name = ?1 COLLATE NOCASE",
            params![name],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if dup > 0 {
        return Err("已存在同名分类".into());
    }
    let max_so: i32 = conn
        .query_row(
            "SELECT COALESCE(MAX(sort_order), 0) FROM ledger_category",
            [],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    conn.execute(
        "INSERT INTO ledger_category (id, name, parent_id, sort_order, created_at, updated_at)
         VALUES (?1, ?2, NULL, ?3, ?4, ?5)",
        params![id, name, max_so + 1, ts, ts],
    )
    .map_err(|e| e.to_string())?;
    Ok(LedgerCategoryDto {
        id,
        name: name.to_string(),
        parent_id: None,
        sort_order: max_so + 1,
        created_at: Some(ts.clone()),
        updated_at: Some(ts),
    })
}

#[tauri::command]
pub fn ledger_delete_category(state: tauri::State<'_, DbState>, id: String) -> Result<(), String> {
    if id.is_empty() {
        return Err("分类 id 无效".into());
    }
    let conn = conn(&state)?;
    conn.execute("DELETE FROM ledger_category WHERE id = ?1", params![id])
        .map_err(|e| e.to_string())?;
    Ok(())
}

fn map_tx_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<LedgerTransactionDto> {
    Ok(LedgerTransactionDto {
        id: row.get(0)?,
        amount: row.get(1)?,
        currency: row.get(2)?,
        direction: row.get(3)?,
        category_id: row.get(4)?,
        occurred_on: row.get(5)?,
        note: row.get(6)?,
        created_at: row.get(7)?,
        updated_at: row.get(8)?,
        category_name: row.get(9)?,
    })
}

#[tauri::command]
pub fn ledger_list_transactions(
    state: tauri::State<'_, DbState>,
    filter: LedgerListFilter,
) -> Result<Vec<LedgerTransactionDto>, String> {
    let conn = conn(&state)?;
    let mut sql = String::from(
        "SELECT t.id, t.amount, t.currency, t.direction, t.category_id, t.occurred_on, t.note, t.created_at, t.updated_at,
            c.name AS category_name
         FROM ledger_transaction t
         LEFT JOIN ledger_category c ON t.category_id = c.id
         WHERE 1=1",
    );
    let mut pvec: Vec<String> = Vec::new();

    if let Some(ref d) = filter.date_from {
        if !d.is_empty() {
            sql.push_str(" AND t.occurred_on >= ?");
            pvec.push(d.clone());
        }
    }
    if let Some(ref d) = filter.date_to {
        if !d.is_empty() {
            sql.push_str(" AND t.occurred_on <= ?");
            pvec.push(d.clone());
        }
    }
    if let Some(ref dir) = filter.direction {
        match dir.as_str() {
            "in" | "out" => {
                sql.push_str(" AND t.direction = ?");
                pvec.push(dir.clone());
            }
            "all" | "" => {}
            _ => return Err("direction 参数无效".into()),
        }
    }
    if let Some(ref cid) = filter.category_id {
        if !cid.is_empty() && cid != "all" {
            sql.push_str(" AND t.category_id = ?");
            pvec.push(cid.clone());
        }
    }
    if let Some(ref cur) = filter.currency {
        match cur.as_str() {
            "cny" | "jpy" => {
                sql.push_str(" AND t.currency = ?");
                pvec.push(cur.clone());
            }
            "all" | "" => {}
            _ => return Err("currency 参数无效".into()),
        }
    }
    if let Some(ref s) = filter.search {
        let t = s.trim();
        if !t.is_empty() {
            let pat = format!("%{}%", t.replace('%', "\\%").replace('_', "\\_"));
            sql.push_str(" AND (IFNULL(t.note, '') LIKE ? ESCAPE '\\' OR t.amount LIKE ? ESCAPE '\\')");
            pvec.push(pat.clone());
            pvec.push(pat);
        }
    }

    sql.push_str(" ORDER BY t.occurred_on DESC, t.created_at DESC, t.id DESC");

    let mut stmt = conn.prepare(&sql).map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(rusqlite::params_from_iter(pvec.iter()), map_tx_row)
        .map_err(|e| e.to_string())?;
    let mut out = Vec::new();
    for r in rows {
        out.push(r.map_err(|e| e.to_string())?);
    }
    Ok(out)
}

#[tauri::command]
pub fn ledger_get_transaction(state: tauri::State<'_, DbState>, id: String) -> Result<Option<LedgerTransactionDto>, String> {
    if id.is_empty() {
        return Ok(None);
    }
    let conn = conn(&state)?;
    let mut stmt = conn
        .prepare(
            "SELECT t.id, t.amount, t.currency, t.direction, t.category_id, t.occurred_on, t.note, t.created_at, t.updated_at,
                c.name AS category_name
             FROM ledger_transaction t
             LEFT JOIN ledger_category c ON t.category_id = c.id
             WHERE t.id = ?1",
        )
        .map_err(|e| e.to_string())?;
    let mut rows = stmt.query_map(params![id], map_tx_row).map_err(|e| e.to_string())?;
    match rows.next() {
        Some(r) => Ok(Some(r.map_err(|e| e.to_string())?)),
        None => Ok(None),
    }
}

#[tauri::command]
pub fn ledger_create_transaction(
    state: tauri::State<'_, DbState>,
    payload: LedgerCreateTransaction,
) -> Result<LedgerTransactionDto, String> {
    let amount = normalize_amount(&payload.amount)?;
    let currency = norm_currency(&payload.currency)?;
    let direction = norm_direction(&payload.direction)?;
    parse_iso_date(&payload.occurred_on)?;
    let conn = conn(&state)?;
    let cat = match &payload.category_id {
        Some(cid) if !cid.trim().is_empty() => {
            verify_category(&conn, cid.trim())?;
            Some(cid.trim().to_string())
        }
        _ => None,
    };
    let id = Uuid::new_v4().to_string();
    let ts = now_iso();
    let note = payload.note.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string());
    conn.execute(
        "INSERT INTO ledger_transaction (id, amount, currency, direction, category_id, occurred_on, note, created_at, updated_at)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
        params![
            id,
            amount,
            currency,
            direction,
            cat,
            payload.occurred_on.trim(),
            note,
            ts,
            ts
        ],
    )
    .map_err(|e| e.to_string())?;
    drop(conn);
    ledger_get_transaction(state, id)?.ok_or_else(|| "写入后读取失败".into())
}

#[tauri::command]
pub fn ledger_update_transaction(
    state: tauri::State<'_, DbState>,
    payload: LedgerUpdateTransaction,
) -> Result<LedgerTransactionDto, String> {
    let amount = normalize_amount(&payload.amount)?;
    let currency = norm_currency(&payload.currency)?;
    let direction = norm_direction(&payload.direction)?;
    parse_iso_date(&payload.occurred_on)?;
    let conn = conn(&state)?;
    let cat = match &payload.category_id {
        Some(cid) if !cid.trim().is_empty() => {
            verify_category(&conn, cid.trim())?;
            Some(cid.trim().to_string())
        }
        _ => None,
    };
    let n: i32 = conn
        .query_row(
            "SELECT COUNT(*) FROM ledger_transaction WHERE id = ?1",
            params![payload.id],
            |row| row.get(0),
        )
        .map_err(|e| e.to_string())?;
    if n == 0 {
        return Err("流水不存在".into());
    }
    let ts = now_iso();
    let note = payload.note.as_ref().map(|s| s.trim()).filter(|s| !s.is_empty()).map(|s| s.to_string());
    conn.execute(
        "UPDATE ledger_transaction SET amount = ?1, currency = ?2, direction = ?3, category_id = ?4,
         occurred_on = ?5, note = ?6, updated_at = ?7 WHERE id = ?8",
        params![
            amount,
            currency,
            direction,
            cat,
            payload.occurred_on.trim(),
            note,
            ts,
            payload.id
        ],
    )
    .map_err(|e| e.to_string())?;
    let rid = payload.id.clone();
    drop(conn);
    ledger_get_transaction(state, rid)?.ok_or_else(|| "流水不存在".into())
}

#[tauri::command]
pub fn ledger_delete_transactions(state: tauri::State<'_, DbState>, ids: Vec<String>) -> Result<u32, String> {
    if ids.is_empty() {
        return Ok(0);
    }
    let conn = conn(&state)?;
    let placeholders: String = (0..ids.len()).map(|_| "?").collect::<Vec<_>>().join(",");
    let sql = format!("DELETE FROM ledger_transaction WHERE id IN ({placeholders})");
    let n = conn.execute(&sql, rusqlite::params_from_iter(ids.iter())).map_err(|e| e.to_string())?;
    Ok(n as u32)
}

fn rollup_for_range(
    conn: &Connection,
    from: &str,
    to: &str,
    period: &str,
    label: &str,
) -> Result<LedgerRollupCard, String> {
    let mut stmt = conn
        .prepare(
            "SELECT currency, direction, COALESCE(SUM(CAST(amount AS REAL)), 0)
             FROM ledger_transaction
             WHERE occurred_on >= ?1 AND occurred_on <= ?2
             GROUP BY currency, direction",
        )
        .map_err(|e| e.to_string())?;
    let rows = stmt
        .query_map(params![from, to], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, f64>(2)?,
            ))
        })
        .map_err(|e| e.to_string())?;
    let mut cny_in = 0f64;
    let mut cny_out = 0f64;
    let mut jpy_in = 0f64;
    let mut jpy_out = 0f64;
    for r in rows {
        let (cur, dir, sum) = r.map_err(|e| e.to_string())?;
        match (cur.as_str(), dir.as_str()) {
            ("cny", "in") => cny_in = sum,
            ("cny", "out") => cny_out = sum,
            ("jpy", "in") => jpy_in = sum,
            ("jpy", "out") => jpy_out = sum,
            _ => {}
        }
    }
    Ok(LedgerRollupCard {
        period: period.into(),
        label: label.into(),
        cny_income: format!("{:.2}", cny_in),
        cny_expense: format!("{:.2}", cny_out),
        jpy_income: format!("{:.2}", jpy_in),
        jpy_expense: format!("{:.2}", jpy_out),
    })
}

#[tauri::command]
pub fn ledger_get_period_rollups(state: tauri::State<'_, DbState>) -> Result<LedgerRollups, String> {
    let conn = conn(&state)?;
    let today = Local::now().date_naive();
    let today_s = today.format("%Y-%m-%d").to_string();

    let wd_mo = today.weekday().num_days_from_monday();
    let monday = today - Duration::days(wd_mo as i64);
    let sunday = monday + Duration::days(6);
    let w_from = monday.format("%Y-%m-%d").to_string();
    let w_to = sunday.format("%Y-%m-%d").to_string();

    let m_first = today.with_day(1).ok_or("月份日期无效".to_string())?;
    let (ny, nm) = if today.month() == 12 {
        (today.year() + 1, 1)
    } else {
        (today.year(), today.month() + 1)
    };
    let m_next_first = NaiveDate::from_ymd_opt(ny, nm, 1).ok_or("日期无效".to_string())?;
    let m_last = m_next_first - Duration::days(1);
    let m_from = m_first.format("%Y-%m-%d").to_string();
    let m_to = m_last.format("%Y-%m-%d").to_string();

    let y_first = NaiveDate::from_ymd_opt(today.year(), 1, 1).ok_or("日期无效".to_string())?;
    let y_last = NaiveDate::from_ymd_opt(today.year(), 12, 31).ok_or("日期无效".to_string())?;
    let y_from = y_first.format("%Y-%m-%d").to_string();
    let y_to = y_last.format("%Y-%m-%d").to_string();

    let cards = vec![
        rollup_for_range(&conn, &today_s, &today_s, "today", "今日")?,
        rollup_for_range(&conn, &w_from, &w_to, "week", "本周")?,
        rollup_for_range(&conn, &m_from, &m_to, "month", "本月")?,
        rollup_for_range(&conn, &y_from, &y_to, "year", "本年")?,
    ];
    Ok(LedgerRollups { cards })
}

#[tauri::command]
pub fn ledger_get_statistics(
    state: tauri::State<'_, DbState>,
    query: LedgerStatQuery,
) -> Result<LedgerStatistics, String> {
    parse_iso_date(&query.date_from)?;
    parse_iso_date(&query.date_to)?;
    let cur_lc = query.currency.trim().to_ascii_lowercase();
    if cur_lc != "cny" && cur_lc != "jpy" {
       return Ok(LedgerStatistics {
            daily: vec![],
            category_expense: vec![],
        });
    }

    let conn = conn(&state)?;
    let mut daily_sql = String::from(
        "SELECT t.occurred_on AS d,
            COALESCE(SUM(CASE WHEN t.direction = 'out' THEN CAST(t.amount AS REAL) ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN t.direction = 'in' THEN CAST(t.amount AS REAL) ELSE 0 END), 0)
         FROM ledger_transaction t
         WHERE t.occurred_on >= ?1 AND t.occurred_on <= ?2 AND t.currency = ?3",
    );
    let mut daily_params: Vec<String> = vec![
        query.date_from.clone(),
        query.date_to.clone(),
        cur_lc.clone(),
    ];
    match query.direction.as_str() {
        "in" => {
            daily_sql.push_str(" AND t.direction = 'in'");
        }
        "out" => {
            daily_sql.push_str(" AND t.direction = 'out'");
        }
        "all" | "" => {}
        _ => return Err("统计 direction 无效".into()),
    }
    if let Some(ref cid) = query.category_id {
        if !cid.is_empty() && cid != "all" {
            daily_sql.push_str(" AND t.category_id = ?");
            daily_params.push(cid.clone());
        }
    }
    daily_sql.push_str(" GROUP BY t.occurred_on ORDER BY t.occurred_on ASC");

    let mut stmt = conn.prepare(&daily_sql).map_err(|e| e.to_string())?;
    let drows = stmt
        .query_map(rusqlite::params_from_iter(daily_params.iter()), |row| {
            Ok(DailySeriesRow {
                date: row.get(0)?,
                out_sum: row.get(1)?,
                in_sum: row.get(2)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut daily = Vec::new();
    for r in drows {
        daily.push(r.map_err(|e| e.to_string())?);
    }

    let cat_dir = match query.direction.as_str() {
        "in" => "in",
        _ => "out",
    };
    let mut cat_sql = String::from(
        "SELECT COALESCE(c.name, '未分类') AS nm,
            COALESCE(SUM(CAST(t.amount AS REAL)), 0) AS total
         FROM ledger_transaction t
         LEFT JOIN ledger_category c ON t.category_id = c.id
         WHERE t.occurred_on >= ?1 AND t.occurred_on <= ?2 AND t.currency = ?3 AND t.direction = ?4",
    );
    let mut cat_params: Vec<String> = vec![
        query.date_from.clone(),
        query.date_to.clone(),
        cur_lc.clone(),
        cat_dir.into(),
    ];
    if let Some(ref cid) = query.category_id {
        if !cid.is_empty() && cid != "all" {
            cat_sql.push_str(" AND t.category_id = ?");
            cat_params.push(cid.clone());
        }
    }
    cat_sql.push_str(" GROUP BY nm HAVING total > 0 ORDER BY total DESC LIMIT 12");

    let mut cstmt = conn.prepare(&cat_sql).map_err(|e| e.to_string())?;
    let crows = cstmt
        .query_map(rusqlite::params_from_iter(cat_params.iter()), |row| {
            Ok(CategoryExpenseRow {
                name: row.get(0)?,
                total: row.get(1)?,
            })
        })
        .map_err(|e| e.to_string())?;
    let mut category_expense = Vec::new();
    for r in crows {
        category_expense.push(r.map_err(|e| e.to_string())?);
    }

    Ok(LedgerStatistics {
        daily,
        category_expense,
    })
}
