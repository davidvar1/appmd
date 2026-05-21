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

                    if let Some(new_content) = block_output.content_changed {
                        editor_state.sync_buffer(&block.id, &new_content);
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

                    if block_output.request_delete {
                        blocks_to_delete.push(i);
                    }

                    let enter_pressed = ui.input(|i| {
                        i.key_pressed(egui::Key::Enter) && !i.modifiers.shift
                    });

                    let backspace_on_empty = ui.input(|i| {
                        i.key_pressed(egui::Key::Backspace) && text.is_empty()
                    });

                    if enter_pressed && is_focused {
                        blocks_to_create_after.push((i, BlockType::Text));
                    }

                    if backspace_on_empty && is_focused && i > 0 {
                        blocks_to_delete.push(i);
                    }
                }

                for &idx in blocks_to_delete.iter().rev() {
                    let block_id = self.blocks[idx].id.clone();
                    actions.delete_blocks.push(block_id.clone());
                    editor_state.remove_buffer(&block_id);
                    self.blocks.remove(idx);
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

                if ui.input(|i| i.key_pressed(egui::Key::Enter) && !ui.input(|i| i.modifiers.shift)) {
                    if editor_state.focused_block_id.is_none() && !self.blocks.is_empty() {
                        let last_id = self.blocks.last().unwrap().id.clone();
                        let new_id = store
                            .create_block(page_id, "text", "", None, None)
                            .unwrap_or_default();
                        editor_state.sync_buffer(&new_id, "");
                        if let Ok(blocks) = store.get_blocks(page_id) {
                            self.blocks = blocks;
                        }
                        editor_state.focused_block_id = Some(new_id);
                    }
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
