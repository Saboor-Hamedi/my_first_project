//! CharMapBuilder: Enforces the core invariant that char_map length
//! strictly equals total_glyphs_in_job + 1 (the EOL sentinel).

use eframe::egui::text::LayoutJob;
use eframe::egui::TextFormat;

/// Builder for constructing character maps with strict 1:1 invariant tracking.
pub struct CharMapBuilder {
    char_start: usize,
    map: Vec<usize>,
    expected_glyphs: usize,
}

impl CharMapBuilder {
    /// Creates a new builder for a line starting at buffer index `char_start`.
    pub fn new(char_start: usize, capacity: usize) -> Self {
        Self {
            char_start,
            map: Vec::with_capacity(capacity + 2),
            expected_glyphs: 0,
        }
    }

    /// Append `glyph_count` glyphs that all map back to `source_idx`.
    pub fn push_span(&mut self, source_idx: usize, glyph_count: usize) {
        for _ in 0..glyph_count {
            self.map.push(source_idx);
        }
        self.expected_glyphs += glyph_count;
    }

    /// Append a 1:1 run of glyphs mapping to `self.char_start + offset .. self.char_start + offset + count`.
    pub fn push_run(&mut self, offset: usize, count: usize) {
        for i in 0..count {
            self.map.push(self.char_start + offset + i);
        }
        self.expected_glyphs += count;
    }

    /// Appends the EOL sentinel and returns the completed character map.
    /// In debug builds, validates that `map.len() == expected_glyphs + 1`.
    pub fn finish(mut self, chars_len: usize) -> Vec<usize> {
        let eol_idx = self.char_start + chars_len;
        self.map.push(eol_idx);

        debug_assert_eq!(
            self.map.len(),
            self.expected_glyphs + 1,
            "CharMap invariant violated: map.len() ({}) != expected_glyphs + 1 ({})",
            self.map.len(),
            self.expected_glyphs + 1
        );

        self.map
    }

    /// Current number of mapped glyphs (excluding EOL sentinel).
    #[allow(dead_code)]
    #[inline]
    pub fn glyph_count(&self) -> usize {
        self.expected_glyphs
    }
}

/// Helper that appends text to a `LayoutJob` and immediately records matching glyphs
/// in `CharMapBuilder`, mapping all glyphs back to `source_idx`.
pub fn append_and_map(
    job: &mut LayoutJob,
    charmap: &mut CharMapBuilder,
    text: &str,
    source_idx: usize,
    fmt: TextFormat,
) {
    append_with_leading_space_and_map(job, charmap, text, source_idx, 0.0, fmt);
}

/// Helper that appends text to a `LayoutJob` with custom leading space in points.
pub fn append_with_leading_space_and_map(
    job: &mut LayoutJob,
    charmap: &mut CharMapBuilder,
    text: &str,
    source_idx: usize,
    leading_space: f32,
    fmt: TextFormat,
) {
    if text.is_empty() {
        return;
    }
    let glyphs = text.chars().count();
    job.append(text, leading_space, fmt);
    charmap.push_span(source_idx, glyphs);
}

/// Helper that appends a slice of characters to a `LayoutJob` and immediately records
/// a 1:1 mapping in `CharMapBuilder` for the slice range.
pub fn append_run_and_map(
    job: &mut LayoutJob,
    charmap: &mut CharMapBuilder,
    chars: &[char],
    range: std::ops::Range<usize>,
    fmt: TextFormat,
) {
    append_run_with_leading_space_and_map(job, charmap, chars, range, 0.0, fmt);
}

/// Helper that appends a slice of characters with custom leading space in points.
pub fn append_run_with_leading_space_and_map(
    job: &mut LayoutJob,
    charmap: &mut CharMapBuilder,
    chars: &[char],
    range: std::ops::Range<usize>,
    leading_space: f32,
    fmt: TextFormat,
) {
    if range.is_empty() {
        return;
    }
    let s: String = chars[range.clone()].iter().collect();
    let glyphs = s.chars().count();
    job.append(&s, leading_space, fmt);
    charmap.push_run(range.start, glyphs);
}
