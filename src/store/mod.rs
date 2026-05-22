pub mod block_types;
pub mod schema;

use block_types::BlockType;
use rusqlite::{params, Connection, Result as SqlResult};
use std::path::Path;
use std::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct Page {
    pub id: String,
    pub title: String,
    pub icon: Option<String>,
    pub cover: Option<String>,
    pub parent_id: Option<String>,
    pub sort_order: i32,
    pub is_database: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct Block {
    pub id: String,
    pub page_id: String,
    pub parent_block_id: Option<String>,
    pub block_type: String,
    pub content: String,
    pub props: String,
    pub sort_order: i32,
    pub depth: i32,
    pub created_at: String,
    pub updated_at: String,
}

impl Block {
    pub fn block_type_enum(&self) -> BlockType {
        BlockType::from_db_string(&self.block_type)
    }

    pub fn set_block_type_enum(&mut self, bt: BlockType) {
        self.block_type = bt.to_db_string().to_string();
    }
}

#[derive(Debug, Clone)]
pub struct Database {
    pub id: String,
    pub page_id: String,
    pub title: String,
    pub columns: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone)]
pub struct DatabaseRow {
    pub id: String,
    pub db_id: String,
    pub page_id: String,
    pub cells: String,
    pub sort_order: i32,
}

#[derive(Debug, Clone)]
pub struct DatabaseView {
    pub id: String,
    pub db_id: String,
    pub name: String,
    pub view_type: String,
    pub options: String,
}

#[derive(Debug, Clone)]
pub struct Template {
    pub id: String,
    pub name: String,
    pub block_json: String,
}

#[derive(Debug, Clone)]
pub struct Backlink {
    pub id: String,
    pub source_page_id: String,
    pub target_page_id: String,
    pub block_id: Option<String>,
    pub snippet: String,
}

#[derive(Debug)]
pub struct SqliteStore {
    conn: Mutex<Connection>,
}

impl SqliteStore {
    pub fn open<P: AsRef<Path>>(path: P) -> SqlResult<Self> {
        let conn = Connection::open(path)?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = SqliteStore {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    pub fn open_in_memory() -> SqlResult<Self> {
        let conn = Connection::open_in_memory()?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")?;
        let store = SqliteStore {
            conn: Mutex::new(conn),
        };
        store.run_migrations()?;
        Ok(store)
    }

    fn run_migrations(&self) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS _migrations (
                version INTEGER PRIMARY KEY,
                applied_at TEXT NOT NULL DEFAULT (strftime('%Y-%m-%dT%H:%M:%fZ','now'))
            );",
        )?;
        let applied: i32 = conn
            .query_row("SELECT COALESCE(MAX(version),0) FROM _migrations", [], |r| r.get(0))
            .unwrap_or(0);
        for (i, migration) in schema::MIGRATIONS.iter().enumerate() {
            let version = (i + 1) as i32;
            if version > applied {
                conn.execute_batch(migration)?;
                conn.execute(
                    "INSERT INTO _migrations (version) VALUES (?1)",
                    params![version],
                )?;
            }
        }
        Ok(())
    }

    fn new_id() -> String {
        Uuid::new_v4().to_string()
    }

    // ── Pages ──

    pub fn create_page(
        &self,
        title: &str,
        icon: Option<&str>,
        parent_id: Option<&str>,
    ) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        let sort: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM pages WHERE parent_id IS ?1",
                params![parent_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO pages (id, title, icon, parent_id, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, title, icon, parent_id, sort],
        )?;
        Ok(id)
    }

    pub fn get_page(&self, id: &str) -> SqlResult<Option<Page>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, icon, cover, parent_id, sort_order, is_database, created_at, updated_at
             FROM pages WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |r| {
            Ok(Page {
                id: r.get(0)?,
                title: r.get(1)?,
                icon: r.get(2)?,
                cover: r.get(3)?,
                parent_id: r.get(4)?,
                sort_order: r.get(5)?,
                is_database: r.get::<_, i32>(6)? != 0,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?;
        match rows.next() {
            Some(Ok(page)) => Ok(Some(page)),
            _ => Ok(None),
        }
    }

    pub fn update_page(
        &self,
        id: &str,
        title: &str,
        icon: Option<&str>,
        cover: Option<&str>,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pages SET title = ?1, icon = ?2, cover = ?3 WHERE id = ?4",
            params![title, icon, cover, id],
        )?;
        Ok(())
    }

    pub fn delete_page(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM pages WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn list_pages(&self, parent_id: Option<&str>) -> SqlResult<Vec<Page>> {
        let conn = self.conn.lock().unwrap();
        if let Some(pid) = parent_id {
            let mut stmt = conn.prepare(
                "SELECT id, title, icon, cover, parent_id, sort_order, is_database, created_at, updated_at
                 FROM pages WHERE parent_id = ?1 ORDER BY sort_order",
            )?;
            let rows = stmt.query_map(params![pid], |r| {
                Ok(Page {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    icon: r.get(2)?,
                    cover: r.get(3)?,
                    parent_id: r.get(4)?,
                    sort_order: r.get(5)?,
                    is_database: r.get::<_, i32>(6)? != 0,
                    created_at: r.get(7)?,
                    updated_at: r.get(8)?,
                })
            })?;
            rows.collect()
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, title, icon, cover, parent_id, sort_order, is_database, created_at, updated_at
                 FROM pages WHERE parent_id IS NULL ORDER BY sort_order",
            )?;
            let rows = stmt.query_map([], |r| {
                Ok(Page {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    icon: r.get(2)?,
                    cover: r.get(3)?,
                    parent_id: r.get(4)?,
                    sort_order: r.get(5)?,
                    is_database: r.get::<_, i32>(6)? != 0,
                    created_at: r.get(7)?,
                    updated_at: r.get(8)?,
                })
            })?;
            rows.collect()
        }
    }

    pub fn move_page(
        &self,
        id: &str,
        new_parent_id: Option<&str>,
        sort_order: i32,
    ) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE pages SET parent_id = ?1, sort_order = ?2 WHERE id = ?3",
            params![new_parent_id, sort_order, id],
        )?;
        Ok(())
    }

    // ── Blocks ──

    pub fn create_block(
        &self,
        page_id: &str,
        block_type: &str,
        content: &str,
        parent_block_id: Option<&str>,
        props: Option<&str>,
    ) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        let sort: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM blocks WHERE page_id = ?1 AND parent_block_id IS ?2",
                params![page_id, parent_block_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        let depth: i32 = if let Some(pid) = parent_block_id {
            conn.query_row(
                "SELECT depth FROM blocks WHERE id = ?1",
                params![pid],
                |r| r.get::<_, i32>(0),
            )
            .unwrap_or(0)
                + 1
        } else {
            0
        };
        conn.execute(
            "INSERT INTO blocks (id, page_id, parent_block_id, block_type, content, props, sort_order, depth)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![id, page_id, parent_block_id, block_type, content, props.unwrap_or("{}"), sort, depth],
        )?;
        Ok(id)
    }

    pub fn get_blocks(&self, page_id: &str) -> SqlResult<Vec<Block>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
             FROM blocks WHERE page_id = ?1 ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![page_id], |r| {
            Ok(Block {
                id: r.get(0)?,
                page_id: r.get(1)?,
                parent_block_id: r.get(2)?,
                block_type: r.get(3)?,
                content: r.get(4)?,
                props: r.get(5)?,
                sort_order: r.get(6)?,
                depth: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
            })
        })?;
        rows.collect()
    }

    pub fn get_block(&self, id: &str) -> SqlResult<Option<Block>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
             FROM blocks WHERE id = ?1",
        )?;
        let mut rows = stmt.query_map(params![id], |r| {
            Ok(Block {
                id: r.get(0)?,
                page_id: r.get(1)?,
                parent_block_id: r.get(2)?,
                block_type: r.get(3)?,
                content: r.get(4)?,
                props: r.get(5)?,
                sort_order: r.get(6)?,
                depth: r.get(7)?,
                created_at: r.get(8)?,
                updated_at: r.get(9)?,
            })
        })?;
        match rows.next() {
            Some(Ok(block)) => Ok(Some(block)),
            _ => Ok(None),
        }
    }

    pub fn update_block_content(&self, id: &str, content: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE blocks SET content = ?1 WHERE id = ?2",
            params![content, id],
        )?;
        Ok(())
    }

    pub fn update_block_type(&self, id: &str, block_type: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE blocks SET block_type = ?1 WHERE id = ?2",
            params![block_type, id],
        )?;
        Ok(())
    }

    pub fn update_block_props(&self, id: &str, props: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE blocks SET props = ?1 WHERE id = ?2",
            params![props, id],
        )?;
        Ok(())
    }

    pub fn delete_block(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM blocks WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn reorder_blocks(&self, page_id: &str, block_ids: &[String]) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        for (i, bid) in block_ids.iter().enumerate() {
            conn.execute(
                "UPDATE blocks SET sort_order = ?1 WHERE id = ?2 AND page_id = ?3",
                params![i as i32, bid, page_id],
            )?;
        }
        Ok(())
    }

    pub fn indent_block(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let block: Block = conn
            .query_row(
                "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
                 FROM blocks WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Block {
                        id: r.get(0)?,
                        page_id: r.get(1)?,
                        parent_block_id: r.get(2)?,
                        block_type: r.get(3)?,
                        content: r.get(4)?,
                        props: r.get(5)?,
                        sort_order: r.get(6)?,
                        depth: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                    })
                },
            )?;
        let prev_sibling: Option<Block> = conn
            .query_row(
                "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
                 FROM blocks WHERE page_id = ?1 AND parent_block_id IS ?2 AND sort_order < ?3
                 ORDER BY sort_order DESC LIMIT 1",
                params![block.page_id, block.parent_block_id, block.sort_order],
                |r| {
                    Ok(Block {
                        id: r.get(0)?,
                        page_id: r.get(1)?,
                        parent_block_id: r.get(2)?,
                        block_type: r.get(3)?,
                        content: r.get(4)?,
                        props: r.get(5)?,
                        sort_order: r.get(6)?,
                        depth: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                    })
                },
            )
            .ok();
        if let Some(prev) = prev_sibling {
            let new_depth = prev.depth + 1;
            conn.execute(
                "UPDATE blocks SET parent_block_id = ?1, depth = ?2 WHERE id = ?3",
                params![prev.id, new_depth, id],
            )?;
        }
        Ok(())
    }

    pub fn outdent_block(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        let block: Block = conn
            .query_row(
                "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
                 FROM blocks WHERE id = ?1",
                params![id],
                |r| {
                    Ok(Block {
                        id: r.get(0)?,
                        page_id: r.get(1)?,
                        parent_block_id: r.get(2)?,
                        block_type: r.get(3)?,
                        content: r.get(4)?,
                        props: r.get(5)?,
                        sort_order: r.get(6)?,
                        depth: r.get(7)?,
                        created_at: r.get(8)?,
                        updated_at: r.get(9)?,
                    })
                },
            )?;
        if let Some(parent_id) = block.parent_block_id {
            let parent: Block = conn
                .query_row(
                    "SELECT id, page_id, parent_block_id, block_type, content, props, sort_order, depth, created_at, updated_at
                     FROM blocks WHERE id = ?1",
                    params![parent_id],
                    |r| {
                        Ok(Block {
                            id: r.get(0)?,
                            page_id: r.get(1)?,
                            parent_block_id: r.get(2)?,
                            block_type: r.get(3)?,
                            content: r.get(4)?,
                            props: r.get(5)?,
                            sort_order: r.get(6)?,
                            depth: r.get(7)?,
                            created_at: r.get(8)?,
                            updated_at: r.get(9)?,
                        })
                    },
                )?;
            let new_depth = parent.depth;
            conn.execute(
                "UPDATE blocks SET parent_block_id = ?1, depth = ?2 WHERE id = ?3",
                params![parent.parent_block_id, new_depth, id],
            )?;
        }
        Ok(())
    }

    pub fn update_block_parent(&self, id: &str, parent_id: Option<&str>, depth: i32) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE blocks SET parent_block_id = ?1, depth = ?2 WHERE id = ?3",
            params![parent_id, depth, id],
        )?;
        Ok(())
    }

    // ── Databases ──

    pub fn create_database(&self, page_id: &str, title: &str, columns: &str) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO databases (id, page_id, title, columns) VALUES (?1, ?2, ?3, ?4)",
            params![id, page_id, title, columns],
        )?;
        Ok(id)
    }

    pub fn get_databases(&self, page_id: &str) -> SqlResult<Vec<Database>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, page_id, title, columns, created_at, updated_at
             FROM databases WHERE page_id = ?1 ORDER BY created_at",
        )?;
        let rows = stmt.query_map(params![page_id], |r| {
            Ok(Database {
                id: r.get(0)?,
                page_id: r.get(1)?,
                title: r.get(2)?,
                columns: r.get(3)?,
                created_at: r.get(4)?,
                updated_at: r.get(5)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_database_columns(&self, id: &str, columns: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE databases SET columns = ?1 WHERE id = ?2",
            params![columns, id],
        )?;
        Ok(())
    }

    pub fn delete_database(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM databases WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Database Rows ──

    pub fn create_database_row(
        &self,
        db_id: &str,
        page_id: &str,
        cells: &str,
    ) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        let sort: i32 = conn
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM database_rows WHERE db_id = ?1",
                params![db_id],
                |r| r.get(0),
            )
            .unwrap_or(0);
        conn.execute(
            "INSERT INTO database_rows (id, db_id, page_id, cells, sort_order) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, db_id, page_id, cells, sort],
        )?;
        Ok(id)
    }

    pub fn get_database_rows(&self, db_id: &str) -> SqlResult<Vec<DatabaseRow>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, db_id, page_id, cells, sort_order
             FROM database_rows WHERE db_id = ?1 ORDER BY sort_order",
        )?;
        let rows = stmt.query_map(params![db_id], |r| {
            Ok(DatabaseRow {
                id: r.get(0)?,
                db_id: r.get(1)?,
                page_id: r.get(2)?,
                cells: r.get(3)?,
                sort_order: r.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn update_database_row_cells(&self, id: &str, cells: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE database_rows SET cells = ?1 WHERE id = ?2",
            params![cells, id],
        )?;
        Ok(())
    }

    pub fn delete_database_row(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM database_rows WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Database Views ──

    pub fn create_database_view(
        &self,
        db_id: &str,
        name: &str,
        view_type: &str,
        options: &str,
    ) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO database_views (id, db_id, name, view_type, options) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, db_id, name, view_type, options],
        )?;
        Ok(id)
    }

    pub fn get_database_views(&self, db_id: &str) -> SqlResult<Vec<DatabaseView>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, db_id, name, view_type, options
             FROM database_views WHERE db_id = ?1 ORDER BY name",
        )?;
        let rows = stmt.query_map(params![db_id], |r| {
            Ok(DatabaseView {
                id: r.get(0)?,
                db_id: r.get(1)?,
                name: r.get(2)?,
                view_type: r.get(3)?,
                options: r.get(4)?,
            })
        })?;
        rows.collect()
    }

    // ── Templates ──

    pub fn create_template(&self, name: &str, block_json: &str) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO templates (id, name, block_json) VALUES (?1, ?2, ?3)",
            params![id, name, block_json],
        )?;
        Ok(id)
    }

    pub fn get_templates(&self) -> SqlResult<Vec<Template>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt =
            conn.prepare("SELECT id, name, block_json FROM templates ORDER BY name")?;
        let rows = stmt.query_map([], |r| {
            Ok(Template {
                id: r.get(0)?,
                name: r.get(1)?,
                block_json: r.get(2)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_template(&self, id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM templates WHERE id = ?1", params![id])?;
        Ok(())
    }

    // ── Backlinks ──

    pub fn create_backlink(
        &self,
        source_page_id: &str,
        target_page_id: &str,
        block_id: &str,
        snippet: &str,
    ) -> SqlResult<String> {
        let id = Self::new_id();
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO backlinks (id, source_page_id, target_page_id, block_id, snippet)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![id, source_page_id, target_page_id, block_id, snippet],
        )?;
        Ok(id)
    }

    pub fn get_backlinks(&self, page_id: &str) -> SqlResult<Vec<Backlink>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, source_page_id, target_page_id, block_id, snippet
             FROM backlinks WHERE target_page_id = ?1",
        )?;
        let rows = stmt.query_map(params![page_id], |r| {
            Ok(Backlink {
                id: r.get(0)?,
                source_page_id: r.get(1)?,
                target_page_id: r.get(2)?,
                block_id: r.get(3)?,
                snippet: r.get(4)?,
            })
        })?;
        rows.collect()
    }

    pub fn delete_backlinks_for_source(&self, source_page_id: &str) -> SqlResult<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM backlinks WHERE source_page_id = ?1",
            params![source_page_id],
        )?;
        Ok(())
    }

    // ── Page tree (sidebar) ──

    pub fn get_page_tree(&self) -> SqlResult<Vec<Page>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT id, title, icon, cover, parent_id, sort_order, is_database, created_at, updated_at
             FROM pages ORDER BY parent_id, sort_order",
        )?;
        let rows = stmt.query_map([], |r| {
            Ok(Page {
                id: r.get(0)?,
                title: r.get(1)?,
                icon: r.get(2)?,
                cover: r.get(3)?,
                parent_id: r.get(4)?,
                sort_order: r.get(5)?,
                is_database: r.get::<_, i32>(6)? != 0,
                created_at: r.get(7)?,
                updated_at: r.get(8)?,
            })
        })?;
        rows.collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_and_get_page() {
        let store = SqliteStore::open_in_memory().unwrap();
        let id = store.create_page("Test Page", Some("📝"), None).unwrap();
        let page = store.get_page(&id).unwrap().unwrap();
        assert_eq!(page.title, "Test Page");
        assert_eq!(page.icon.unwrap(), "📝");
        assert!(page.parent_id.is_none());
    }

    #[test]
    fn test_create_nested_pages() {
        let store = SqliteStore::open_in_memory().unwrap();
        let parent_id = store.create_page("Parent", None, None).unwrap();
        let child_id = store
            .create_page("Child", None, Some(&parent_id))
            .unwrap();
        let root_pages = store.list_pages(None).unwrap();
        assert_eq!(root_pages.len(), 1);
        assert_eq!(root_pages[0].id, parent_id);
        let children = store.list_pages(Some(&parent_id)).unwrap();
        assert_eq!(children.len(), 1);
        assert_eq!(children[0].id, child_id);
    }

    #[test]
    fn test_update_and_delete_page() {
        let store = SqliteStore::open_in_memory().unwrap();
        let id = store.create_page("Old", None, None).unwrap();
        store.update_page(&id, "New", Some("🔵"), None).unwrap();
        let page = store.get_page(&id).unwrap().unwrap();
        assert_eq!(page.title, "New");
        assert_eq!(page.icon.unwrap(), "🔵");
        store.delete_page(&id).unwrap();
        assert!(store.get_page(&id).unwrap().is_none());
    }

    #[test]
    fn test_create_block() {
        let store = SqliteStore::open_in_memory().unwrap();
        let page_id = store.create_page("Page", None, None).unwrap();
        let block_id = store.create_block(&page_id, "text", "Hello", None, None).unwrap();
        let blocks = store.get_blocks(&page_id).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].content, "Hello");
        assert_eq!(blocks[0].block_type, "text");
        assert_eq!(blocks[0].page_id, page_id);
    }

    #[test]
    fn test_nested_blocks() {
        let store = SqliteStore::open_in_memory().unwrap();
        let page_id = store.create_page("Page", None, None).unwrap();
        let parent = store.create_block(&page_id, "text", "Parent", None, None).unwrap();
        let child = store
            .create_block(&page_id, "text", "Child", Some(&parent), None)
            .unwrap();
        let blocks = store.get_blocks(&page_id).unwrap();
        let child_block = store.get_block(&child).unwrap().unwrap();
        assert_eq!(child_block.parent_block_id.unwrap(), parent);
        assert_eq!(child_block.depth, 1);
    }

    #[test]
    fn test_reorder_blocks() {
        let store = SqliteStore::open_in_memory().unwrap();
        let page_id = store.create_page("Page", None, None).unwrap();
        let a = store.create_block(&page_id, "text", "A", None, None).unwrap();
        let b = store.create_block(&page_id, "text", "B", None, None).unwrap();
        store.reorder_blocks(&page_id, &[b.clone(), a.clone()]).unwrap();
        let blocks = store.get_blocks(&page_id).unwrap();
        assert_eq!(blocks[0].id, b);
        assert_eq!(blocks[0].sort_order, 0);
        assert_eq!(blocks[1].id, a);
        assert_eq!(blocks[1].sort_order, 1);
    }

    #[test]
    fn test_database() {
        let store = SqliteStore::open_in_memory().unwrap();
        let page_id = store.create_page("DB Page", None, None).unwrap();
        let db_id = store
            .create_database(&page_id, "Tasks", r#"[{"name":"Task","type":"text"}]"#)
            .unwrap();
        let dbs = store.get_databases(&page_id).unwrap();
        assert_eq!(dbs.len(), 1);
        assert_eq!(dbs[0].title, "Tasks");
        store
            .update_database_columns(&db_id, r#"[{"name":"Task","type":"text"},{"name":"Status","type":"select"}]"#)
            .unwrap();
        let dbs = store.get_databases(&page_id).unwrap();
        assert!(dbs[0].columns.contains("Status"));
    }

    #[test]
    fn test_database_rows_and_views() {
        let store = SqliteStore::open_in_memory().unwrap();
        let page_id = store.create_page("DB", None, None).unwrap();
        let db_id = store
            .create_database(&page_id, "Items", r#"[{"name":"Name","type":"text"}]"#)
            .unwrap();
        let row_id = store
            .create_database_row(&db_id, &page_id, r#"{"Name":"Foo"}"#)
            .unwrap();
        let rows = store.get_database_rows(&db_id).unwrap();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].cells, r#"{"Name":"Foo"}"#);
        let view_id = store
            .create_database_view(&db_id, "Board", "kanban", r#"{"groupBy":"Status"}"#)
            .unwrap();
        let views = store.get_database_views(&db_id).unwrap();
        assert_eq!(views.len(), 1);
        assert_eq!(views[0].view_type, "kanban");
    }

    #[test]
    fn test_templates() {
        let store = SqliteStore::open_in_memory().unwrap();
        let tid = store
            .create_template("Meeting Notes", r#"[{"type":"heading","content":"Meeting"}]"#)
            .unwrap();
        let templates = store.get_templates().unwrap();
        assert_eq!(templates.len(), 1);
        assert_eq!(templates[0].name, "Meeting Notes");
        store.delete_template(&tid).unwrap();
        assert!(store.get_templates().unwrap().is_empty());
    }

    #[test]
    fn test_backlinks() {
        let store = SqliteStore::open_in_memory().unwrap();
        let src = store.create_page("Source", None, None).unwrap();
        let tgt = store.create_page("Target", None, None).unwrap();
        let blk = store
            .create_block(&src, "text", "See [[Target]]", None, None)
            .unwrap();
        store
            .create_backlink(&src, &tgt, &blk, "See [[Target]]")
            .unwrap();
        let links = store.get_backlinks(&tgt).unwrap();
        assert_eq!(links.len(), 1);
        assert_eq!(links[0].source_page_id, src);
    }

    #[test]
    fn test_trigger_updated_at() {
        let store = SqliteStore::open_in_memory().unwrap();
        let id = store.create_page("Test", None, None).unwrap();
        let original = store.get_page(&id).unwrap().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(10));
        store.update_page(&id, "Updated", None, None).unwrap();
        let updated = store.get_page(&id).unwrap().unwrap();
        assert_ne!(original.updated_at, updated.updated_at);
    }

    #[test]
    fn test_get_page_tree() {
        let store = SqliteStore::open_in_memory().unwrap();
        let p1 = store.create_page("Root1", None, None).unwrap();
        let p2 = store.create_page("Root2", None, None).unwrap();
        let c1 = store
            .create_page("Child1", None, Some(&p1))
            .unwrap();
        let tree = store.get_page_tree().unwrap();
        assert!(tree.iter().any(|p| p.id == p1));
        assert!(tree.iter().any(|p| p.id == p2));
        assert!(tree.iter().any(|p| p.id == c1));
    }

    #[test]
    fn test_move_page() {
        let store = SqliteStore::open_in_memory().unwrap();
        let p1 = store.create_page("Parent", None, None).unwrap();
        let p2 = store.create_page("Child", None, None).unwrap();
        store.move_page(&p2, Some(&p1), 0).unwrap();
        let page = store.get_page(&p2).unwrap().unwrap();
        assert_eq!(page.parent_id.unwrap(), p1);
    }
}
