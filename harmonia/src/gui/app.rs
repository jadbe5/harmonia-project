use eframe::egui;
use super::left_panel::draw_left_panel;
use super::right_panel::draw_right_panel;

pub struct StringInfo {
    pub name: &'static str,
    pub note: &'static str,
}

pub const STRINGS: [StringInfo; 7] = [
    StringInfo { name: "E", note: "Mi grave (E2)" },
    StringInfo { name: "A", note: "La (A2)" },
    StringInfo { name: "D", note: "Ré (D3)" },
    StringInfo { name: "G", note: "Sol (G3)" },
    StringInfo { name: "B", note: "Si (B3)" },
    StringInfo { name: "E", note: "Mi aigu (E4)" },
    StringInfo { name: "",  note: "" },
];

pub struct HarmoniaApp {
    pub selected_string: usize,
    pub derror: f32,
}

impl Default for HarmoniaApp {
    fn default() -> Self {
        Self {
            selected_string: 6,
            derror: 0.0,
        }
    }
}

impl eframe::App for HarmoniaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = egui::Color32::from_rgb(85, 85, 85);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_height = ui.available_height();
            ui.horizontal(|ui| {
                draw_left_panel(ui, panel_height, &mut self.selected_string);
                ui.add_space(20.0);
                draw_right_panel(ui, &STRINGS[self.selected_string].note, &mut self.derror);
            });
        });
    }
}
