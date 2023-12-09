use std::{cell::RefCell, rc::Rc};

use eframe::egui;
use egui::RichText;
use log::error;

use crate::{project_mgmt::project::Project, state::State};

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
                    state.set_project(project);
                    state.change_menu(crate::state::MenuEntry::AwcGenerator);
                }
            });
        });
    }
}
