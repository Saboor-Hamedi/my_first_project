//! List metrics and glyph constants for bullets, task checkboxes, and numbered items.

/// Standard bullet character with trailing padding.
#[inline]
pub fn bullet_glyph() -> &'static str {
    "•  "
}

/// Standard unicode checkbox glyphs for task items.
#[inline]
#[allow(dead_code)]
pub fn checkbox_glyph(checked: bool) -> &'static str {
    if checked {
        "☑ "
    } else {
        "☐ "
    }
}

/// Standard numbered list prefix glyph.
#[inline]
pub fn number_glyph(num: &str) -> String {
    format!("{num}. ")
}
