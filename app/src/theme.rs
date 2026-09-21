use eframe::egui::Color32;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThemeKind {
    Green,
    Amber,
    White,
    Ice,
}

impl ThemeKind {
    pub fn parse(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "green" => Some(Self::Green),
            "amber" => Some(Self::Amber),
            "white" => Some(Self::White),
            "ice" => Some(Self::Ice),
            _ => None,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::Green => "green",
            Self::Amber => "amber",
            Self::White => "white",
            Self::Ice => "ice",
        }
    }
}

#[derive(Clone, Copy)]
#[allow(dead_code)]
pub struct Theme {
    pub kind: ThemeKind,
    pub text: Color32,
    pub accent: Color32,
    pub muted: Color32,
    pub highlight: Color32,
}

impl Theme {
    pub fn from_kind(kind: ThemeKind) -> Self {
        match kind {
            ThemeKind::Green => Self {
                kind,
                text: Color32::from_rgb(200, 255, 210),
                accent: Color32::from_rgb(51, 255, 102),
                muted: Color32::from_rgb(90, 140, 100),
                highlight: Color32::from_rgb(100, 255, 150),
            },
            ThemeKind::Amber => Self {
                kind,
                text: Color32::from_rgb(255, 230, 180),
                accent: Color32::from_rgb(255, 175, 45),
                muted: Color32::from_rgb(150, 110, 60),
                highlight: Color32::from_rgb(255, 210, 100),
            },
            ThemeKind::White => Self {
                kind,
                text: Color32::from_rgb(235, 235, 235),
                accent: Color32::from_rgb(255, 255, 255),
                muted: Color32::from_rgb(130, 130, 130),
                highlight: Color32::from_rgb(200, 220, 255),
            },
            ThemeKind::Ice => Self {
                kind,
                text: Color32::from_rgb(210, 240, 255),
                accent: Color32::from_rgb(80, 210, 255),
                muted: Color32::from_rgb(70, 130, 160),
                highlight: Color32::from_rgb(140, 230, 255),
            },
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self::from_kind(ThemeKind::Green)
    }
}
