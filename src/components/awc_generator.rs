use std::{cell::RefCell, rc::Rc};

use eframe::egui;
use egui::Id;

use crate::state::State;

pub struct AwcGenerator {
    state: Rc<RefCell<State>>,
    active_pack: String,
}

impl AwcGenerator {
    pub fn new(state: Rc<RefCell<State>>) -> Self {
        Self {
            state,
            active_pack: String::from(""),
        }
    }
}

impl eframe::App for AwcGenerator {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            let state = self.state.borrow();
            if state.active_project.is_none() {
                return;
            }
            let project = state.active_project.unwrap();
            egui::ComboBox::from_id_source(Id::new("awc_generator_pack_selector"))
                .selected_text(&self.active_pack)
                .show_ui(ui, |ui| {
                    let state = self.state.borrow();
                    for awc_pack in project.awc_info.iter() {
                        ui.selectable_label(self.active_pack == awc_pack.name, &awc_pack.name);
                    }
                });
        });
    }
}
