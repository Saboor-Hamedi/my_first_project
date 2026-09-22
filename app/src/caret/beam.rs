use eframe::egui::{vec2, Color32, Painter, Pos2, Rect};

pub fn paint_block(p: &Painter, pos: Pos2, cw: f32, lh: f32, color: Color32) {
    p.rect_filled(Rect::from_min_size(pos, vec2(cw, lh)), 0.0, color);
}

pub fn paint_beam(p: &Painter, pos: Pos2, w: f32, lh: f32, color: Color32) {
    p.rect_filled(Rect::from_min_size(pos, vec2(w, lh)), 0.0, color);
}

pub fn paint_underline(p: &Painter, pos: Pos2, cw: f32, w: f32, lh: f32, color: Color32) {
    let y = pos.y + lh - w;
    p.rect_filled(Rect::from_min_size(eframe::egui::pos2(pos.x, y), vec2(cw, w)), 0.0, color);
}
