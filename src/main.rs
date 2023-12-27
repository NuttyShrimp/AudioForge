use std::{cell::RefCell, rc::Rc};

use eframe::egui;
use log::error;
use state::{LoadedTabs, MenuEntry, State};
use strum::IntoEnumIterator;

mod components;
mod dat_files;
mod project_mgmt;
mod state;
mod utils;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();
    ffmpeg_next::init().unwrap();
    // std::env::set_var("WINIT_UNIX_BACKEND", "x11");

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([800.0, 600.0])
            .with_drag_and_drop(true),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "AudioForge",
        options,
        Box::new(|_cc| Box::<AudioForge>::default()),
    );

    Ok(())
}

struct AudioForge {
    state: Rc<RefCell<State>>,
    tabs: LoadedTabs,
}

impl Default for AudioForge {
    fn default() -> Self {
        let state = Rc::new(RefCell::new(State::default()));
        Self {
            state: state.clone(),
            tabs: LoadedTabs::new(state.clone()),
        }
    }
}

impl AudioForge {
    fn app_tab_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_nav_bar").show(ctx, |ui| {
            let mut state = self.state.borrow_mut();
            let selected_entry = state.active_menu;
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;

                ui.menu_button("File", |ui| {
                    // if ui.button("Open project").clicked() {} // With the recent opened projects
                    if ui.button("Close project").clicked() {
                        if let Some(project) = state.active_project.as_mut() {
                            let res = project.save();
                            if res.is_err() {
                                error!("Failed to save project: {:?}", res.unwrap_err());
                            }
                        }
                        state.close_project();
                        state.change_menu(MenuEntry::ProjectSelector);
                    }
                });

                for entry in MenuEntry::iter() {
                    if let Some(label) = entry.label() {
                        if ui
                            .selectable_label(selected_entry == entry, label)
                            .clicked()
                        {
                            state.change_menu(entry);
                        }
                    }
                }
            });
        });
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let state = self.state.borrow();
        let mut menu_entry = state.active_menu;
        if state.active_project.is_none() {
            menu_entry = MenuEntry::ProjectSelector;
        }
        // Dropping current borrow of state here so we can use it in our current component
        drop(state);
        menu_entry.render_entry(&mut self.tabs).update(ctx, frame);
    }
}

impl eframe::App for AudioForge {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Top tabbed panel
        self.app_tab_bar(ctx);
        self.show_selected_app(ctx, frame)
    }
    fn save(&mut self, _storage: &mut dyn eframe::Storage) {
        if let Some(project) = self.state.borrow_mut().active_project.as_mut() {
            let res = project.save();
            if res.is_err() {
                error!("Failed to save project: {:?}", res.unwrap_err());
            }
        }
    }
}
