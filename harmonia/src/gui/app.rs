use eframe::egui;
use super::left_panel::draw_left_panel;
use super::right_panel::draw_right_panel;
use crate::notes::GUITAR;
use crate::comparaison::cents_deviation;
use std::sync::{Arc, Mutex};

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
    pub detected_freq: Option<f32>,
    shared_freq: Arc<Mutex<Option<f32>>>,
}

impl HarmoniaApp {
    pub fn new(shared_freq: Arc<Mutex<Option<f32>>>) -> Self {
        Self {
            selected_string: 6,
            derror: 0.0,
            detected_freq: None,
            shared_freq,
        }
    }
}

impl eframe::App for HarmoniaApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Lire la dernière fréquence détectée par le thread audio
        if let Ok(lock) = self.shared_freq.try_lock() {
            self.detected_freq = *lock;
        }

        // Calculer l'écart en cents si une corde est sélectionnée
        if self.selected_string < 6 {
            if let Some(freq) = self.detected_freq {
                let target = GUITAR[self.selected_string].frequency;
                self.derror = cents_deviation(freq, target).clamp(-50.0, 50.0);
            }
        } else {
            self.derror = 0.0;
        }

        // Redessiner en continu pour afficher les mises à jour audio
        ctx.request_repaint();

        let mut style = (*ctx.style()).clone();
        style.visuals.panel_fill = egui::Color32::from_rgb(85, 85, 85);
        ctx.set_style(style);

        egui::CentralPanel::default().show(ctx, |ui| {
            let panel_height = ui.available_height();
            ui.horizontal(|ui| {
                draw_left_panel(ui, panel_height, &mut self.selected_string);
                ui.add_space(20.0);
                draw_right_panel(
                    ui,
                    &STRINGS[self.selected_string].note,
                    &mut self.derror,
                    self.detected_freq,
                    self.selected_string,
                );
            });
        });
    }
}
