//! Learning statistics, writing analytics, and daily activity story view.
//! Responsive, scroll-protected, and bounded to avoid any window overflow.

use crate::theme::Theme;
use core::DailyActivity;
use eframe::egui::{self, pos2, vec2, Align2, Color32, FontId, Rect, Sense, Stroke};

fn format_duration(seconds: u32) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else {
        let hrs = seconds / 3600;
        let mins = (seconds % 3600) / 60;
        if mins > 0 {
            format!("{}h {}m", hrs, mins)
        } else {
            format!("{}h", hrs)
        }
    }
}

fn format_duration_u64(seconds: u64) -> String {
    if seconds < 60 {
        format!("{}s", seconds)
    } else if seconds < 3600 {
        format!("{}m", seconds / 60)
    } else {
        let hrs = seconds as f64 / 3600.0;
        format!("{:.1}h", hrs)
    }
}

fn format_number(n: u64) -> String {
    if n >= 1_000_000 {
        format!("{:.1}M", n as f64 / 1_000_000.0)
    } else if n >= 1_000 {
        format!("{:.1}k", n as f64 / 1_000.0)
    } else {
        n.to_string()
    }
}

pub fn render_stats(
    ui: &mut egui::Ui,
    editor_rect: Rect,
    today_activity: &DailyActivity,
    history: &[DailyActivity],
    lifetime: (u64, u64, u64, usize), // (total_seconds, total_keystrokes, total_words, active_days)
    total_notes: usize,
    theme: &Theme,
    today_date: &str,
    yesterday_date: &str,
) {
    // Wrap entire stats view in a bounded vertical ScrollArea so content never offsets out of view
    ui.allocate_new_ui(egui::UiBuilder::new().max_rect(editor_rect), |ui| {
        egui::ScrollArea::vertical()
            .id_salt("stats_scroll_view")
            .auto_shrink([false; 2])
            .show(ui, |ui| {
                let avail_w = ui.available_width().max(360.0);

                // ── 1. Header Section ─────────────────────────────────────────
                let (header_rect, _) = ui.allocate_exact_size(vec2(avail_w, 52.0), Sense::hover());
                let p = ui.painter();

                p.text(
                    header_rect.min,
                    Align2::LEFT_TOP,
                    "WRITING STORY & ACTIVITY",
                    FontId::monospace(19.0),
                    theme.highlight,
                );
                p.text(
                    header_rect.min + vec2(0.0, 26.0),
                    Align2::LEFT_TOP,
                    "A living chronicle of your thoughts, focus, and time in MindForge",
                    FontId::monospace(11.5),
                    theme.muted,
                );

                ui.add_space(8.0);

                // ── 2. Responsive Hero Summary Cards ─────────────────────────
                let cards_data = [
                    (
                        "⏱ TIME IN EDITOR",
                        format_duration(today_activity.active_seconds),
                        format!("{} all-time", format_duration_u64(lifetime.0)),
                    ),
                    (
                        "✍ WORDS WRITTEN",
                        format_number(today_activity.words_written as u64),
                        format!("{} total words", format_number(lifetime.2)),
                    ),
                    (
                        "⌨ KEYSTROKES",
                        format_number(today_activity.keystrokes as u64),
                        format!("{} total keys", format_number(lifetime.1)),
                    ),
                    (
                        "📚 NOTES & STREAK",
                        format!("{} notes", total_notes),
                        format!("{} active days", lifetime.3),
                    ),
                ];

                let gap = 12.0;
                let is_4_cols = avail_w >= 640.0;
                let card_h = 72.0;

                let section_h = if is_4_cols {
                    card_h
                } else {
                    card_h * 2.0 + gap
                };

                let (cards_rect, _) = ui.allocate_exact_size(vec2(avail_w, section_h), Sense::hover());
                let p = ui.painter();

                if is_4_cols {
                    let card_w = ((avail_w - 3.0 * gap) / 4.0).max(100.0);
                    for (i, (label, val, sub)) in cards_data.iter().enumerate() {
                        let c_rect = Rect::from_min_size(
                            pos2(cards_rect.min.x + i as f32 * (card_w + gap), cards_rect.min.y),
                            vec2(card_w, card_h),
                        );
                        draw_metric_card(p, c_rect, label, val, sub, theme);
                    }
                } else {
                    let card_w = ((avail_w - gap) / 2.0).max(100.0);
                    for (i, (label, val, sub)) in cards_data.iter().enumerate() {
                        let col = i % 2;
                        let row = i / 2;
                        let c_rect = Rect::from_min_size(
                            pos2(
                                cards_rect.min.x + col as f32 * (card_w + gap),
                                cards_rect.min.y + row as f32 * (card_h + gap),
                            ),
                            vec2(card_w, card_h),
                        );
                        draw_metric_card(p, c_rect, label, val, sub, theme);
                    }
                }

                ui.add_space(20.0);

                // ── 3. Writing Rhythm (14-Day Activity Bar Chart) ────────────
                let chart_box_h = 136.0;
                let (chart_container, _) = ui.allocate_exact_size(vec2(avail_w, chart_box_h), Sense::hover());
                let p = ui.painter();

                // Chart outer container card
                p.rect(
                    chart_container,
                    5.0,
                    Color32::from_rgb(14, 15, 18),
                    Stroke::new(1.0, Color32::from_rgb(26, 28, 34)),
                    egui::StrokeKind::Inside,
                );

                p.text(
                    pos2(chart_container.min.x + 14.0, chart_container.min.y + 12.0),
                    Align2::LEFT_TOP,
                    "WRITING RHYTHM (RECENT DAYS)",
                    FontId::monospace(11.5),
                    theme.highlight,
                );

                // Take up to 14 days
                let display_days: Vec<&DailyActivity> = history.iter().take(14).collect();
                let num_bars = display_days.len().max(1);
                let bar_area_w = chart_container.width() - 28.0;
                let bar_gap = 8.0;
                let bar_w = ((bar_area_w - (num_bars as f32 - 1.0) * bar_gap) / num_bars as f32)
                    .clamp(14.0, 36.0);

                let max_chart_h = 60.0;
                let chart_base_y = chart_container.min.y + 104.0;

                let max_secs = display_days
                    .iter()
                    .map(|a| a.active_seconds)
                    .max()
                    .unwrap_or(60)
                    .max(60) as f32;

                for (i, act) in display_days.iter().enumerate() {
                    let bx = chart_container.min.x + 14.0 + i as f32 * (bar_w + bar_gap);
                    let h = ((act.active_seconds as f32 / max_secs) * max_chart_h).max(3.0);
                    let bar_rect = Rect::from_min_max(pos2(bx, chart_base_y - h), pos2(bx + bar_w, chart_base_y));

                    let is_today = act.date == today_date;
                    let is_active = act.active_seconds > 0;

                    let fill = if is_today {
                        theme.highlight
                    } else if is_active {
                        theme.accent
                    } else {
                        Color32::from_rgb(24, 25, 30)
                    };

                    p.rect_filled(bar_rect, 2.5, fill);

                    // Minutes badge on top of bar
                    if act.active_seconds >= 60 {
                        p.text(
                            pos2(bx + bar_w * 0.5, chart_base_y - h - 12.0),
                            Align2::CENTER_TOP,
                            format!("{}m", act.active_seconds / 60),
                            FontId::monospace(9.0),
                            if is_today { theme.highlight } else { Color32::from_gray(160) },
                        );
                    }

                    // Date label beneath bar
                    let date_label = if is_today {
                        "Today"
                    } else if act.date == yesterday_date {
                        "Yest"
                    } else {
                        act.date.split('-').last().unwrap_or(&act.date)
                    };

                    p.text(
                        pos2(bx + bar_w * 0.5, chart_base_y + 4.0),
                        Align2::CENTER_TOP,
                        date_label,
                        FontId::monospace(9.0),
                        if is_today { theme.accent } else { Color32::from_gray(120) },
                    );
                }

                ui.add_space(20.0);

                // ── 4. Chronological Activity Journal Cards ──────────────────
                let (story_hdr_rect, _) = ui.allocate_exact_size(vec2(avail_w, 24.0), Sense::hover());
                ui.painter().text(
                    story_hdr_rect.min,
                    Align2::LEFT_TOP,
                    "ACTIVITY JOURNAL & STORY",
                    FontId::monospace(12.5),
                    theme.highlight,
                );

                ui.add_space(6.0);

                if history.is_empty() {
                    let (empty_rect, _) = ui.allocate_exact_size(vec2(avail_w, 36.0), Sense::hover());
                    ui.painter().text(
                        empty_rect.min + vec2(10.0, 8.0),
                        Align2::LEFT_TOP,
                        "Start typing your notes to begin your activity story!",
                        FontId::monospace(12.0),
                        theme.muted,
                    );
                } else {
                    for act in history {
                        let (row_rect, _) = ui.allocate_exact_size(vec2(avail_w, 34.0), Sense::hover());
                        let p = ui.painter();
                        let is_today = act.date == today_date;

                        p.rect(
                            row_rect,
                            4.0,
                            if is_today { Color32::from_rgb(18, 26, 20) } else { Color32::from_rgb(14, 15, 18) },
                            Stroke::new(1.0, if is_today { theme.accent } else { Color32::from_rgb(26, 28, 34) }),
                            egui::StrokeKind::Inside,
                        );

                        // Date tag
                        let display_date = if is_today {
                            format!("★ TODAY ({})", act.date)
                        } else if act.date == yesterday_date {
                            format!("• YESTERDAY ({})", act.date)
                        } else {
                            format!("• {}", act.date)
                        };

                        p.text(
                            pos2(row_rect.min.x + 12.0, row_rect.center().y),
                            Align2::LEFT_CENTER,
                            display_date,
                            FontId::monospace(11.0),
                            if is_today { theme.accent } else { Color32::from_gray(180) },
                        );

                        // Flexible positioning depending on available width
                        if avail_w >= 600.0 {
                            // Full layout with 4 metrics
                            let col2 = row_rect.min.x + avail_w * 0.28;
                            let col3 = row_rect.min.x + avail_w * 0.44;
                            let col4 = row_rect.min.x + avail_w * 0.62;

                            p.text(
                                pos2(col2, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("⏱ {}", format_duration(act.active_seconds)),
                                FontId::monospace(11.0),
                                Color32::from_gray(190),
                            );

                            p.text(
                                pos2(col3, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("✍ {} words", format_number(act.words_written as u64)),
                                FontId::monospace(11.0),
                                theme.highlight,
                            );

                            p.text(
                                pos2(col4, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("⌨ {} keys", format_number(act.keystrokes as u64)),
                                FontId::monospace(11.0),
                                Color32::from_gray(160),
                            );

                            p.text(
                                pos2(row_rect.max.x - 14.0, row_rect.center().y),
                                Align2::RIGHT_CENTER,
                                format!("{} created · {} saved", act.notes_created, act.notes_edited),
                                FontId::monospace(10.5),
                                theme.muted,
                            );
                        } else {
                            // Compact layout for narrow windows
                            let col2 = row_rect.min.x + avail_w * 0.36;
                            let col3 = row_rect.min.x + avail_w * 0.62;

                            p.text(
                                pos2(col2, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("⏱ {}", format_duration(act.active_seconds)),
                                FontId::monospace(10.5),
                                Color32::from_gray(190),
                            );

                            p.text(
                                pos2(col3, row_rect.center().y),
                                Align2::LEFT_CENTER,
                                format!("✍ {}", format_number(act.words_written as u64)),
                                FontId::monospace(10.5),
                                theme.highlight,
                            );

                            p.text(
                                pos2(row_rect.max.x - 10.0, row_rect.center().y),
                                Align2::RIGHT_CENTER,
                                format!("{} saved", act.notes_edited),
                                FontId::monospace(10.0),
                                theme.muted,
                            );
                        }

                        ui.add_space(4.0);
                    }
                }

                ui.add_space(24.0);
            });
    });
}

fn draw_metric_card(
    p: &eframe::egui::Painter,
    rect: Rect,
    label: &str,
    val: &str,
    sub: &str,
    theme: &Theme,
) {
    p.rect(
        rect,
        5.0,
        Color32::from_rgb(15, 16, 20),
        Stroke::new(1.0, Color32::from_rgb(30, 32, 40)),
        egui::StrokeKind::Inside,
    );

    // Accent line on left edge
    let stripe = Rect::from_min_size(rect.min, vec2(3.0, rect.height()));
    p.rect_filled(stripe, egui::CornerRadius { nw: 5, sw: 5, ne: 0, se: 0 }, theme.accent);

    // Header label
    p.text(
        pos2(rect.min.x + 12.0, rect.min.y + 10.0),
        Align2::LEFT_TOP,
        label,
        FontId::monospace(9.5),
        theme.muted,
    );

    // Main value
    p.text(
        pos2(rect.min.x + 12.0, rect.min.y + 26.0),
        Align2::LEFT_TOP,
        val,
        FontId::monospace(17.0),
        theme.highlight,
    );

    // Subtitle
    p.text(
        pos2(rect.min.x + 12.0, rect.min.y + 50.0),
        Align2::LEFT_TOP,
        sub,
        FontId::monospace(10.0),
        Color32::from_gray(140),
    );
}
