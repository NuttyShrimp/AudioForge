use std::{cell::RefCell, fs, path::Path, rc::Rc};

use anyhow::{anyhow, Result};
use eframe::egui;
use egui::{Button, DroppedFile, Id, Window};
use egui_extras::{Column, TableBuilder};
use ffmpeg_next::{
    format::{self},
    media,
};
use itertools::Itertools;
use log::error;
use strum::IntoEnumIterator;

use crate::{
    project_mgmt::awc::{self, AwcPackType},
    state::State,
    utils::transcoder,
};

use super::inputs;

pub struct AwcGenerator {
    state: Rc<RefCell<State>>,
    active_pack: usize,
    creator_window_state: AwcPackCreation,
    // Map of awc entry indexes where the header editor windows should be shown for
    // TODO: Are we able to refactor to only store the active indexes?
    header_editor_window: Vec<bool>,
}

#[derive(Default)]
struct AwcPackCreation {
    visible: bool,
    name: String,
    pack_type: awc::AwcPackType,
}

impl AwcGenerator {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self {
            state,
            active_pack: 0,
            creator_window_state: AwcPackCreation::default(),
            header_editor_window: vec![],
        }
    }

    fn create_audio_pack_windows(&mut self, ctx: &egui::Context, show_create_window: &mut bool) {
        Window::new("Create new audio pack")
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .default_size([600.0, 300.0])
            .open(show_create_window)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    let name_label = ui.label("Pack name");
                    ui.text_edit_singleline(&mut self.creator_window_state.name)
                        .labelled_by(name_label.id);
                });
                ui.horizontal(|ui| {
                    let pack_label = ui.label("Pack type");
                    egui::ComboBox::from_id_source(pack_label.id)
                        .width(200.0)
                        .selected_text(self.creator_window_state.pack_type.to_string())
                        .show_ui(ui, |ui| {
                            for option in AwcPackType::iter() {
                                ui.selectable_value(
                                    &mut self.creator_window_state.pack_type,
                                    option,
                                    option.to_string(),
                                );
                            }
                        });
                });
                if ui.button("Create").clicked() {
                    let pack = awc::AwcPack {
                        name: self.creator_window_state.name.clone(),
                        pack_type: self.creator_window_state.pack_type,
                        entries: vec![],
                    };
                    if let Some(project) = self.state.borrow_mut().active_project.as_mut() {
                        project.add_awc_pack(pack);
                        self.creator_window_state.visible = false;
                    }
                }
            });
    }

    fn edit_entry_header_window(&mut self, ctx: &egui::Context, awc_entry_index: usize) {
        if self.header_editor_window.len() <= awc_entry_index {
            return;
        }

        let mut state = self.state.borrow_mut();
        if state.active_project.is_none() {
            return;
        }
        let project = state.active_project.as_mut().unwrap();
        let awc_pack = &mut project.awc_info[self.active_pack];
        let awc_entry = &mut awc_pack.entries[awc_entry_index];

        Window::new(format!("Edit entry headers {}", awc_entry.name))
            .title_bar(true)
            .collapsible(false)
            .resizable(false)
            .default_size([600.0, 300.0])
            .open(&mut self.header_editor_window[awc_entry_index])
            .show(ctx, |ui| {
                let headers = &mut awc_entry.headers;
                // TODO: Can this be done via loops?
                ui.horizontal(|ui| {
                    let label = ui.label("Category");
                    ui.text_edit_singleline(&mut headers.category)
                        .labelled_by(label.id);
                });

                ui.horizontal(|ui| {
                    let label = ui.label("Volume");
                    ui.add(egui::widgets::DragValue::new(&mut headers.volume))
                        .labelled_by(label.id)
                        .on_hover_ui(|ui| {
                            ui.label("each 100 represent 10% in volume increase, recommended to be 100, max 65535");
                        });
                });

                ui.horizontal(|ui| {
                    let label = ui.label("Volume Curve Hash (AKA Rolloff Hash)");
                    ui.text_edit_singleline(&mut headers.volume_curve)
                        .labelled_by(label.id)
                        .on_hover_ui(|ui| {
                            ui.label("Distance attenuation curves");
                        });
                });

                inputs::drag_value(ui, "Volume Distance", &mut headers.volume_curve_distance, Some("0 - 65535, How for the sound can be heard"));
                inputs::optional_drag_value(ui, "Doppler Factor", &mut headers.doppler_factor, Some("0 - 65535"));
                inputs::optional_drag_value(ui, "Attack Time", &mut headers.attack_time, Some("Fade-in time, 0 - 65535"));
                inputs::optional_drag_value(ui, "Release Time", &mut headers.release_time, Some("Fade-out time, 0 - 65535"));
                inputs::drag_value(ui, "Stereo Panning", &mut headers.unk20, Some("0 for stereo, see Monkeys audio research for other options"));
                inputs::optional_drag_value(ui, "Echo x", &mut headers.echo_x, None);
                inputs::optional_drag_value(ui, "Echo y", &mut headers.echo_y, None);
                inputs::optional_drag_value(ui, "Echo z", &mut headers.echo_z, None);
            });
    }

    fn show_awc_entry_table(&mut self, ui: &mut egui::Ui) {
        let mut state = self.state.borrow_mut();
        if state.active_project.is_none() {
            return;
        }
        let project = state.active_project.as_mut().unwrap();
        if project.awc_info.len() <= self.active_pack {
            return;
        }
        egui::ScrollArea::horizontal().show(ui, |ui| {
            let text_height = egui::TextStyle::Body.resolve(ui.style()).size;

            let table = TableBuilder::new(ui)
                .striped(true)
                .resizable(true)
                .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::auto())
                .column(Column::remainder());

            table
                .header(20.0, |mut header| {
                    header.col(|ui| {
                        ui.strong("Name");
                    });
                    header.col(|ui| {
                        ui.strong("Looped");
                    });
                    header.col(|ui| {
                        ui.strong("Headers");
                    });
                    header.col(|ui| {
                        ui.label("");
                    });
                })
                .body(|body| {
                    body.rows(
                        text_height,
                        project.awc_info[self.active_pack].entries.len(),
                        |row_index, mut row| {
                            let entry = &mut project.awc_info[self.active_pack].entries[row_index];
                            row.col(|ui| {
                                ui.label(&entry.name);
                            });
                            row.col(|ui| {
                                ui.checkbox(&mut entry.looped, "Looped");
                            });
                            row.col(|ui| {
                                if ui.button("Headers").clicked() {
                                    if self.header_editor_window.len() <= row_index {
                                        self.header_editor_window.resize(row_index + 1, false);
                                    }
                                    self.header_editor_window[row_index] = true;
                                };
                            });
                            row.col(|ui| {
                                if ui.button("Delete").clicked() {
                                    let _ = project.awc_info[self.active_pack]
                                        .entries
                                        .remove(row_index);
                                };
                            });
                        },
                    );
                });
        });
    }
}

impl eframe::App for AwcGenerator {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut show_create_window = self.creator_window_state.visible;
        if show_create_window {
            self.create_audio_pack_windows(ctx, &mut show_create_window);
        }
        self.creator_window_state.visible &= show_create_window;

        let state = self.state.borrow();
        if state.active_project.is_none() {
            return;
        }
        let project = state.active_project.as_ref().unwrap();
        let is_awc_pack_selected = project.awc_info.is_empty();

        if project.awc_info.len() > 0 {
            let pack_count = project.awc_info[self.active_pack].entries.len();
            drop(state);

            for i in 0..pack_count {
                self.edit_entry_header_window(ctx, i);
            }
        } else {
            drop(state);
        }

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let state = self.state.borrow();
                if state.active_project.is_none() {
                    return;
                }
                let project = state.active_project.as_ref().unwrap();
                let project_loc = project.location.clone();

                if project.awc_info.len() > 0 {
                    egui::ComboBox::from_id_source(Id::new("awc_generator_pack_selector"))
                        .selected_text(&project.awc_info[self.active_pack].name)
                        .show_ui(ui, |ui| {
                            project
                                .awc_info
                                .iter()
                                .group_by(|e| e.pack_type)
                                .into_iter()
                                .for_each(|(key, group)| {
                                    ui.add(egui::Label::new(key.to_string()).wrap(false));
                                    for (awc_pack, i) in group.into_iter().zip(0..) {
                                        ui.selectable_value(
                                            &mut self.active_pack,
                                            i,
                                            &awc_pack.name,
                                        );
                                    }
                                });
                        });
                } else {
                    ui.add_enabled_ui(false, |ui| {
                        egui::ComboBox::from_id_source(Id::new("awc_generator_pack_selector"))
                            .show_ui(ui, |_ui| {});
                    });
                }

                if ui.button("New audio pack").clicked() {
                    self.creator_window_state.name = String::from("");
                    self.creator_window_state.visible = true;
                }

                drop(state);
                if !is_awc_pack_selected {
                    if ui.button("Add audio file").clicked() {
                        // TODO: Make this usable in spawnable thread so render thread is not blocked
                        if let Some(paths) = rfd::FileDialog::new()
                            .set_title("Select to be added audio files")
                            .pick_files()
                        {
                            for path in paths {
                                if self::AwcGenerator::validate_file(&path).is_ok() {
                                    let process = self.import_file(&path);
                                    if process.is_err() {
                                        error!("{:?}", process.unwrap_err());
                                    }
                                };
                            }
                        }
                    }
                }
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button("Generate FiveM resource").clicked() {
                        let _ = fs::create_dir_all(project_loc.join("output/awc_resource"));
                        let state = self.state.borrow();
                        if state.active_project.is_none() {
                            return;
                        }
                        let project = state.active_project.as_ref().unwrap();
                        project.generate_awc_file(self.active_pack);

                        // our AwcXML struct serialized to xml
                        // TODO: generate dat54 file
                    }
                });
            });

            if is_awc_pack_selected {
                return;
            }

            self.show_awc_entry_table(ui);
            ui.set_min_height(ui.available_height());

            ctx.input(|i| {
                for file in i.raw.dropped_files.iter() {
                    if let Some(path) = &file.path {
                        if self::AwcGenerator::validate_file(path).is_ok() {
                            let process = self.import_file(path);
                            if process.is_err() {
                                error!("{:?}", process.unwrap_err());
                            }
                        };
                    };
                }
            });
        });
    }
}

impl AwcGenerator {
    fn validate_file(path: &Path) -> Result<()> {
        let input_format = format::input(&path)?;
        let input = input_format.streams().best(media::Type::Audio);
        if input.is_none() {
            return Err(anyhow!(
                "File does not contain audio stream: {}",
                path.display().to_string()
            ));
        }
        Ok(())
    }

    fn import_file(&mut self, path: &Path) -> Result<()> {
        let mut state = self.state.borrow_mut();
        let project = state.active_project.as_mut().unwrap();
        let awc_pack = &project.awc_info[self.active_pack];

        let entry_name = &path.file_stem().unwrap().to_string_lossy().to_string();
        let output_dir = project
            .location
            .clone()
            .join("awc_packs")
            .join(awc_pack.name.clone());
        fs::create_dir_all(output_dir.as_path())?;

        transcoder::encode_to_wav(path, &output_dir)?;

        let proj_loc = project.location.clone();
        project.get_mut_entries_slice()[self.active_pack].add_entry(
            &proj_loc,
            &output_dir,
            entry_name,
        )?;

        Ok(())
    }
}
