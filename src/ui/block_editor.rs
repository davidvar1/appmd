use crate::store::block_types::BlockType;
use crate::store::Block;
use eframe::egui;

#[derive(Debug, Clone)]
pub struct BlockEditorOutput {
    pub block_id: String,
    pub content_changed: Option<String>,
    pub block_type_changed: Option<BlockType>,
    pub checked_changed: Option<bool>,
    pub request_new_block_after: Option<String>,
    pub request_delete: bool,
    pub request_focus: Option<String>,
}

pub fn render_block(
    ui: &mut egui::Ui,
    block: &Block,
    block_type: &BlockType,
    text: &mut String,
    index: usize,
    is_focused: bool,
    is_dark: bool,
) -> BlockEditorOutput {
    let block_id = block.id.clone();
    let mut output = BlockEditorOutput {
        block_id: block_id.clone(),
        content_changed: None,
        block_type_changed: None,
        checked_changed: None,
        request_new_block_after: None,
        request_delete: false,
        request_focus: None,
    };

    let accent = if is_dark {
        egui::Color32::from_rgb(203, 166, 247)
    } else {
        egui::Color32::from_rgb(130, 80, 223)
    };

    let bg_hover = if is_dark {
        egui::Color32::from_rgb(30, 30, 40)
    } else {
        egui::Color32::from_rgb(245, 245, 250)
    };

    egui::Frame::none()
        .inner_margin(egui::Margin::symmetric(4, 2))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.set_min_height(28.0);

                match block_type {
                    BlockType::Heading1 => {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Heading)
                                .desired_width(f32::INFINITY)
                                .text_color(if is_dark {
                                    egui::Color32::from_rgb(220, 220, 255)
                                } else {
                                    egui::Color32::from_rgb(30, 30, 80)
                                })
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::Heading2 => {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Name("heading".into()))
                                .desired_width(f32::INFINITY)
                                .text_color(if is_dark {
                                    egui::Color32::from_rgb(200, 200, 240)
                                } else {
                                    egui::Color32::from_rgb(40, 40, 100)
                                })
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::Heading3 => {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Name("small_heading".into()))
                                .desired_width(f32::INFINITY)
                                .text_color(if is_dark {
                                    egui::Color32::from_rgb(180, 180, 220)
                                } else {
                                    egui::Color32::from_rgb(60, 60, 120)
                                })
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::Todo => {
                        let checked = block.props.parse::<bool>().unwrap_or(false);
                        let mut new_checked = checked;
                        ui.checkbox(&mut new_checked, "");
                        if new_checked != checked {
                            output.checked_changed = Some(new_checked);
                        }

                        let style = if checked {
                            egui::TextStyle::Body
                        } else {
                            egui::TextStyle::Body
                        };
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(style)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::BulletedList => {
                        ui.label("  \u{2022}  ");
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Body)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::NumberedList => {
                        ui.label(format!("  {}.  ", index + 1));
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Body)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::Toggle => {
                        ui.label("  \u{25B6}  ");
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Body)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                    BlockType::Code => {
                        ui.vertical(|ui| {
                            let code_bg = if is_dark {
                                egui::Color32::from_rgb(25, 25, 35)
                            } else {
                                egui::Color32::from_rgb(240, 240, 245)
                            };
                            egui::Frame::none()
                                .fill(code_bg)
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    let response = ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::multiline(text)
                                            .font(egui::TextStyle::Monospace)
                                            .desired_width(f32::INFINITY)
                                            .code_editor()
                                            .id(egui::Id::new("block_edit").with(&block_id)),
                                    );
                                    if response.changed() {
                                        output.content_changed = Some(text.clone());
                                    }
                                });
                        });
                    }
                    BlockType::Quote => {
                        let bar_color = if is_dark {
                            egui::Color32::from_rgb(100, 180, 100)
                        } else {
                            egui::Color32::from_rgb(80, 160, 80)
                        };
                        ui.vertical(|ui| {
                            egui::Frame::none()
                                .inner_margin(egui::Margin::symmetric(8, 4))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        let (rect, _) =
                                            ui.allocate_exact_size(egui::vec2(3.0, 0.0), egui::Sense::hover());
                                        ui.painter().rect_filled(
                                            egui::Rect::from_min_max(
                                                rect.min,
                                                egui::pos2(rect.max.x, rect.max.y.max(ui.min_rect().bottom())),
                                            ),
                                            0.0,
                                            bar_color,
                                        );
                                        let response = ui.add_sized(
                                            ui.available_size(),
                                            egui::TextEdit::multiline(text)
                                                .font(egui::TextStyle::Body)
                                                .desired_width(f32::INFINITY)
                                                .id(egui::Id::new("block_edit").with(&block_id)),
                                        );
                                        if response.changed() {
                                            output.content_changed = Some(text.clone());
                                        }
                                    });
                                });
                        });
                    }
                    BlockType::Divider => {
                        ui.separator();
                    }
                    BlockType::Callout => {
                        let callout_bg = if is_dark {
                            egui::Color32::from_rgb(35, 35, 50)
                        } else {
                            egui::Color32::from_rgb(240, 240, 250)
                        };
                        egui::Frame::none()
                            .fill(callout_bg)
                            .rounding(4.0)
                            .inner_margin(egui::Margin::symmetric(8, 4))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    let _icon = block.props.as_str();
                                    if _icon.is_empty() || _icon == "{}" {
                                        ui.label("\u{2139}\u{FE0F}  ");
                                    } else {
                                        ui.label(format!("{}  ", _icon));
                                    }
                                    let response = ui.add_sized(
                                        ui.available_size(),
                                        egui::TextEdit::multiline(text)
                                            .font(egui::TextStyle::Body)
                                            .desired_width(f32::INFINITY)
                                            .id(egui::Id::new("block_edit").with(&block_id)),
                                    );
                                    if response.changed() {
                                        output.content_changed = Some(text.clone());
                                    }
                                });
                            });
                    }
                    BlockType::Image | BlockType::Bookmark | BlockType::Equation => {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Body)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                    }
                    BlockType::Text => {
                        let response = ui.add_sized(
                            ui.available_size(),
                            egui::TextEdit::multiline(text)
                                .font(egui::TextStyle::Body)
                                .desired_width(f32::INFINITY)
                                .id(egui::Id::new("block_edit").with(&block_id)),
                        );
                        if response.changed() {
                            output.content_changed = Some(text.clone());
                        }
                        if response.lost_focus() && text.is_empty() {
                            output.request_delete = true;
                        }
                    }
                }
            });
        });

    output
}
