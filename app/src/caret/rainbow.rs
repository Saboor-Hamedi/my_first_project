use eframe::egui::{vec2, Color32, Painter, Pos2, Rect};

pub fn paint_rainbow(p: &Painter, pos: Pos2, w: f32, lh: f32, hue: f32) {
    let c = Color32::from(eframe::egui::ecolor::Hsva::new(hue, 0.9, 1.0, 1.0));
    p.rect_filled(Rect::from_min_size(pos, vec2(w, lh)), 0.0, c);
}
