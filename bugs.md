warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/app/modals.rs:34:29
   |
34 |                 Stroke::new(1.0, self.theme.border()),
   |                             ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>
   = note: `#[warn(float_literal_f32_fallback)]` (part of `#[warn(future_incompatible)]`) on by default

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/app/panes.rs:400:41
    |
400 | ...                   Stroke::new(1.0, grip_color),
    |                                   ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/app/shell.rs:26:25
   |
26 |             Stroke::new(1.0, Color32::from_rgb(32, 34, 40)),
   |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/app/shell.rs:188:33
    |
188 |                     Stroke::new(1.0, grip_color),
    |                                 ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/app/terminal_drawer.rs:76:33
   |
76 |                     Stroke::new(1.0, grip_color),
   |                                 ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:158:21
    |
158 |         Stroke::new(1.0, theme.border().gamma_multiply(open_t)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:168:21
    |
168 |         Stroke::new(1.0, theme.border().gamma_multiply(open_t)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:232:21
    |
232 |         Stroke::new(1.0, theme.border().gamma_multiply(open_t)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:366:21
    |
366 |         Stroke::new(1.0, lerp_color(theme.border(), theme.accent, reset_t)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:415:21
    |
415 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:446:21
    |
446 |         Stroke::new(1.0, lerp_color(theme.border(), theme.accent, badge_t)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:503:25
    |
503 |             Stroke::new(1.8, theme.accent)
    |                         ^^^ help: explicitly specify the type as `f32`: `1.8_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:507:25
    |
507 |             Stroke::new(0.8, theme.border())
    |                         ^^^ help: explicitly specify the type as `f32`: `0.8_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/accent/accentcolor.rs:582:21
    |
582 |         Stroke::new(1.0, border),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/statusbar.rs:275:25
    |
275 |             Stroke::new(1.4, knob_color),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.4_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/caret/candle.rs:27:21
   |
27 |         Stroke::new(1.0, Color32::from_rgb(50, 40, 30)),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/caret/effects.rs:36:25
   |
36 |             Stroke::new(1.5, Color32::from_rgba_unmultiplied(210, 235, 255, a)),
   |                         ^^^ help: explicitly specify the type as `f32`: `1.5_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/caret/neon.rs:20:43
   |
20 |     p.rect_stroke(inner, 1.0, Stroke::new(1.0, accent), StrokeKind::Inside);
   |                                           ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/caret/water.rs:38:25
   |
38 |             Stroke::new(1.0, Color32::from_rgba_unmultiplied(80, 200, 255, a)),
   |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/command/command_suggestion.rs:309:21
    |
309 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/command/command_suggestion.rs:334:21
    |
334 |         Stroke::new(1.0, theme.border().linear_multiply(0.6)),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/modals.rs:88:36
   |
88 |     let glass_border = Stroke::new(1.0, theme.border());
   |                                    ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:278:25
    |
278 |             Stroke::new(1.0, theme.border()),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:463:25
    |
463 |             Stroke::new(1.0, theme.border()),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:518:21
    |
518 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:548:21
    |
548 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:632:21
    |
632 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:748:21
    |
748 |         Stroke::new(1.0, cancel_stroke),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/modals.rs:778:21
    |
778 |         Stroke::new(1.0, del_stroke),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/ai_engine.rs:50:21
   |
50 |         Stroke::new(1.0, theme.border()),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/ai_engine.rs:79:21
   |
79 |         Stroke::new(1.0, if show_hover { theme.accent } else { theme.border() }),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/ai_engine.rs:110:21
    |
110 |         Stroke::new(1.0, if paste_hover { theme.accent } else { theme.border() }),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/ai_engine.rs:138:25
    |
138 |             Stroke::new(1.0, if clear_hover { Color32::from_rgb(220, 60, 60) } else { theme.border() }),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/ai_engine.rs:244:25
    |
244 | ...   Stroke::new(1.0, if is_sel { theme.accent } else if opt_hover { theme.border().lerp_to_gamma(theme.accent, 0.4) } else { them...
    |                   ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/ai_engine.rs:277:21
    |
277 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/backup.rs:44:21
   |
44 |         Stroke::new(1.0, theme.border()),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/backup.rs:73:21
   |
73 |         Stroke::new(1.0, theme.border()),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/backup.rs:99:21
   |
99 |         Stroke::new(1.0, if browse_hover { theme.accent } else { theme.border() }),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/backup.rs:129:21
    |
129 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/backup.rs:171:21
    |
171 |         Stroke::new(1.0, if run_hover { theme.accent } else { theme.border() }),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/backup.rs:194:21
    |
194 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/carets.rs:82:25
   |
82 |             Stroke::new(1.0, theme.accent)
   |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/carets.rs:84:25
   |
84 |             Stroke::new(1.0, theme.border().lerp_to_gamma(theme.accent, 0.4))
   |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/carets.rs:86:25
   |
86 |             Stroke::new(1.0, theme.border())
   |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/carets.rs:204:21
    |
204 |         Stroke::new(1.5, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.5_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/editor_mode.rs:53:21
   |
53 | ...   Stroke::new(1.0, if is_hybrid { theme.accent } else if hybrid_hover { theme.border().lerp_to_gamma(theme.accent, 0.4) } else {...
   |                   ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/editor_mode.rs:108:21
    |
108 | ...   Stroke::new(1.0, if is_vim { theme.accent } else if vim_hover { theme.border().lerp_to_gamma(theme.accent, 0.4) } else { them...
    |                   ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/keybindings_tab.rs:93:21
   |
93 |         Stroke::new(1.0, if reset_all_hov { theme.accent } else { theme.border() }),
   |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:143:21
    |
143 |         Stroke::new(1.0, if is_normal { theme.accent } else { theme.border() }),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:170:21
    |
170 |         Stroke::new(1.0, if is_visual { theme.accent } else { theme.border() }),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:196:21
    |
196 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:285:21
    |
285 |         Stroke::new(1.0, theme.border()),
    |                     ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:386:29
    |
386 |                 Stroke::new(1.0, if reset_hov { theme.accent } else { theme.border() }),
    |                             ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:416:25
    |
416 |             Stroke::new(1.0, if add_hov { theme.accent } else { theme.border() }),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:487:25
    |
487 |                         1.0,
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:663:25
    |
663 |             Stroke::new(1.0, if cap.staged_stroke.is_some() { theme.accent } else { theme.border() }),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:750:25
    |
750 |             Stroke::new(1.0, cancel_stroke),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
   --> app/src/settings/keybindings_tab.rs:785:25
    |
785 |             Stroke::new(1.0, save_stroke),
    |                         ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
    |
    = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
    = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>

warning: falling back to `f32` as the trait bound `f32: From<f64>` is not satisfied
  --> app/src/settings/setting_font.rs:68:17
   |
68 |                 1.0,
   |                 ^^^ help: explicitly specify the type as `f32`: `1.0_f32`
   |
   = warning: this was previously accepted by the compiler but is being phased out; it will become a hard error in a future release!
   = note: for more information, see issue #154024 <https://github.com/rust-lang/rust/issues/154024>
