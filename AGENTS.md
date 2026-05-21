# AppMD — Proyecto Notion MVP

## Goal
Convertir AppMD (formerly Ferrite) — editor Markdown Rust/egui — en un clon de Notion 100% local, open source, sin suscripciones.

## Branch
`notion-mvp`

## Constraints & Preferences
- 100% local (sin cloud, sin cuentas), open source (MIT), gratuito.
- Nativo Rust/egui (sin Electron).
- Markdown nativo compatible con herramientas existentes.
- Almacenamiento SQLite (rusqlite), UUID v4.
- UI con temática tipo Catppuccin Mocha (oscuro/claro).
- Proyecto renombrado de "Ferrite" a "AppMD".
- Keyboard shortcut: `Ctrl+Shift+N` toggles page tree.

## Progress

### Done
- Día 1: Store SQLite con CRUD completo de pages, blocks, databases, templates, backlinks. 13 tests.
- Día 2: PageTreePanel widget en sidebar izquierdo con árbol jerárquico desde SQLite, búsqueda, expandir/colapsar, menú contextual, iconos/emojis.
- Día 3: CRUD completo en page tree (crear sub-páginas con parent_id, renombrar inline, drag & drop para re-anidar páginas).
- Día 4: Block types base + renderizado básico.
  - `BlockType` enum con 15 tipos (Text, Heading1-3, Todo, BulletedList, NumberedList, Toggle, Code, Quote, Divider, Callout, Image, Bookmark, Equation) en `src/store/block_types.rs`.
  - `BlockEditor` widget (`src/ui/block_editor.rs`) que renderiza cada block type con egui widgets.
  - `PageEditor` widget (`src/ui/page_editor.rs`) que carga blocks de SQLite y los renderiza en lista vertical, con creación/borrado de blocks.
  - `PageEditorState` en `AppState` con buffers de edición por block y tracking de foco.
  - Integración en central panel: cuando `show_page_tree && current_page_id.is_some()`, muestra PageEditor.
  - Keyboard shortcut `Ctrl+Shift+N` para toggle page tree.
  - Fix: eliminación de BOM UTF-8 de todos los YAML locales y upgrade rust-i18n v3→v4.

### In Progress
- (ninguno)

### Blocked
- (ninguno)

## Next Steps
- Día 5: Slash commands y drag & drop de bloques.

## Key Decisions
- PageTreePanel se integra en AppMDApp como campo independiente (no reemplaza FileTreePanel; se activa con `show_page_tree`).
- SqliteStore se agrega a AppState como `Option<SqliteStore>`, inicializado en `AppState::new()`.
- DB se abre en `dirs::data_dir()/AppMD/pages.db`.
- `PageEditor` usa `&SqliteStore` directamente (ya tiene `Mutex` interno).
- Al seleccionar página en tree, se activa PageEditor en central panel (reemplaza editor de archivos normal).
- `rust-i18n` v4 necesario por breaking change en serde_yaml 0.9.34+deprecated.

## Relevant Files
- `src/store/block_types.rs` — BlockType enum
- `src/ui/block_editor.rs` — render_block() function
- `src/ui/page_editor.rs` — PageEditor widget
- `src/state.rs` — PageEditorState struct
- `src/store/mod.rs` — Block struct with block_type_enum() helper
- `src/app/central_panel.rs` — PageEditor integration point
- `src/app/keyboard.rs` — Ctrl+Shift+N toggle
- `src/app/types.rs` — TogglePageTree KeyboardAction
- `Cargo.toml` — rust-i18n = "4"
