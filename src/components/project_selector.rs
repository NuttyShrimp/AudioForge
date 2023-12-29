use std::{cell::RefCell, path::PathBuf, rc::Rc};

use eframe::egui;
use egui::RichText;
use log::error;

use crate::{
    project_mgmt::project::{add_to_recent_projects, Project},
    state::State,
};

pub struct ProjectSelector {
    state: Rc<RefCell<State>>,
}

impl ProjectSelector {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self { state }
    }
}

impl eframe::App for ProjectSelector {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.vertical_centered(|ui| {
                ui.label(RichText::new("GTA V Audio Toolkit").size(24.0));
                ui.add_space(10.0);
                if ui.button("Open Project").clicked() {
                    let select_res = Project::choose_project();
                    if select_res.is_err() {
                        // TODO: replace with error dialog
                        error!("{:?}", select_res.unwrap_err());
                        return;
                    }
                    // TODO: add project to recent projects in frame storage
                    let project = select_res.unwrap();
                    let mut state = self.state.borrow_mut();

                    add_to_recent_projects(frame, project.location.clone());

                    state.set_project(project);
                    state.change_menu(crate::state::MenuEntry::AwcGenerator);
                }

                let recent_projects_str = frame.storage().unwrap().get_string("project_history");
                let mut recent_projects = Vec::<PathBuf>::new();
                if let Some(paths) = recent_projects_str {
                    recent_projects = serde_json::from_str(&paths).unwrap();
                }

                // TODO: make it not fill all the available width space
                ui.group(|ui| {
                    ui.label(RichText::new("Recent projects").size(16.0));
                    ui.separator();

                    for proj in recent_projects {
                        if ui.selectable_label(false, proj.to_string_lossy()).clicked() {
                            let proj = Project::open_project(&proj);
                            if proj.is_err() {
                                error!("failed to select recent project: {:?}", proj.unwrap_err());
                                continue;
                            }

                            let mut state = self.state.borrow_mut();
                            state.set_project(proj.unwrap());
                            state.change_menu(crate::state::MenuEntry::AwcGenerator);
                        };
                    }
                });
            });
        });
    }
}
