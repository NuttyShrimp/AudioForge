pub struct OcclGenerator {}

impl eframe::App for OcclGenerator {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.label("Occlussion generator");
        });
    }
}
