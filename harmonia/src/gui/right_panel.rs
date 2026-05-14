use eframe::egui;
use crate::comparaison::closest_note;

pub fn draw_right_panel(
    ui: &mut egui::Ui,
    note: &str,
    derror: &mut f32,
    detected_freq: Option<f32>,
    selected_string: usize,
) {
    ui.vertical(|ui| {
        ui.add_space(40.0);

        // Corde cible
        ui.label(
            egui::RichText::new(format!("Corde choisie : {}", note))
                .size(36.0)
                .color(egui::Color32::WHITE),
        );

        ui.add_space(20.0);

        // Note détectée
        match detected_freq {
            Some(freq) if selected_string < 6 => {
                let closest = closest_note(freq);
                ui.label(
                    egui::RichText::new(format!("Note détectée : {}  ({:.1} Hz)", closest.name, freq))
                        .size(22.0)
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
            }
            Some(freq) => {
                let closest = closest_note(freq);
                ui.label(
                    egui::RichText::new(format!("Note détectée : {}  ({:.1} Hz)", closest.name, freq))
                        .size(22.0)
                        .color(egui::Color32::from_rgb(180, 220, 255)),
                );
            }
            None => {
                ui.label(
                    egui::RichText::new("En attente d'un signal…")
                        .size(22.0)
                        .color(egui::Color32::from_rgb(150, 150, 150)),
                );
            }
        }

        ui.add_space(30.0);

        // Jauge d'accord
        let gauge_width = 340.0;
        let gauge_height = 30.0;
        let (gauge_rect, _) = ui.allocate_exact_size(
            egui::vec2(gauge_width, gauge_height + 60.0),
            egui::Sense::hover(),
        );

        let painter = ui.painter_at(gauge_rect);

        let bar_rect = egui::Rect::from_min_size(
            gauge_rect.min,
            egui::vec2(gauge_width, gauge_height),
        );

        // Fond de la barre
        painter.rect_filled(bar_rect, 6.0, egui::Color32::from_rgb(65, 65, 65));

        // Colorier en vert si bien accordé (< 5 cents)
        let bar_color = if selected_string < 6 && detected_freq.is_some() {
            if derror.abs() < 5.0 {
                egui::Color32::from_rgb(80, 200, 100)
            } else {
                egui::Color32::from_rgb(65, 65, 65)
            }
        } else {
            egui::Color32::from_rgb(65, 65, 65)
        };
        painter.rect_filled(bar_rect, 6.0, bar_color);

        // Ligne centrale
        painter.line_segment(
            [
                egui::pos2(bar_rect.center().x, bar_rect.min.y),
                egui::pos2(bar_rect.center().x, bar_rect.max.y),
            ],
            egui::Stroke::new(3.0, egui::Color32::from_rgb(220, 220, 220)),
        );

        // Curseur (flèche)
        let pointer_x = bar_rect.center().x + (*derror / 50.0) * (gauge_width / 2.0);
        let arrow_y = bar_rect.max.y + 15.0;

        let cursor_color = if selected_string < 6 && detected_freq.is_some() {
            if derror.abs() < 5.0 {
                egui::Color32::from_rgb(80, 200, 100)
            } else {
                egui::Color32::WHITE
            }
        } else {
            egui::Color32::from_rgb(100, 100, 100)
        };

        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(pointer_x, arrow_y - 12.0),
                egui::pos2(pointer_x - 12.0, arrow_y + 12.0),
                egui::pos2(pointer_x + 12.0, arrow_y + 12.0),
            ],
            cursor_color,
            egui::Stroke::NONE,
        ));

        // Valeur en cents sous le curseur
        if selected_string < 6 && detected_freq.is_some() {
            painter.text(
                egui::pos2(pointer_x, arrow_y + 25.0),
                egui::Align2::CENTER_CENTER,
                format!("{:+.0} ¢", derror),
                egui::FontId::proportional(18.0),
                cursor_color,
            );
        }

        // Étiquettes b / #
        painter.text(
            egui::pos2(bar_rect.min.x, arrow_y + 15.0),
            egui::Align2::LEFT_CENTER,
            "b",
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );

        painter.text(
            egui::pos2(bar_rect.max.x, arrow_y + 15.0),
            egui::Align2::RIGHT_CENTER,
            "#",
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );
    });
}
