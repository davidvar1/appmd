pub const MIGRATIONS: &[&str] = &[
    // v1: Tablas iniciales
    "CREATE TABLE IF NOT EXISTS pages (
        id          TEXT PRIMARY KEY,
        title       TEXT NOT NULL DEFAULT 'Untitled',
        icon        TEXT,
        cover       TEXT,
        parent_id   TEXT REFERENCES pages(id) ON DELETE CASCADE,
        sort_order  INTEGER NOT NULL DEFAULT 0,
        is_database INTEGER NOT NULL DEFAULT 0,
        created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    );",
    "CREATE TABLE IF NOT EXISTS blocks (
        id              TEXT PRIMARY KEY,
        page_id         TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
        parent_block_id TEXT REFERENCES blocks(id) ON DELETE CASCADE,
        block_type      TEXT NOT NULL DEFAULT 'text',
        content         TEXT NOT NULL DEFAULT '',
        props           TEXT NOT NULL DEFAULT '{}',
        sort_order      INTEGER NOT NULL DEFAULT 0,
        depth           INTEGER NOT NULL DEFAULT 0,
        created_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        updated_at      TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    );",
    "CREATE INDEX IF NOT EXISTS idx_blocks_page ON blocks(page_id);",
    "CREATE INDEX IF NOT EXISTS idx_blocks_parent ON blocks(parent_block_id);",
    "CREATE TABLE IF NOT EXISTS databases (
        id          TEXT PRIMARY KEY,
        page_id     TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
        title       TEXT NOT NULL DEFAULT '',
        columns     TEXT NOT NULL DEFAULT '[]',
        created_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now')),
        updated_at  TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
    );",
    "CREATE TABLE IF NOT EXISTS database_rows (
        id          TEXT PRIMARY KEY,
        db_id       TEXT NOT NULL REFERENCES databases(id) ON DELETE CASCADE,
        page_id     TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
        cells       TEXT NOT NULL DEFAULT '{}',
        sort_order  INTEGER NOT NULL DEFAULT 0
    );",
    "CREATE INDEX IF NOT EXISTS idx_db_rows_db ON database_rows(db_id);",
    "CREATE TABLE IF NOT EXISTS database_views (
        id          TEXT PRIMARY KEY,
        db_id       TEXT NOT NULL REFERENCES databases(id) ON DELETE CASCADE,
        name        TEXT NOT NULL DEFAULT 'Grid view',
        view_type   TEXT NOT NULL DEFAULT 'grid',
        options     TEXT NOT NULL DEFAULT '{}'
    );",
    "CREATE TABLE IF NOT EXISTS templates (
        id          TEXT PRIMARY KEY,
        name        TEXT NOT NULL,
        block_json  TEXT NOT NULL
    );",
    "CREATE TABLE IF NOT EXISTS backlinks (
        id              TEXT PRIMARY KEY,
        source_page_id  TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
        target_page_id  TEXT NOT NULL REFERENCES pages(id) ON DELETE CASCADE,
        block_id        TEXT REFERENCES blocks(id) ON DELETE CASCADE,
        snippet         TEXT NOT NULL DEFAULT ''
    );",
    "CREATE INDEX IF NOT EXISTS idx_backlinks_target ON backlinks(target_page_id);",
    "CREATE INDEX IF NOT EXISTS idx_backlinks_source ON backlinks(source_page_id);",
    "CREATE TRIGGER IF NOT EXISTS trg_page_updated AFTER UPDATE ON pages
     BEGIN
         UPDATE pages SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = NEW.id;
     END;",
    "CREATE TRIGGER IF NOT EXISTS trg_block_updated AFTER UPDATE ON blocks
     BEGIN
         UPDATE blocks SET updated_at = strftime('%Y-%m-%dT%H:%M:%fZ','now') WHERE id = NEW.id;
     END;",
    // FTS5 para búsqueda full-text en páginas
    "CREATE VIRTUAL TABLE IF NOT EXISTS pages_fts USING fts5(
        title, content,
        content='pages',
        content_rowid='rowid'
    );",
];
