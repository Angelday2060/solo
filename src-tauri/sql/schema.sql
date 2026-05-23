-- solo SQLite schema（与 docs/实体类设计/记账模块实体.md 对齐）

PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS ledger_category (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES ledger_category (id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS ledger_transaction (
    id TEXT PRIMARY KEY NOT NULL,
    amount TEXT NOT NULL,
    currency TEXT NOT NULL,
    direction TEXT NOT NULL,
    category_id TEXT REFERENCES ledger_category (id) ON DELETE SET NULL,
    occurred_on TEXT NOT NULL,
    note TEXT,
    created_at TEXT,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_ledger_tx_occurred ON ledger_transaction (occurred_on);
CREATE INDEX IF NOT EXISTS idx_ledger_tx_category ON ledger_transaction (category_id);

-- 日记（与 docs/实体类设计/日记模块实体.md 对齐）
CREATE TABLE IF NOT EXISTS diary_entry (
    id TEXT PRIMARY KEY NOT NULL,
    entry_date TEXT NOT NULL,
    title TEXT,
    body TEXT NOT NULL DEFAULT '',
    weather TEXT,
    mood TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_diary_entry_date ON diary_entry (entry_date);

-- 笔记（与 docs/实体类设计/笔记模块实体.md 对齐）
CREATE TABLE IF NOT EXISTS note_notebook (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL,
    parent_id TEXT REFERENCES note_notebook (id) ON DELETE SET NULL,
    sort_order INTEGER NOT NULL DEFAULT 0,
    is_pinned INTEGER NOT NULL DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS note (
    id TEXT PRIMARY KEY NOT NULL,
    notebook_id TEXT REFERENCES note_notebook (id) ON DELETE SET NULL,
    title TEXT,
    body TEXT NOT NULL DEFAULT '',
    is_pinned INTEGER NOT NULL DEFAULT 0,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT,
    updated_at TEXT
);

CREATE TABLE IF NOT EXISTS note_tag (
    id TEXT PRIMARY KEY NOT NULL,
    name TEXT NOT NULL UNIQUE COLLATE NOCASE,
    created_at TEXT
);

CREATE TABLE IF NOT EXISTS note_tag_link (
    note_id TEXT NOT NULL REFERENCES note (id) ON DELETE CASCADE,
    tag_id TEXT NOT NULL REFERENCES note_tag (id) ON DELETE CASCADE,
    PRIMARY KEY (note_id, tag_id)
);

CREATE TABLE IF NOT EXISTS note_attachment (
    id TEXT PRIMARY KEY NOT NULL,
    note_id TEXT NOT NULL REFERENCES note (id) ON DELETE CASCADE,
    file_path TEXT NOT NULL,
    display_name TEXT,
    sort_order INTEGER NOT NULL DEFAULT 0,
    created_at TEXT
);

CREATE INDEX IF NOT EXISTS idx_note_notebook ON note (notebook_id);
CREATE INDEX IF NOT EXISTS idx_note_updated ON note (updated_at);
CREATE INDEX IF NOT EXISTS idx_note_attachment_note ON note_attachment (note_id);
CREATE INDEX IF NOT EXISTS idx_note_tag_link_tag ON note_tag_link (tag_id);
