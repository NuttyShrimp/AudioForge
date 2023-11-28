use eframe::egui;
use state::{LoadedTabs, MenuEntry, State};
use strum::IntoEnumIterator;

mod components;
mod state;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([800.0, 600.0]),
        ..Default::default()
    };

    let _ = eframe::run_native(
        "AudioForge",
        options,
        Box::new(|_cc| Box::<AudioForge>::default()),
    );

    println!("Hello, world!");
    Ok(())
}

#[derive(Default)]
struct AudioForge {
    state: State,
    tabs: LoadedTabs,
}

impl AudioForge {
    fn top_bar(&mut self, ctx: &egui::Context) {
        egui::TopBottomPanel::top("top_nav_bar").show(ctx, |ui| {
            let selected_entry = self.state.active_menu;
            ui.horizontal_wrapped(|ui| {
                ui.visuals_mut().button_frame = false;

                for entry in MenuEntry::iter() {
                    if ui
                        .selectable_label(selected_entry == entry, entry.label())
                        .clicked()
                    {
                        self.state.change_menu(entry);
                    }
                }
            });
        });
    }

    fn show_selected_app(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        let app = self.state.active_menu.render_entry(&mut self.tabs);
        app.update(ctx, frame);
    }
}

impl eframe::App for AudioForge {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        // Top tabbed panel
        self.top_bar(ctx);
        self.show_selected_app(ctx, frame)
    }
}
