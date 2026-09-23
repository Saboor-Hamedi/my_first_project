//! Appearance & theme settings tab.
//!
//! v2: the grid now scrolls. At 11 themes, 6 rows mostly fit in a normal
//! panel; at 17 themes (9 rows) that stopped being true, and since the old
//! code drew every card unconditionally with no clipping or scroll offset,
//! cards rendered below the visible panel were still clickable — an actual
//! bug, not just a cosmetic one. Also added a visible contrast badge per
//! card (AA/AAA) using the `contrast_ratio` check from theme.rs, so the
//! readability guarantee is something the user can actually see.

use crate::theme::{contrast_ratio, Theme, ThemeKind};
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Id, Pos2, Rect, Stroke};

pub fn render_theme_tab(
    ui: &egui::Ui,
    painter: &egui::Painter,
    panel_rect: Rect,
    p_origin: Pos2,
    theme: &mut Theme,
    on_save_setting: &mut dyn FnMut(&str, &str),
) {
    painter.text(
        p_origin,
        Align2::LEFT_TOP,
        "APPEARANCE & THEME",
        FontId::monospace(14.5),
        theme.highlight,
    );
    painter.text(
        p_origin + vec2(0.0, 22.0),
        Align2::LEFT_TOP,
        "Curated developer palettes that transform the entire interface",
        FontId::monospace(11.5),
        theme.muted,
    );

    let pad_x = 20.0;
    let grid_left = panel_rect.min.x + pad_x;
    let grid_right = panel_rect.max.x - pad_x;
    let grid_w = grid_right - grid_left;

    let col_gap = 12.0;
    let row_gap = 10.0;
    let card_w = (grid_w - col_gap) * 0.5;
    let card_h = 64.0;
    let grid_top = p_origin.y + 56.0;

    let rows = ThemeKind::ALL.len().div_ceil(2);
    let content_h = rows as f32 * (card_h + row_gap) - row_gap;
    let visible_h = (panel_rect.max.y - grid_top).max(0.0);
    let max_scroll = (content_h - visible_h).max(0.0);

    // Persistent scroll offset for this tab, stored in egui's own temp
    // storage — no new fields needed on any struct.
    let scroll_id = Id::new("theme_tab_scroll");
    let mut scroll = ui.ctx().data_mut(|d| d.get_temp::<f32>(scroll_id)).unwrap_or(0.0);

    let pointer_over_grid = ui.rect_contains_pointer(Rect::from_min_max(
        pos2(panel_rect.min.x, grid_top),
        panel_rect.max,
    ));
    if pointer_over_grid && max_scroll > 0.0 {
        let wheel = ui.input(|i| i.smooth_scroll_delta.y);
        // If this feels inverted on your setup, flip the sign here.
        scroll = (scroll - wheel).clamp(0.0, max_scroll);
    }
    ui.ctx().data_mut(|d| d.insert_temp(scroll_id, scroll));

    // Everything below grid_top is clipped to the panel, so a card that's
    // half-scrolled-off doesn't paint over the header text above it.
    let clip_rect = Rect::from_min_max(pos2(panel_rect.min.x, grid_top), panel_rect.max);
    let grid_painter = painter.with_clip_rect(clip_rect);

    let mut selected: Option<ThemeKind> = None;

    for (i, &t_kind) in ThemeKind::ALL.iter().enumerate() {
        let col = i % 2;
        let row = i / 2;
        let t_preset = Theme::from_kind(t_kind);

        let unscrolled = Rect::from_min_size(
            pos2(
                grid_left + col as f32 * (card_w + col_gap),
                grid_top + row as f32 * (card_h + row_gap),
            ),
            vec2(card_w, card_h),
        );
        let t_rect = unscrolled.translate(vec2(0.0, -scroll));

        // Cull: skip both drawing and input for cards outside the visible
        // clip area. This is what actually fixes the offscreen-click bug —
        // clipping alone only stops drawing, not interaction.
        if !clip_rect.intersects(t_rect) {
            continue;
        }

        let is_sel = theme.kind == t_kind;
        let hovered = ui.rect_contains_pointer(t_rect);
        let hover_t = ui.ctx().animate_bool(Id::new(("theme_card_hover", t_kind)), hovered);

        let bg = if is_sel {
            Color32::from_rgba_unmultiplied(t_preset.accent.r(), t_preset.accent.g(), t_preset.accent.b(), 30)
        } else {
            let base = Color32::from_rgba_unmultiplied(theme.bg.r(), theme.bg.g(), theme.bg.b(), 130);
            let hover_bg = theme.surface();
            lerp_color(base, hover_bg, hover_t)
        };

        let border = if is_sel {
            Stroke::new(1.5, t_preset.accent)
        } else {
            Stroke::new(1.0 + 0.3 * hover_t, lerp_color(theme.border(), t_preset.accent, hover_t * 0.6))
        };

        grid_painter.rect(t_rect, 6.0, bg, border, egui::StrokeKind::Inside);

        if is_sel {
            let stripe = Rect::from_min_size(t_rect.min, vec2(t_rect.width(), 2.5));
            grid_painter.rect_filled(stripe, egui::CornerRadius { nw: 6, ne: 6, sw: 0, se: 0 }, t_preset.accent);
        }

        let title_y = t_rect.min.y + 13.0;
        let inner_left = t_rect.min.x + 12.0;
        let inner_right = t_rect.max.x - 12.0;

        grid_painter.text(
            pos2(inner_left, title_y),
            Align2::LEFT_CENTER,
            t_kind.display_name(),
            FontId::monospace(12.0),
            if is_sel { t_preset.accent } else { theme.text },
        );

        if is_sel {
            grid_painter.text(
                pos2(inner_right, title_y),
                Align2::RIGHT_CENTER,
                "\u{25cf} ACTIVE",
                FontId::monospace(10.0),
                t_preset.accent,
            );
        } else {
            // Readability badge: makes the contrast guarantee from theme.rs's
            // `every_theme_is_readable` test visible in the UI, not just enforced
            // silently at test time.
            let ratio = contrast_ratio(t_preset.text, t_preset.bg);
            let label = if ratio >= 7.0 { "AAA" } else { "AA" };
            grid_painter.text(
                pos2(inner_right, title_y),
                Align2::RIGHT_CENTER,
                format!("{label} \u{00b7} {ratio:.1}:1"),
                FontId::monospace(9.0),
                theme.muted,
            );
        }

        let swatches = [t_preset.bg, t_preset.text, t_preset.accent, t_preset.highlight];
        let swatch_gap = 6.0;
        let swatch_count = swatches.len() as f32;
        let swatch_available = inner_right - inner_left;
        let swatch_w = ((swatch_available - (swatch_count - 1.0) * swatch_gap) / swatch_count).floor().max(10.0);
        let swatch_h = 16.0;
        let swatch_y = t_rect.min.y + 34.0;

        for (s_idx, &color) in swatches.iter().enumerate() {
            let sw_x = inner_left + s_idx as f32 * (swatch_w + swatch_gap);
            let sw_rect = Rect::from_min_size(pos2(sw_x, swatch_y), vec2(swatch_w, swatch_h));
            grid_painter.rect_filled(sw_rect, 3.0, color);
            grid_painter.rect(
                sw_rect,
                3.0,
                Color32::TRANSPARENT,
                Stroke::new(1.0, Color32::from_rgba_unmultiplied(255, 255, 255, 30)),
                egui::StrokeKind::Inside,
            );
        }

        if hovered && ui.input(|inp| inp.pointer.primary_clicked()) {
            selected = Some(t_kind);
        }
    }

    // Thin custom scrollbar so it's obvious there's more content below —
    // visual only, no drag-to-scroll (mouse wheel covers the common case).
    if max_scroll > 0.0 {
        let track_x = panel_rect.max.x - 6.0;
        let track = Rect::from_min_max(pos2(track_x, grid_top), pos2(track_x + 3.0, panel_rect.max.y));
        painter.rect_filled(track, 1.5, Color32::from_rgba_unmultiplied(255, 255, 255, 15));

        let thumb_h = (visible_h * visible_h / content_h).max(24.0);
        let thumb_y = grid_top + (scroll / max_scroll) * (visible_h - thumb_h);
        let thumb = Rect::from_min_size(pos2(track_x, thumb_y), vec2(3.0, thumb_h));
        painter.rect_filled(thumb, 1.5, Color32::from_rgba_unmultiplied(255, 255, 255, 60));
    }

    if let Some(t_kind) = selected {
        *theme = Theme::from_kind(t_kind);
        on_save_setting("theme", t_kind.name());
    }
}

fn lerp_color(a: Color32, b: Color32, t: f32) -> Color32 {
    let l = |x: u8, y: u8| (x as f32 + (y as f32 - x as f32) * t) as u8;
    Color32::from_rgb(l(a.r(), b.r()), l(a.g(), b.g()), l(a.b(), b.b()))
}
