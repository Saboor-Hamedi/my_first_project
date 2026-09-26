use eframe::egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Stroke, StrokeKind};

pub fn paint_neon(p: &Painter, pos: Pos2, w: f32, lh: f32, accent: Color32) {
    // Subtle laser glow hugging the beam without spilling onto the character to the left
    let glow_pads = [(0.8, 50u8), (1.6, 25u8)];
    for (pad, alpha) in glow_pads {
        let outer = Rect::from_min_size(
            pos2(pos.x - pad * 0.5, pos.y - pad),
            vec2(w + pad, lh + pad * 2.0),
        );
        p.rect_filled(
            outer,
            1.5,
            Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha),
        );
    }
    // Crisp vibrant laser core
    let inner = Rect::from_min_size(pos, vec2(w, lh));
    p.rect_filled(inner, 1.0, Color32::WHITE);
    p.rect_stroke(inner, 1.0, Stroke::new(1.0_f32, accent), StrokeKind::Inside);
}
