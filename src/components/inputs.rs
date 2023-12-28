use egui::{Ui};


pub fn optional_drag_value<T: eframe::emath::Numeric>(
    ui: &mut Ui,
    label: &str,
    val: &mut Option<T>,
    tooltip: Option<&str>,
) {
    let res = ui.add_enabled_ui(val.is_some(), |ui| {
        ui.horizontal(|ui| {
            let mut temp_val = 0;
            let mut widget = egui::widgets::DragValue::new(&mut temp_val);

            if let Some(factor) = val.as_mut() {
                widget = egui::widgets::DragValue::new(factor);
            }

            let label = ui.label(label);
            let res = ui.add(widget).labelled_by(label.id);

            if let Some(tooltip_label) = tooltip {
                res.clone().on_hover_ui(|ui| {
                    ui.label(tooltip_label);
                });
            }
            res.on_disabled_hover_text("Double click to enable/disable");
        });
    });

    if ui
        .interact(res.response.rect, res.response.id, egui::Sense::click())
        .double_clicked()
    {
        if val.is_some() {
            val.take();
        } else {
            val.replace(T::MIN);
        }
    }
}

pub fn drag_value<T: eframe::emath::Numeric>(
    ui: &mut Ui,
    label: &str,
    val: &mut T,
    tooltip: Option<&str>,
) {
    ui.horizontal(|ui| {
        let label = ui.label(label);
        let res = ui
            .add(egui::widgets::DragValue::new(val))
            .labelled_by(label.id);
        if let Some(tooltip_label) = tooltip {
            res.on_hover_ui(|ui| {
                ui.label(tooltip_label);
            });
        }
    });
}
