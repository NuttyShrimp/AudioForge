use std::{cell::RefCell, fs, path::Path, rc::Rc};

use anyhow::{anyhow, Result};
use eframe::egui;
use egui::{DroppedFile, Id, Window};
use egui_extras::{Column, TableBuilder};
use ffmpeg_next::{
    format::{self},
    media,
};
use itertools::Itertools;
use log::{error, info};
use strum::IntoEnumIterator;

use crate::{
    project_mgmt::awc::{self, AwcPackType},
    state::State,
    utils::transcoder,
};

pub struct AwcGenerator {
    state: Rc<RefCell<State>>,
    active_pack: usize,
    creator_window_state: AwcPackCreation,
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
        }
    }
}

impl AwcGenerator {
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
                        .selected_text(&self.creator_window_state.pack_type.to_string())
                        .show_ui(ui, |ui| {
                            for option in AwcPackType::iter() {
                                ui.selectable_value(
                                    &mut self.creator_window_state.pack_type,
                                    option.clone(),
                                    option.to_string(),
                                );
                            }
                        });
                });
                if ui.button("Create").clicked() {
                    let pack = awc::AwcPack {
                        name: self.creator_window_state.name.clone(),
                        pack_type: self.creator_window_state.pack_type.clone(),
                        entries: vec![],
                    };
                    if let Some(project) = self.state.borrow_mut().active_project.as_mut() {
                        project.add_awc_pack(pack);
                        self.creator_window_state.visible = false;
                    }
                }
            });
    }

    fn show_awc_entry_table(&self, ui: &mut egui::Ui) {
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
                                    info!("OPen headers popup")
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

        egui::CentralPanel::default().show(ctx, |ui| {
            ui.horizontal(|ui| {
                let state = self.state.borrow();
                if state.active_project.is_none() {
                    return;
                }
                let project = state.active_project.as_ref().unwrap();

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
                                    ui.selectable_value(&mut self.active_pack, i, &awc_pack.name);
                                }
                            });
                    });

                if ui.button("New audio pack").clicked() {
                    self.creator_window_state.name = String::from("");
                    self.creator_window_state.visible = true;
                }

                if project.awc_info.len() == 0 {
                    return;
                }

                drop(state);

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
            });

            let state = self.state.borrow();
            if state.active_project.is_none() {
                return;
            }
            let project = state.active_project.as_ref().unwrap();
            if project.awc_info.len() == 0 {
                return;
            }
            drop(state);

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

        transcoder::encode_to_wav(&path, &output_dir)?;

        let proj_loc = project.location.clone();
        project.get_mut_entries_slice()[self.active_pack].add_entry(
            &proj_loc,
            &output_dir,
            entry_name,
        )?;

        Ok(())
    }

    // TODO: parameter should be changed to AwcEtry
    #[allow(dead_code)]
    fn split_file(&mut self, file: &DroppedFile) -> Result<()> {
        if let None = &file.path {
            return Err(anyhow!("Invalid dropped file: not path"));
        }
        let path = file.path.as_ref().unwrap();
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

        transcoder::split_stereo_to_mono(&path, output_dir.as_path())?;

        let proj_loc = project.location.clone();
        project.get_mut_entries_slice()[self.active_pack].add_entry(
            &proj_loc,
            &output_dir,
            entry_name,
        )?;

        Ok(())
    }
}
