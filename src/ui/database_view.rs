use crate::store::{Database, DatabaseRow, SqliteStore};
use eframe::egui;
use serde_json::Value;

pub struct DatabaseView {
    db: Option<Database>,
    rows: Vec<DatabaseRow>,
    previous_db_id: Option<String>,
}

impl DatabaseView {
    pub fn new() -> Self {
        Self {
            db: None,
            rows: Vec::new(),
            previous_db_id: None,
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        store: &SqliteStore,
        page_id: &str,
        is_dark: bool,
    ) {
        let dbs = store.get_databases(page_id).unwrap_or_default();
        if dbs.is_empty() {
            ui.label("No database on this page.");
            if ui.button("Create database").clicked() {
                let _ = store.create_database(
                    page_id,
                    "Untitled Database",
                    r#"[{"name":"Name","type":"text"},{"name":"Status","type":"text"}]"#,
                );
            }
            return;
        }

        let db = dbs[0].clone();
        if self.previous_db_id.as_deref() != Some(&db.id) {
            self.db = Some(db.clone());
            self.rows = store.get_database_rows(&db.id).unwrap_or_default();
            self.previous_db_id = Some(db.id.clone());
        }

        let db = self.db.as_ref().unwrap();
        let columns: Vec<ColumnDef> = serde_json::from_str(&db.columns).unwrap_or_default();

        let header_bg = if is_dark {
            egui::Color32::from_rgb(30, 30, 45)
        } else {
            egui::Color32::from_rgb(240, 240, 245)
        };
        let border_color = if is_dark {
            egui::Color32::from_rgb(45, 45, 60)
        } else {
            egui::Color32::from_rgb(220, 220, 230)
        };
        let alt_row_bg = if is_dark {
            egui::Color32::from_rgb(22, 22, 35)
        } else {
            egui::Color32::from_rgb(248, 248, 252)
        };

        let mut delete_row_id: Option<String> = None;
        let mut cell_edits: Vec<(String, String, String)> = Vec::new();

        egui::Frame::none()
            .rounding(4.0)
            .stroke(egui::epaint::Stroke::new(1.0, border_color))
            .show(ui, |ui| {
                egui::ScrollArea::horizontal()
                    .id_source("db_table_h_scroll")
                    .show(ui, |ui| {
                        let total_cols = columns.len() + 1;
                        let col_width = 150.0;
                        let total_width = col_width * total_cols as f32;
                        ui.set_min_width(total_width);

                        let header_height = 32.0;
                        egui::Frame::none()
                            .fill(header_bg)
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.set_min_height(header_height);
                                    ui.allocate_space(egui::vec2(40.0, header_height));
                                    for col in &columns {
                                        ui.vertical(|ui| {
                                            ui.set_min_size(egui::vec2(col_width, header_height));
                                            ui.strong(&col.name);
                                        });
                                    }
                                });
                            });

                        for (row_idx, row) in self.rows.iter().enumerate() {
                            let row_bg = if row_idx % 2 == 0 {
                                egui::Color32::TRANSPARENT
                            } else {
                                alt_row_bg
                            };
                            let cells: Value =
                                serde_json::from_str(&row.cells).unwrap_or(Value::Object(serde_json::Map::new()));

                            egui::Frame::none()
                                .fill(row_bg)
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.set_min_height(28.0);
                                        if ui
                                            .add(
                                                egui::Button::new("\u{2716}")
                                                    .frame(false)
                                                    .small(),
                                            )
                                            .clicked()
                                        {
                                            delete_row_id = Some(row.id.clone());
                                        }
                                        for col in &columns {
                                            let mut val = cell_value(&cells, &col.name);
                                            let resp = ui.add_sized(
                                                egui::vec2(col_width, 28.0),
                                                egui::TextEdit::singleline(&mut val)
                                                    .desired_width(f32::INFINITY),
                                            );
                                            if resp.changed() {
                                                cell_edits.push((row.id.clone(), col.name.clone(), val));
                                            }
                                        }
                                    });
                                });
                        }
                    });
            });

        for (row_id, col_name, val) in &cell_edits {
            if let Some(row) = self.rows.iter_mut().find(|r| r.id == *row_id) {
                let mut cells: Value =
                    serde_json::from_str(&row.cells).unwrap_or(Value::Object(serde_json::Map::new()));
                if let Value::Object(ref mut map) = cells {
                    map.insert(col_name.clone(), Value::String(val.clone()));
                }
                let new_cells_str = serde_json::to_string(&cells).unwrap_or_default();
                let _ = store.update_database_row_cells(row_id, &new_cells_str);
                row.cells = new_cells_str;
            }
        }

        if let Some(delete_id) = delete_row_id {
            let db_id = self.rows.iter().find(|r| r.id == delete_id).map(|r| r.db_id.clone());
            if let Some(ref db_id) = db_id {
                let _ = store.delete_database_row(&delete_id);
                if let Ok(rows) = store.get_database_rows(db_id) {
                    self.rows = rows;
                }
            }
        }

        ui.horizontal(|ui| {
            if ui.button("+ New row").clicked() {
                let cells = serde_json::json!({}).to_string();
                let _ = store.create_database_row(&db.id, page_id, &cells);
                if let Ok(rows) = store.get_database_rows(&db.id) {
                    self.rows = rows;
                }
            }
        });
    }
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ColumnDef {
    pub name: String,
    #[serde(rename = "type")]
    pub col_type: String,
}

impl Default for ColumnDef {
    fn default() -> Self {
        Self {
            name: String::new(),
            col_type: "text".to_string(),
        }
    }
}

fn cell_value(cells: &Value, col_name: &str) -> String {
    if let Value::Object(ref map) = cells {
        if let Some(Value::String(s)) = map.get(col_name) {
            return s.clone();
        }
    }
    String::new()
}
