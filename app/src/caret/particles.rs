use eframe::egui::{Color32, Pos2, Vec2};

#[derive(Clone, Copy)]
pub struct Particle {
    pub pos: Pos2,
    pub vel: Vec2,
    pub age: f32,
    pub life: f32,
    pub size: f32,
}

#[derive(Clone, Copy)]
pub struct Ripple {
    pub center: Pos2,
    pub age: f32,
}

#[derive(Clone)]
pub struct Bolt {
    pub pts: Vec<Pos2>,
    pub age: f32,
}

pub fn ember_color(t: f32) -> Color32 {
    let a = ((1.0 - t) * 220.0) as u8;
    if t < 0.25 {
        Color32::from_rgba_unmultiplied(255, 240, 150, a)
    } else if t < 0.60 {
        Color32::from_rgba_unmultiplied(255, 130, 20, a)
    } else {
        Color32::from_rgba_unmultiplied(180, 40, 10, a)
    }
}
