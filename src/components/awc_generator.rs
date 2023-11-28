use eframe::egui;

pub struct AwcGenerator {}

impl eframe::App for AwcGenerator {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("AWC Generator");
        });
    }
}
