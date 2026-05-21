//! Page tree sidebar panel for the Notion-like page hierarchy.
//!
//! Displays pages from the SQLite store as a collapsible tree
//! with icons/emojis, click-to-navigate, and context menu actions.

#![allow(dead_code)]

use crate::store::{Page, SqliteStore};
use crate::ui::docked_sidebar::{self, DockedSidebarEdge};
use eframe::egui::{self, Color32, RichText, Sense, Ui, Vec2};

const DEFAULT_PANEL_WIDTH: f32 = 260.0;
const MIN_PANEL_WIDTH: f32 = 180.0;
const MAX_PANEL_WIDTH: f32 = 450.0;
const INDENT_PER_LEVEL: f32 = 18.0;
const ROW_HEIGHT: f32 = 24.0;

#[derive(Debug, Clone)]
pub struct PageTreeNode {
    pub page: Page,
    pub children: Vec<PageTreeNode>,
    pub is_expanded: bool,
}

#[derive(Debug, Default)]
pub struct PageTreeOutput {
    pub page_clicked: Option<String>,
    pub create_page_requested: Option<Option<String>>,
    pub delete_page_requested: Option<String>,
    pub rename_page_completed: Option<(String, String)>,
    pub page_move_requested: Option<(String, Option<String>)>,
    pub close_requested: bool,
    pub new_width: Option<f32>,
}

pub struct PageTreePanel {
    width: f32,
    is_resizing: bool,
    expanded_ids: Vec<String>,
    search_query: String,
    rename_page_id: Option<String>,
    rename_buffer: String,
    dragged_page_id: Option<String>,
    drop_target_id: Option<String>,
}

impl Default for PageTreePanel {
    fn default() -> Self {
        Self::new()
    }
}

impl PageTreePanel {
    pub fn new() -> Self {
        Self {
            width: DEFAULT_PANEL_WIDTH,
            is_resizing: false,
            expanded_ids: Vec::new(),
            search_query: String::new(),
            rename_page_id: None,
            rename_buffer: String::new(),
            dragged_page_id: None,
            drop_target_id: None,
        }
    }

    pub fn with_width(mut self, width: f32) -> Self {
        self.width = width.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
        self
    }

    pub fn set_width(&mut self, width: f32) {
        self.width = width.clamp(MIN_PANEL_WIDTH, MAX_PANEL_WIDTH);
    }

    pub fn width(&self) -> f32 {
        self.width
    }

    pub fn show(
        &mut self,
        ctx: &egui::Context,
        store: &SqliteStore,
        current_page_id: Option<&str>,
        is_dark: bool,
    ) -> PageTreeOutput {
        let mut output = PageTreeOutput::default();
        let mut expanded_ids = std::mem::take(&mut self.expanded_ids);

        let panel_bg = if is_dark {
            Color32::from_rgb(30, 30, 46)
        } else {
            Color32::from_rgb(245, 245, 245)
        };

        let border_color = if is_dark {
            Color32::from_rgb(42, 42, 62)
        } else {
            Color32::from_rgb(200, 200, 200)
        };

        let _text_color = if is_dark {
            Color32::from_rgb(205, 214, 244)
        } else {
            Color32::from_rgb(40, 40, 40)
        };

        let muted_color = if is_dark {
            Color32::from_rgb(108, 112, 134)
        } else {
            Color32::from_rgb(140, 140, 140)
        };

        let accent_color = if is_dark {
            Color32::from_rgb(203, 166, 247)
        } else {
            Color32::from_rgb(130, 80, 200)
        };

        // Build page tree from store
        let nodes = Self::build_page_tree(store);

        egui::SidePanel::left("page_tree_panel")
            .resizable(true)
            .default_width(self.width)
            .width_range(MIN_PANEL_WIDTH..=MAX_PANEL_WIDTH)
            .frame(docked_sidebar::frame(panel_bg))
            .show(ctx, |ui| {
                let panel_width = ui.available_width();
                if (panel_width - self.width).abs() > 1.0 {
                    self.width = panel_width;
                    output.new_width = Some(panel_width);
                }

                // Header
                ui.horizontal(|ui| {
                    ui.add_space(4.0);
                    ui.label("📄");
                    ui.add(
                        egui::Label::new(RichText::new("Páginas").size(13.0).strong())
                            .truncate(),
                    );
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.add_space(8.0);
                        if ui
                            .add(egui::Button::new(RichText::new("✕").size(12.0)).frame(false))
                            .on_hover_text("Cerrar")
                            .clicked()
                        {
                            output.close_requested = true;
                        }
                    });
                });

                ui.add_space(2.0);

                // Search bar
                ui.add_space(4.0);
                let _search_id = ui.next_auto_id();
                let search_response = ui.add(
                    egui::TextEdit::singleline(&mut self.search_query)
                        .hint_text("🔍 Buscar páginas...")
                        .desired_width(f32::INFINITY)
                        .font(egui::TextStyle::Body),
                );
                if search_response.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Escape)) {
                    self.search_query.clear();
                }
                ui.add_space(4.0);

                ui.separator();

                // Scrollable tree
                egui::ScrollArea::vertical()
                    .auto_shrink([false, false])
                    .show(ui, |ui| {
                        ui.add_space(4.0);

                        if nodes.is_empty() {
                            ui.add_space(20.0);
                            ui.vertical_centered(|ui| {
                                ui.label(RichText::new("Sin páginas").color(muted_color).size(13.0));
                                ui.add_space(4.0);
                                if ui.button(RichText::new("＋ Crear página").color(accent_color).size(12.0)).clicked() {
                                    output.create_page_requested = Some(None);
                                }
                            });
                        } else {
                            let filtered = if self.search_query.is_empty() {
                                nodes.clone()
                            } else {
                                self.filter_nodes(&nodes, &self.search_query.to_lowercase())
                            };

                            for node in &filtered {
                                self.render_node(ui, node, 0, current_page_id, is_dark, &mut output, &mut expanded_ids);
                            }
                        }

                        ui.add_space(8.0);

                        // New page button
                        let add_label = if self.search_query.is_empty() {
                            "＋ Nueva página"
                        } else {
                            "＋ Crear"
                        };
                        if ui
                            .add(
                                egui::Button::new(RichText::new(add_label).size(12.0).color(accent_color))
                                    .fill(Color32::TRANSPARENT)
                                    .frame(false),
                            )
                            .on_hover_text("Crear nueva página")
                            .clicked()
                        {
                            output.create_page_requested = Some(None);
                        }
                    });

                // Handle drag & drop completion
                if self.dragged_page_id.is_some() {
                    let mouse_down = ui.input(|i| i.pointer.any_down());
                    if !mouse_down {
                        if let Some(target_id) = self.drop_target_id.take() {
                            if let Some(drag_id) = self.dragged_page_id.take() {
                                if drag_id != target_id {
                                    output.page_move_requested = Some((drag_id, Some(target_id)));
                                }
                            }
                        } else {
                            self.dragged_page_id = None;
                        }
                    }
                }

                docked_sidebar::paint_vertical_divider(ui, border_color, DockedSidebarEdge::Left);
            });

        self.expanded_ids = expanded_ids;
        output
    }

    fn build_page_tree(store: &SqliteStore) -> Vec<PageTreeNode> {
        let pages = store.get_page_tree().unwrap_or_default();
        Self::build_tree(&pages, None)
    }

    fn build_tree(pages: &[Page], parent_id: Option<&str>) -> Vec<PageTreeNode> {
        let children: Vec<PageTreeNode> = pages
            .iter()
            .filter(|p| p.parent_id.as_deref() == parent_id)
            .map(|p| {
                let sub = Self::build_tree(pages, Some(&p.id));
                PageTreeNode {
                    page: p.clone(),
                    children: sub,
                    is_expanded: true,
                }
            })
            .collect();
        children
    }

    fn filter_nodes(&self, nodes: &[PageTreeNode], query: &str) -> Vec<PageTreeNode> {
        nodes
            .iter()
            .filter(|n| {
                let title_match = n.page.title.to_lowercase().contains(query);
                let child_match = !self.filter_nodes(&n.children, query).is_empty();
                title_match || child_match
            })
            .map(|n| {
                let filtered_children = self.filter_nodes(&n.children, query);
                if !filtered_children.is_empty() {
                    // Show parent when children match, expanded
                    PageTreeNode {
                        is_expanded: true,
                        ..n.clone()
                    }
                } else {
                    PageTreeNode {
                        children: filtered_children,
                        ..n.clone()
                    }
                }
            })
            .collect()
    }

    fn render_node(
        &mut self,
        ui: &mut Ui,
        node: &PageTreeNode,
        depth: usize,
        current_page_id: Option<&str>,
        is_dark: bool,
        output: &mut PageTreeOutput,
        expanded_ids: &mut Vec<String>,
    ) {
        let indent = depth as f32 * INDENT_PER_LEVEL;

        let text_color = if is_dark {
            Color32::from_rgb(205, 214, 244)
        } else {
            Color32::from_rgb(40, 40, 40)
        };

        let hover_bg = if is_dark {
            Color32::from_rgb(49, 50, 68)
        } else {
            Color32::from_rgb(220, 225, 235)
        };

        let active_bg = if is_dark {
            Color32::from_rgb(69, 71, 90)
        } else {
            Color32::from_rgb(200, 210, 230)
        };

        let is_active = current_page_id == Some(&node.page.id);
        let has_children = !node.children.is_empty();
        let is_expanded = expanded_ids.contains(&node.page.id);

        let row_width = ui.available_width();
        let (row_rect, row_response) =
            ui.allocate_exact_size(Vec2::new(row_width, ROW_HEIGHT), Sense::click_and_drag());

        // Background
        if is_active {
            ui.painter().rect_filled(row_rect, 4.0, active_bg);
        } else if row_response.hovered() {
            ui.painter().rect_filled(row_rect, 4.0, hover_bg);
            ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
        }

        // Draw left accent for active page
        if is_active {
            let accent = if is_dark {
                Color32::from_rgb(203, 166, 247)
            } else {
                Color32::from_rgb(130, 80, 200)
            };
            ui.painter().rect_filled(
                egui::Rect::from_min_size(
                    egui::pos2(row_rect.left(), row_rect.top()),
                    egui::vec2(3.0, row_rect.height()),
                ),
                2.0,
                accent,
            );
        }

        let mut content_pos = row_rect.left_top() + Vec2::new(indent + 4.0, 3.0);

        // Expand/collapse arrow for pages with children
        if has_children {
            let arrow = if is_expanded { "▼" } else { "▶" };
            ui.painter().text(
                content_pos,
                egui::Align2::LEFT_TOP,
                arrow,
                egui::FontId::proportional(9.0),
                text_color,
            );
            content_pos.x += 14.0;
        } else {
            content_pos.x += 14.0;
        }

        // Page icon (emoji or fallback)
        let icon = node.page.icon.as_deref().unwrap_or("📄");
        ui.painter().text(
            content_pos,
            egui::Align2::LEFT_TOP,
            icon,
            egui::FontId::proportional(14.0),
            text_color,
        );
        content_pos.x += 20.0;

        // Rename mode: show inline TextEdit
        if self.rename_page_id.as_deref() == Some(&node.page.id) {
            let text_rect = egui::Rect::from_min_size(
                content_pos,
                egui::vec2(row_width - content_pos.x - 8.0, ROW_HEIGHT),
            );
            let text_edit = egui::TextEdit::singleline(&mut self.rename_buffer)
                .desired_width(f32::INFINITY)
                .font(egui::TextStyle::Body);
            let response = ui.put(text_rect, text_edit);

            if !response.has_focus() {
                response.request_focus();
            }

            let enter_pressed = ui.input(|i| i.key_pressed(egui::Key::Enter));
            let escape_pressed = ui.input(|i| i.key_pressed(egui::Key::Escape));
            let focus_lost = response.lost_focus() && !enter_pressed && !escape_pressed;

            if enter_pressed {
                if !self.rename_buffer.is_empty() {
                    output.rename_page_completed =
                        Some((node.page.id.clone(), self.rename_buffer.clone()));
                }
                self.rename_page_id = None;
                self.rename_buffer.clear();
            } else if escape_pressed {
                self.rename_page_id = None;
                self.rename_buffer.clear();
            } else if focus_lost {
                if !self.rename_buffer.is_empty() {
                    output.rename_page_completed =
                        Some((node.page.id.clone(), self.rename_buffer.clone()));
                }
                self.rename_page_id = None;
                self.rename_buffer.clear();
            }
        } else {
            // Page title
            let title = if node.page.title.is_empty() {
                "Sin título"
            } else {
                &node.page.title
            };

            let title_color = if node.page.title.is_empty() {
                if is_dark {
                    Color32::from_rgb(108, 112, 134)
                } else {
                    Color32::from_rgb(140, 140, 140)
                }
            } else {
                text_color
            };

            ui.painter().text(
                content_pos,
                egui::Align2::LEFT_TOP,
                title,
                egui::FontId::proportional(13.0),
                title_color,
            );
        }

        // Handle click
        if row_response.clicked() {
            if has_children {
                // Toggle expansion
                if is_expanded {
                    expanded_ids.retain(|id| id != &node.page.id);
                } else {
                    expanded_ids.push(node.page.id.clone());
                }
            }
            output.page_clicked = Some(node.page.id.clone());
        }

        // Drag & drop: detect drag start
        if row_response.drag_started() {
            self.dragged_page_id = Some(node.page.id.clone());
        }

        // Drop target detection while dragging
        if self.dragged_page_id.is_some()
            && self.dragged_page_id.as_deref() != Some(&node.page.id)
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                if row_rect.contains(pointer_pos) {
                    self.drop_target_id = Some(node.page.id.clone());

                    // Draw drop indicator (colored bar at top of row)
                    let indicator_color = Color32::from_rgb(203, 166, 247);
                    ui.painter().rect_filled(
                        egui::Rect::from_min_size(
                            egui::pos2(row_rect.left(), row_rect.top()),
                            egui::vec2(row_rect.width(), 2.0),
                        ),
                        1.0,
                        indicator_color,
                    );
                }
            }
        } else if self.dragged_page_id.as_deref() == Some(&node.page.id) {
            // Dim the dragged item
            let dim_bg = Color32::from_rgba_premultiplied(100, 100, 120, 40);
            ui.painter().rect_filled(row_rect, 4.0, dim_bg);
        }

        // Context menu
        row_response.context_menu(|ui| {
            if ui.button("＋ Nueva sub-página").clicked() {
                output.create_page_requested = Some(Some(node.page.id.clone()));
                ui.close_menu();
            }
            ui.separator();
            if ui.button("✏️ Renombrar").clicked() {
                self.rename_page_id = Some(node.page.id.clone());
                self.rename_buffer = if node.page.title.is_empty() {
                    String::new()
                } else {
                    node.page.title.clone()
                };
                ui.close_menu();
            }
            ui.separator();
            if ui.button("🗑️ Eliminar").clicked() {
                output.delete_page_requested = Some(node.page.id.clone());
                ui.close_menu();
            }
        });

        // Render children if expanded
        if has_children && is_expanded {
            for child in &node.children {
                self.render_node(ui, child, depth + 1, current_page_id, is_dark, output, expanded_ids);
            }
        }
    }
}
