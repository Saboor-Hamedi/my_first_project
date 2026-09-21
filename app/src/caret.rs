use eframe::egui::{self, pos2, vec2, Color32, Pos2, Rect, Stroke, Vec2};
use std::collections::VecDeque;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaretKind {
    Block,
    Beam,
    Underline,
    Fire,
    Water,
    Electric,
    Comet,
    Rainbow,
    Matrix,
    Ice,
    Glitch,
    Neon,
    Heartbeat,
}

impl CaretKind {
    pub const ALL: &'static [CaretKind] = &[
        CaretKind::Block,
        CaretKind::Beam,
        CaretKind::Underline,
        CaretKind::Fire,
        CaretKind::Water,
        CaretKind::Electric,
        CaretKind::Comet,
        CaretKind::Rainbow,
        CaretKind::Matrix,
        CaretKind::Ice,
        CaretKind::Glitch,
        CaretKind::Neon,
        CaretKind::Heartbeat,
    ];

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.to_lowercase().as_str() {
            "block" => Self::Block,
            "beam" => Self::Beam,
            "underline" => Self::Underline,
            "fire" => Self::Fire,
            "water" => Self::Water,
            "electric" => Self::Electric,
            "comet" => Self::Comet,
            "rainbow" => Self::Rainbow,
            "matrix" => Self::Matrix,
            "ice" => Self::Ice,
            "glitch" => Self::Glitch,
            "neon" => Self::Neon,
            "heartbeat" => Self::Heartbeat,
            _ => return None,
        })
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Block => "block",
            Self::Beam => "beam",
            Self::Underline => "underline",
            Self::Fire => "fire",
            Self::Water => "water",
            Self::Electric => "electric",
            Self::Comet => "comet",
            Self::Rainbow => "rainbow",
            Self::Matrix => "matrix",
            Self::Ice => "ice",
            Self::Glitch => "glitch",
            Self::Neon => "neon",
            Self::Heartbeat => "heartbeat",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Block => "Classic solid block",
            Self::Beam => "Vertical cursor bar",
            Self::Underline => "Horizontal underscore",
            Self::Fire => "Flame & ember sparks",
            Self::Water => "Water drops & ripples",
            Self::Electric => "High-voltage electric sparks",
            Self::Comet => "Motion-blur fading trail",
            Self::Rainbow => "Smooth color spectrum",
            Self::Matrix => "Digital rain green code",
            Self::Ice => "Frost crystal shards",
            Self::Glitch => "Cyberpunk chromatic split",
            Self::Neon => "Soft luminous aura glow",
            Self::Heartbeat => "Gentle rhythmic pulse",
        }
    }
}

struct Particle {
    pos: Pos2,
    vel: Vec2,
    age: f32,
    life: f32,
    size: f32,
}

struct Ripple {
    center: Pos2,
    age: f32,
}

struct Bolt {
    pts: Vec<Pos2>,
    age: f32,
}

pub struct Caret {
    pub kind: CaretKind,
    pub glide: f32, // higher = snappier; f32::INFINITY = instant
    pub calm: bool, // no ambient effects
    pub width: f32, // caret width in pixels (1.0..=10.0, default: 3.0)
    pub animations_enabled: bool, // toggle all animations
    pos: Pos2,      // current drawn position
    started: bool,
    gliding: bool,
    last_type: f64,
    hue: f32,
    particles: Vec<Particle>,
    ripples: Vec<Ripple>,
    bolts: Vec<Bolt>,
    trail: VecDeque<Pos2>,
}

impl Caret {
    pub fn new(kind: CaretKind) -> Self {
        Self {
            kind,
            glide: 50.0,
            calm: false,
            width: 3.0,
            animations_enabled: true,
            pos: pos2(0.0, 0.0),
            started: false,
            gliding: false,
            last_type: -10.0,
            hue: 0.0,
            particles: Vec::with_capacity(700),
            ripples: Vec::new(),
            bolts: Vec::new(),
            trail: VecDeque::with_capacity(16),
        }
    }

    pub fn is_animating(&self, now: f64) -> bool {
        if !self.animations_enabled {
            return false;
        }
        self.gliding
            || !self.particles.is_empty()
            || !self.ripples.is_empty()
            || !self.bolts.is_empty()
            || (now - self.last_type) < 2.0
            || matches!(
                self.kind,
                CaretKind::Fire
                    | CaretKind::Rainbow
                    | CaretKind::Matrix
                    | CaretKind::Ice
                    | CaretKind::Neon
                    | CaretKind::Heartbeat
            )
    }

    pub fn update(&mut self, dt: f32, target: Pos2, typed: bool, now: f64, cw: f32, lh: f32) {
        if !self.started {
            self.pos = target;
            self.started = true;
        }

        if !self.animations_enabled {
            self.pos = target;
            self.gliding = false;
            self.particles.clear();
            self.ripples.clear();
            self.bolts.clear();
            self.trail.clear();
            if typed {
                self.last_type = now;
            }
            return;
        }

        // 1. Adaptive smooth glide: critically responsive horizontal typing + silky multi-line flight
        if self.glide.is_finite() && self.glide > 0.0 {
            let diff = target - self.pos;
            if diff.length() > 0.0 {
                let speed = if (diff.y).abs() < 2.0 {
                    // Snappy, instantaneous response on the same line without perceived lag
                    52.0
                } else {
                    // Smooth, elegant flight across lines and paragraphs
                    34.0
                };
                let k = 1.0 - (-dt * speed).exp();
                self.pos += diff * k;
                if (target - self.pos).length() < 0.15 {
                    self.pos = target;
                }
            }
        } else {
            self.pos = target;
        }
        self.gliding = self.pos != target;

        if typed {
            self.last_type = now;
        }
        self.hue = (self.hue + dt * 0.4) % 1.0;

        // 2. Spawn effects
        let kind = self.kind;
        match kind {
            CaretKind::Fire => {
                let n = if typed {
                    6
                } else if self.calm {
                    0
                } else {
                    1
                };
                self.emit_fire(n, cw);
            }
            CaretKind::Water if typed => {
                self.ripples.push(Ripple {
                    center: pos2(self.pos.x + cw * 0.5, self.pos.y + lh),
                    age: 0.0,
                });
                self.emit_drops(5, cw);
            }
            CaretKind::Electric if typed => {
                if self.bolts.len() < 5 {
                    let b = self.make_bolt(cw);
                    self.bolts.push(b);
                }
            }
            CaretKind::Comet => {
                self.trail.push_front(self.pos);
                if self.trail.len() > 14 {
                    self.trail.pop_back();
                }
            }
            CaretKind::Matrix => {
                let n = if typed {
                    4
                } else if self.calm {
                    0
                } else {
                    1
                };
                self.emit_matrix(n, cw);
            }
            CaretKind::Ice => {
                let n = if typed {
                    5
                } else if self.calm {
                    0
                } else {
                    1
                };
                self.emit_ice(n, cw);
            }
            _ => {}
        }

        // 3. Advance particle physics with dt
        for p in &mut self.particles {
            p.age += dt;
            p.pos += p.vel * dt;
            if kind == CaretKind::Water {
                p.vel.y += 900.0 * dt; // gravity
            }
        }
        self.particles.retain(|p| p.age < p.life);
        if self.particles.len() > 600 {
            let extra = self.particles.len() - 600;
            self.particles.drain(0..extra);
        }

        for r in &mut self.ripples {
            r.age += dt;
        }
        self.ripples.retain(|r| r.age < 0.6);

        for b in &mut self.bolts {
            b.age += dt;
        }
        self.bolts.retain(|b| b.age < 0.15);
    }

    fn blink_visible(&self, now: f64) -> bool {
        let idle = now - self.last_type;
        // Solid while typing, starts blinking after 0.5s idle
        idle < 0.5 || (((idle - 0.5) / 0.53) as u64) % 2 == 0
    }

    fn blink_alpha(&self, now: f64) -> f32 {
        let idle = (now - self.last_type) as f32;
        if idle < 0.45 {
            1.0
        } else {
            let phase = ((idle - 0.45) * std::f32::consts::TAU * 0.95).cos();
            0.5 + 0.5 * phase
        }
    }

    pub fn paint(&self, p: &egui::Painter, cw: f32, lh: f32, now: f64, accent: Color32) {
        let at = |w: f32, h: f32, y: f32| Rect::from_min_size(pos2(self.pos.x, self.pos.y + y), vec2(w, h));
        let w = self.width.clamp(1.0, 10.0);

        // Static mode: all animations disabled
        if !self.animations_enabled {
            if !self.blink_visible(now) {
                return;
            }
            match self.kind {
                CaretKind::Block => {
                    p.rect_filled(at(cw, lh, 0.0), 0.0, accent);
                }
                CaretKind::Underline => {
                    p.rect_filled(at(cw, w, lh - w), 0.0, accent);
                }
                CaretKind::Fire => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(255, 140, 20));
                }
                CaretKind::Water => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(70, 170, 255));
                }
                CaretKind::Electric => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(170, 210, 255));
                }
                CaretKind::Matrix => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(40, 255, 90));
                }
                CaretKind::Ice => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(130, 230, 255));
                }
                CaretKind::Heartbeat => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(255, 60, 100));
                }
                _ => {
                    p.rect_filled(at(w, lh, 0.0), 0.0, accent);
                }
            }
            return;
        }

        let alpha = self.blink_alpha(now);
        if alpha < 0.03 {
            return;
        }

        let with_alpha = |c: Color32| -> Color32 {
            Color32::from_rgba_unmultiplied(
                c.r(),
                c.g(),
                c.b(),
                ((c.a() as f32) * alpha) as u8,
            )
        };

        // Animated rendering mode
        match self.kind {
            CaretKind::Block => {
                p.rect_filled(at(cw, lh, 0.0), 0.0, with_alpha(accent));
            }
            CaretKind::Beam => {
                p.rect_filled(at(w, lh, 0.0), 0.0, with_alpha(accent));
            }
            CaretKind::Underline => {
                p.rect_filled(at(cw, w, lh - w), 0.0, with_alpha(accent));
            }
            CaretKind::Rainbow => {
                let c = Color32::from(egui::ecolor::Hsva::new(self.hue, 0.9, 1.0, 1.0));
                p.rect_filled(at(w, lh, 0.0), 0.0, c);
            }
            CaretKind::Fire => {
                // Tightly tuned flickering flame base
                let flick = 0.85 + 0.15 * ((now * 25.0).sin() as f32);
                p.rect_filled(
                    at(w, lh, 0.0),
                    1.0,
                    Color32::from_rgb((255.0 * flick) as u8, (135.0 * flick) as u8, 20),
                );
                // Glowing sparks (tighter and scaled down)
                for s in &self.particles {
                    let t = s.age / s.life;
                    p.circle_filled(s.pos, s.size * (1.0 - 0.6 * t), fire_color(t));
                }
            }
            CaretKind::Water => {
                p.rect_filled(at(w, lh, 0.0), 1.0, Color32::from_rgb(70, 170, 255));
                for r in &self.ripples {
                    let t = r.age / 0.6;
                    let a = ((1.0 - t) * 255.0) as u8;
                    p.circle_stroke(
                        r.center,
                        3.0 + t * 60.0,
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(90, 200, 255, a)),
                    );
                }
                for d in &self.particles {
                    let t = d.age / d.life;
                    let a = ((1.0 - t) * 255.0) as u8;
                    p.circle_filled(
                        d.pos,
                        d.size,
                        Color32::from_rgba_unmultiplied(120, 210, 255, a),
                    );
                }
            }
            CaretKind::Electric => {
                p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(170, 210, 255));
                for b in &self.bolts {
                    let a = ((1.0 - b.age / 0.15) * 255.0) as u8;
                    p.add(egui::Shape::line(
                        b.pts.clone(),
                        Stroke::new(1.5, Color32::from_rgba_unmultiplied(210, 235, 255, a)),
                    ));
                }
            }
            CaretKind::Comet => {
                let n = self.trail.len().max(1) as f32;
                for (i, tp) in self.trail.iter().enumerate() {
                    let t = i as f32 / n;
                    let a = ((1.0 - t) * 150.0) as u8;
                    p.rect_filled(
                        Rect::from_min_size(*tp, vec2(w, lh)),
                        0.0,
                        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), a),
                    );
                }
                p.rect_filled(at(w, lh, 0.0), 0.0, accent);
            }
            CaretKind::Matrix => {
                // Green phosphor digital rain
                p.rect_filled(at(w, lh, 0.0), 0.0, Color32::from_rgb(40, 255, 90));
                for p_item in &self.particles {
                    let t = p_item.age / p_item.life;
                    let a = ((1.0 - t) * 220.0) as u8;
                    p.rect_filled(
                        Rect::from_min_size(p_item.pos, vec2(w * 0.7, p_item.size * 2.0)),
                        0.0,
                        Color32::from_rgba_unmultiplied(60, 255, 120, a),
                    );
                }
            }
            CaretKind::Ice => {
                // Cyan frost crystals
                p.rect_filled(at(w, lh, 0.0), 1.0, Color32::from_rgb(130, 230, 255));
                for crystal in &self.particles {
                    let t = crystal.age / crystal.life;
                    let a = ((1.0 - t) * 200.0) as u8;
                    p.circle_filled(
                        crystal.pos,
                        crystal.size,
                        Color32::from_rgba_unmultiplied(180, 245, 255, a),
                    );
                }
            }
            CaretKind::Glitch => {
                let is_glitching = (now - self.last_type) < 0.1;
                if is_glitching {
                    // Chromatic aberration RGB split
                    let shift = ((now * 60.0).sin() * 3.0) as f32;
                    let r_rect = Rect::from_min_size(
                        pos2(self.pos.x - shift, self.pos.y),
                        vec2(w, lh),
                    );
                    let b_rect = Rect::from_min_size(
                        pos2(self.pos.x + shift, self.pos.y),
                        vec2(w, lh),
                    );
                    p.rect_filled(r_rect, 0.0, Color32::from_rgba_unmultiplied(255, 30, 60, 160));
                    p.rect_filled(b_rect, 0.0, Color32::from_rgba_unmultiplied(30, 120, 255, 160));
                }
                p.rect_filled(at(w, lh, 0.0), 0.0, Color32::WHITE);
            }
            CaretKind::Neon => {
                // 3 stacked glowing layers
                for i in (1..=3).rev() {
                    let pad = i as f32 * 2.5;
                    let alpha = (45 - i * 10) as u8;
                    let outer = Rect::from_min_size(
                        pos2(self.pos.x - pad, self.pos.y - pad),
                        vec2(w + pad * 2.0, lh + pad * 2.0),
                    );
                    p.rect_filled(
                        outer,
                        2.0,
                        Color32::from_rgba_unmultiplied(accent.r(), accent.g(), accent.b(), alpha),
                    );
                }
                p.rect_filled(at(w, lh, 0.0), 1.0, accent);
            }
            CaretKind::Heartbeat => {
                // Smooth rhythmic pulse
                let pulse = ((now * 5.0).sin() as f32).max(0.0);
                let extra_w = pulse * 2.0;
                let extra_h = pulse * 2.0;
                let heart_rect = Rect::from_min_size(
                    pos2(self.pos.x - extra_w * 0.5, self.pos.y - extra_h * 0.5),
                    vec2(w + extra_w, lh + extra_h),
                );
                p.rect_filled(heart_rect, 1.0, Color32::from_rgb(255, 60, 100));
            }
        }
    }

    fn emit_fire(&mut self, n: usize, cw: f32) {
        for _ in 0..n {
            self.particles.push(Particle {
                pos: pos2(self.pos.x + fastrand::f32() * cw, self.pos.y + 2.0),
                vel: vec2(
                    (fastrand::f32() - 0.5) * 16.0,
                    -(35.0 + fastrand::f32() * 45.0),
                ),
                age: 0.0,
                life: 0.22 + fastrand::f32() * 0.22,
                size: 1.0 + fastrand::f32() * 1.6, // Tightly sized sparks
            });
        }
    }

    fn emit_drops(&mut self, n: usize, cw: f32) {
        for _ in 0..n {
            self.particles.push(Particle {
                pos: pos2(self.pos.x + fastrand::f32() * cw, self.pos.y),
                vel: vec2(
                    (fastrand::f32() - 0.5) * 90.0,
                    -(90.0 + fastrand::f32() * 120.0),
                ),
                age: 0.0,
                life: 0.45 + fastrand::f32() * 0.25,
                size: 1.3 + fastrand::f32() * 1.5,
            });
        }
    }

    fn emit_matrix(&mut self, n: usize, cw: f32) {
        for _ in 0..n {
            self.particles.push(Particle {
                pos: pos2(self.pos.x + fastrand::f32() * cw, self.pos.y + 4.0),
                vel: vec2(0.0, 50.0 + fastrand::f32() * 70.0),
                age: 0.0,
                life: 0.35 + fastrand::f32() * 0.35,
                size: 1.4 + fastrand::f32() * 1.4,
            });
        }
    }

    fn emit_ice(&mut self, n: usize, cw: f32) {
        for _ in 0..n {
            self.particles.push(Particle {
                pos: pos2(
                    self.pos.x + (fastrand::f32() - 0.2) * cw * 1.1,
                    self.pos.y + 2.0,
                ),
                vel: vec2(
                    (fastrand::f32() - 0.5) * 18.0,
                    25.0 + fastrand::f32() * 35.0,
                ),
                age: 0.0,
                life: 0.35 + fastrand::f32() * 0.25,
                size: 1.1 + fastrand::f32() * 1.7,
            });
        }
    }

    fn make_bolt(&self, cw: f32) -> Bolt {
        let start = pos2(self.pos.x + cw * 0.5, self.pos.y);
        let ang = fastrand::f32() * std::f32::consts::TAU;
        let end = start + vec2(ang.cos(), ang.sin()) * (24.0 + fastrand::f32() * 36.0);
        let mut pts = vec![start];
        for i in 1..5 {
            let base = start + (end - start) * (i as f32 / 5.0);
            pts.push(
                base + vec2(
                    (fastrand::f32() - 0.5) * 10.0,
                    (fastrand::f32() - 0.5) * 10.0,
                ),
            );
        }
        pts.push(end);
        Bolt { pts, age: 0.0 }
    }
}

/// Additive glowing fire: white-yellow -> orange -> red -> transparent
fn fire_color(t: f32) -> Color32 {
    let t = t.clamp(0.0, 1.0);
    let c = if t < 0.3 {
        mix([255., 240., 170.], [255., 150., 30.], t / 0.3)
    } else {
        mix([255., 150., 30.], [190., 25., 0.], (t - 0.3) / 0.7)
    };
    let fade = (1.0 - t).powf(1.4);
    Color32::from_rgba_premultiplied(
        (c[0] * fade) as u8,
        (c[1] * fade) as u8,
        (c[2] * fade) as u8,
        0,
    )
}

fn mix(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [
        a[0] + (b[0] - a[0]) * t,
        a[1] + (b[1] - a[1]) * t,
        a[2] + (b[2] - a[2]) * t,
    ]
}
