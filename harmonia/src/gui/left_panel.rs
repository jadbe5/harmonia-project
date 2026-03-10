use eframe::egui;
use super::app::STRINGS;

pub fn draw_left_panel(ui: &mut egui::Ui, panel_height: f32, selected_string: &mut usize) {
    let left_panel_width = 460.0;
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(left_panel_width, panel_height),
        egui::Sense::hover(),
    );

    ui.painter().rect_filled(rect, 0.0, egui::Color32::from_rgb(58, 58, 58));

    let image_width = 367.0 * 0.8;
    let image_height = 687.0 * 0.8;
    let image_center = egui::pos2(
        rect.center().x,
        rect.min.y + (image_height / 2.0) + 20.0,
    );
    let image_rect = egui::Rect::from_center_size(
        image_center,
        egui::vec2(image_width, image_height),
    );

    let image_source = match *selected_string {
        0 => egui::include_image!("../../assets/selection/E2.png"),
        1 => egui::include_image!("../../assets/selection/A2.png"),
        2 => egui::include_image!("../../assets/selection/D3.png"),
        3 => egui::include_image!("../../assets/selection/G3.png"),
        4 => egui::include_image!("../../assets/selection/B3.png"),
        5 => egui::include_image!("../../assets/selection/E4.png"),
        _ => egui::include_image!("../../assets/tete_guitare.png"),
    };

    egui::Image::new(image_source).paint_at(ui, image_rect);

    let btn_radius = 22.0;
    let dist_btn = 180.0;

    for i in 0..6 {
        let is_left = match i { 0 | 1 | 2 => -1.0, _ => 1.0 };
        let x_offset = is_left * dist_btn;
        let y_offset = match i {
            0 | 5 => -21.0,
            1 | 4 => -105.0,
            _ => -190.0,
        };

        let btn_center = egui::pos2(image_center.x + x_offset, image_center.y + y_offset);
        let delta = 70.0;

        let hit_rect = if is_left < 0.0 {
            egui::Rect::from_min_max(
                egui::pos2(btn_center.x - btn_radius, btn_center.y - btn_radius),
                egui::pos2(btn_center.x + btn_radius + delta, btn_center.y + btn_radius),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(btn_center.x - btn_radius - delta, btn_center.y - btn_radius),
                egui::pos2(btn_center.x + btn_radius, btn_center.y + btn_radius),
            )
        };

        // DEBUG
        ui.painter().rect_stroke(hit_rect, 0.0, egui::Stroke::new(2.0, egui::Color32::RED));

        let interact = ui.interact(hit_rect, ui.id().with(i), egui::Sense::click());

        if interact.clicked() { *selected_string = i; }
        if interact.hovered() { ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand); }

        let (circle_color, text_color) = if *selected_string == i {
            (egui::Color32::WHITE, egui::Color32::from_rgb(58, 58, 58))
        } else if interact.hovered() {
            (egui::Color32::from_rgb(130, 130, 130), egui::Color32::WHITE)
        } else {
            (egui::Color32::from_rgb(90, 90, 90), egui::Color32::WHITE)
        };

        ui.painter().circle_filled(btn_center, btn_radius, circle_color);
        ui.painter().text(
            btn_center,
            egui::Align2::CENTER_CENTER,
            STRINGS[i].name,
            egui::FontId::proportional(20.0),
            text_color,
        );
    }
}
