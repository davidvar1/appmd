use crate::state::PageEditorState;
use crate::store::block_types::BlockType;
use crate::store::{Block, SqliteStore};
use eframe::egui;

use super::block_editor::render_block;

#[derive(Debug, Default)]
pub struct PageEditorActions {
    pub create_blocks: Vec<CreateBlockAction>,
    pub update_content: Vec<UpdateContentAction>,
    pub update_type: Vec<UpdateTypeAction>,
    pub update_props: Vec<UpdatePropsAction>,
    pub delete_blocks: Vec<String>,
    pub reorder: bool,
}

#[derive(Debug)]
pub struct CreateBlockAction {
    pub page_id: String,
    pub block_type: BlockType,
    pub content: String,
    pub after_block_id: Option<String>,
}

#[derive(Debug)]
pub struct UpdateContentAction {
    pub block_id: String,
    pub content: String,
}

#[derive(Debug)]
pub struct UpdateTypeAction {
    pub block_id: String,
    pub block_type: BlockType,
}

#[derive(Debug)]
pub struct UpdatePropsAction {
    pub block_id: String,
    pub props: String,
}

pub struct PageEditor {
    blocks: Vec<Block>,
    initialized: bool,
    previous_page_id: Option<String>,
}

impl PageEditor {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            initialized: false,
            previous_page_id: None,
        }
    }

    fn render_slash_menu(
        ui: &mut egui::Ui,
        editor_state: &mut PageEditorState,
        block_id: &str,
        actions: &mut PageEditorActions,
        store: &SqliteStore,
        page_id: &str,
    ) {
        use crate::store::block_types::BlockType;

        let slash_types = [
            (BlockType::Text, "Text", "Just start writing"),
            (BlockType::Heading1, "Heading 1", "Large section heading"),
            (BlockType::Heading2, "Heading 2", "Medium section heading"),
            (BlockType::Heading3, "Heading 3", "Small section heading"),
            (BlockType::Todo, "To-do", "Checkbox task"),
            (BlockType::BulletedList, "Bulleted List", "Unordered list"),
            (BlockType::NumberedList, "Numbered List", "Ordered list"),
            (BlockType::Toggle, "Toggle", "Collapsible content"),
            (BlockType::Code, "Code", "Code block with syntax"),
            (BlockType::Quote, "Quote", "Block quote"),
            (BlockType::Divider, "Divider", "Horizontal line"),
            (BlockType::Callout, "Callout", "Highlighted note"),
        ];

        let area_id = egui::Id::new("slash_menu").with(block_id);
        let mut close_menu = false;

        egui::Area::new(area_id)
            .order(egui::Order::Foreground)
            .show(ui.ctx(), |ui| {
                let frame_bg = if ui.visuals().dark_mode {
                    egui::Color32::from_rgb(35, 35, 50)
                } else {
                    egui::Color32::from_rgb(245, 245, 250)
                };
                egui::Frame::none()
                    .fill(frame_bg)
                    .rounding(6.0)
                    .shadow(egui::epaint::Shadow {
                        offset: [0, 4].into(),
                        blur: 12,
                        spread: 0,
                        color: egui::Color32::from_black_alpha(60),
                    })
                    .inner_margin(egui::Margin::symmetric(4, 4))
                    .show(ui, |ui| {
                        ui.set_min_width(220.0);
                        ui.label("Block types:");
                        ui.separator();

                        let mut selected: Option<BlockType> = None;
                        for (bt, name, desc) in &slash_types {
                            let resp = ui.selectable_label(false, format!("{}  —  {}", name, desc));
                            if resp.clicked() {
                                selected = Some(bt.clone());
                                close_menu = true;
                            }
                        }

                        if let Some(bt) = selected {
                            let block_id = editor_state.slash_menu_block_id.clone().unwrap_or_default();
                            editor_state.sync_buffer(&block_id, "");
                            actions.update_content.push(UpdateContentAction {
                                block_id: block_id.clone(),
                                content: String::new(),
                            });
                            actions.update_type.push(UpdateTypeAction {
                                block_id: block_id.clone(),
                                block_type: bt.clone(),
                            });
                            editor_state.slash_menu_open = false;
                            editor_state.slash_menu_block_id = None;
                        }
                    });
            });

        if close_menu || ui.input(|i| i.key_pressed(egui::Key::Escape)) {
            editor_state.slash_menu_open = false;
            editor_state.slash_menu_block_id = None;
        }
    }

    pub fn show(
        &mut self,
        ui: &mut egui::Ui,
        store: &SqliteStore,
        page_id: &str,
        editor_state: &mut PageEditorState,
        is_dark: bool,
    ) -> PageEditorActions {
        let mut actions = PageEditorActions::default();

        if self.previous_page_id.as_deref() != Some(page_id) {
            self.blocks = store.get_blocks(page_id).unwrap_or_default();
            if self.blocks.is_empty() {
                let id = store
                    .create_block(page_id, "text", "", None, None)
                    .unwrap_or_default();
                if let Ok(blocks) = store.get_blocks(page_id) {
                    self.blocks = blocks;
                }
                editor_state.active = true;
            }

            editor_state.clear();
            for block in &self.blocks {
                editor_state.sync_buffer(&block.id, &block.content);
            }
            editor_state.active = true;
            self.initialized = true;
            self.previous_page_id = Some(page_id.to_string());
        }

        if self.blocks.is_empty() {
            ui.label("This page is empty. Start writing...");
            return actions;
        }

        let scroll_id = egui::Id::new("page_editor_scroll").with(page_id);
        egui::ScrollArea::vertical()
            .id_source(scroll_id)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing.y = 2.0;

                let block_ids: Vec<String> = self.blocks.iter().map(|b| b.id.clone()).collect();

                let mut blocks_to_delete: Vec<usize> = Vec::new();
                let mut blocks_to_create_after: Vec<(usize, BlockType)> = Vec::new();
                let mut drop_idx: Option<usize> = None;

                for (i, block) in self.blocks.iter_mut().enumerate() {
                    let is_focused = editor_state
                        .focused_block_id
                        .as_deref()
                        .map_or(false, |id| id == &block.id);
                    let block_type = block.block_type_enum();
                    let mut text = editor_state
                        .get_buffer(&block.id, &block.content)
                        .clone();

                    let block_output = render_block(
                        ui,
                        block,
                        &block_type,
                        &mut text,
                        i,
                        is_focused,
                        is_dark,
                    );

                    if block_output.drag_started {
                        editor_state.drag_block_id = Some(block.id.clone());
                        editor_state.drag_start_idx = Some(i);
                    }

                    if let Some(new_content) = block_output.content_changed {
                        editor_state.sync_buffer(&block.id, &new_content);
                        if editor_state.slash_menu_open {
                            let bid = editor_state.slash_menu_block_id.clone().unwrap_or_default();
                            if new_content != "/" && block.id == bid {
                                editor_state.slash_menu_open = false;
                                editor_state.slash_menu_block_id = None;
                            }
                        }
                        actions
                            .update_content
                            .push(UpdateContentAction {
                                block_id: block.id.clone(),
                                content: new_content.clone(),
                            });
                        block.content = new_content;
                    }

                    if let Some(new_checked) = block_output.checked_changed {
                        actions
                            .update_props
                            .push(UpdatePropsAction {
                                block_id: block.id.clone(),
                                props: new_checked.to_string(),
                            });
                    }

                    if let Some(focus_id) = block_output.request_focus {
                        editor_state.focused_block_id = Some(focus_id);
                    }

                    if block_output.show_slash_menu && is_focused {
                        editor_state.slash_menu_open = true;
                        editor_state.slash_menu_block_id = Some(block.id.clone());
                    }

                    if block_output.request_delete {
                        blocks_to_delete.push(i);
                    }

                    if editor_state.slash_menu_open
                        && editor_state.slash_menu_block_id.as_deref() == Some(&block.id)
                    {
                        Self::render_slash_menu(ui, editor_state, &block.id, &mut actions, store, page_id);
                    }

                    let enter_pressed = ui.input(|i| {
                        i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                    });

                    let backspace_on_empty = ui.input(|i| {
                        i.key_pressed(egui::Key::Backspace) && text.is_empty()
                    });

                    if enter_pressed && is_focused && !editor_state.slash_menu_open {
                        blocks_to_create_after.push((i, BlockType::Text));
                    }

                    if backspace_on_empty && is_focused && i > 0 {
                        blocks_to_delete.push(i);
                    }

                    if let Some(drag_id) = &editor_state.drag_block_id {
                        if block.id != *drag_id {
                            let drop_zone = ui.allocate_space(egui::vec2(ui.available_width(), 4.0));
                            let drop_response = ui.interact(
                                drop_zone.1,
                                egui::Id::new("drop_zone").with(i).with("after"),
                                egui::Sense::click(),
                            );
                            if drop_response.hovered() && ui.ctx().input(|i| i.pointer.any_released()) {
                                drop_idx = Some(i);
                            }
                        }
                    }
                }

                for &idx in blocks_to_delete.iter().rev() {
                    let block_id = self.blocks[idx].id.clone();
                    actions.delete_blocks.push(block_id.clone());
                    editor_state.remove_buffer(&block_id);
                    self.blocks.remove(idx);
                }

                if let Some(drop_to) = drop_idx {
                    if let Some(drag_id) = &editor_state.drag_block_id {
                        let drag_from = self.blocks.iter().position(|b| b.id == *drag_id);
                        if let Some(from) = drag_from {
                            let mut new_order: Vec<String> = self.blocks.iter().map(|b| b.id.clone()).collect();
                            if from < drop_to {
                                let item = new_order.remove(from);
                                new_order.insert(drop_to, item);
                            } else {
                                let item = new_order.remove(from);
                                let adjusted_to = if drop_to > new_order.len() { new_order.len() } else { drop_to };
                                new_order.insert(adjusted_to, item);
                            }
                            let _ = store.reorder_blocks(page_id, &new_order);
                            if let Ok(blocks) = store.get_blocks(page_id) {
                                self.blocks = blocks;
                            }
                            actions.reorder = true;
                        }
                    }
                    editor_state.drag_block_id = None;
                    editor_state.drag_start_idx = None;
                }

                if ui.input(|i| i.pointer.any_released()) && editor_state.drag_block_id.is_some() {
                    editor_state.drag_block_id = None;
                    editor_state.drag_start_idx = None;
                }

                for (idx, bt) in blocks_to_create_after {
                    let new_id = store
                        .create_block(page_id, bt.to_db_string(), "", None, None)
                        .unwrap_or_default();
                    editor_state.sync_buffer(&new_id, "");
                    if let Ok(blocks) = store.get_blocks(page_id) {
                        self.blocks = blocks;
                    }
                    editor_state.focused_block_id = Some(new_id);
                    break;
                }

                let click_rect = ui.allocate_space(egui::vec2(ui.available_width(), 40.0));
                if ui.interact(click_rect.1, egui::Id::new("page_click").with(page_id), egui::Sense::click())
                    .clicked()
                {
                    let new_id = store
                        .create_block(page_id, "text", "", None, None)
                        .unwrap_or_default();
                    editor_state.sync_buffer(&new_id, "");
                    if let Ok(blocks) = store.get_blocks(page_id) {
                        self.blocks = blocks;
                    }
                    editor_state.focused_block_id = Some(new_id);
                }
            });

        actions
    }
}
