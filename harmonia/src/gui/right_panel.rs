use eframe::egui;

pub fn draw_right_panel(ui: &mut egui::Ui, note: &str, derror: &mut f32) {
    ui.vertical(|ui| {
        ui.add_space(40.0);

        ui.label(
            egui::RichText::new(format!("Corde choisie : {}", note))
                .size(36.0)
                .color(egui::Color32::WHITE),
        );

        ui.add_space(60.0);

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

        painter.rect_filled(bar_rect, 6.0, egui::Color32::from_rgb(65, 65, 65));
        painter.line_segment(
            [
                egui::pos2(bar_rect.center().x, bar_rect.min.y),
                egui::pos2(bar_rect.center().x, bar_rect.max.y),
            ],
            egui::Stroke::new(3.0, egui::Color32::from_rgb(220, 220, 220)),
        );

        let pointer_x = bar_rect.center().x + (*derror / 50.0) * (gauge_width / 2.0);
        let arrow_y = bar_rect.max.y + 15.0;

        painter.add(egui::Shape::convex_polygon(
            vec![
                egui::pos2(pointer_x, arrow_y - 12.0),
                egui::pos2(pointer_x - 12.0, arrow_y + 12.0),
                egui::pos2(pointer_x + 12.0, arrow_y + 12.0),
            ],
            egui::Color32::WHITE,
            egui::Stroke::NONE,
        ));

        painter.text(
            egui::pos2(pointer_x, arrow_y + 25.0),
            egui::Align2::CENTER_CENTER,
            format!("{:.0}", derror),
            egui::FontId::proportional(20.0),
            egui::Color32::WHITE,
        );

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

        // DEBUG
        ui.add_space(80.0);
        ui.label(egui::RichText::new("Outils de simulation (Debug):").color(egui::Color32::GRAY));
        ui.add(egui::Slider::new(derror, -50.0..=50.0).text("Simuler décalage (derror)"));
    });
}
