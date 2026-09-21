//! Motion execution engine — executes movements and selections with count multipliers.
//!
//! For example, `3j` executes down visual motion 3 times; `5w` jumps forward 5 words.

use crate::editor::{Editor, VisualLine};
use crate::vim::types::VimMotion;

/// Executes a motion in Normal mode `count` times.
pub fn execute_normal_motion(ed: &mut Editor, lines: &[VisualLine], motion: VimMotion, count: usize) {
    let iterations = count.max(1);

    match motion {
        VimMotion::Left => {
            for _ in 0..iterations {
                ed.left();
            }
        }
        VimMotion::Right => {
            for _ in 0..iterations {
                ed.right();
            }
        }
        VimMotion::UpVisual => {
            for _ in 0..iterations {
                ed.up_visual(lines);
            }
        }
        VimMotion::DownVisual => {
            for _ in 0..iterations {
                ed.down_visual(lines);
            }
        }
        VimMotion::WordForward => {
            for _ in 0..iterations {
                ed.word_right();
            }
        }
        VimMotion::WordBackward => {
            for _ in 0..iterations {
                ed.word_left();
            }
        }
        VimMotion::LineStart => {
            ed.home_visual(lines);
        }
        VimMotion::LineEnd => {
            ed.end_visual(lines);
        }
        VimMotion::BufferStart => {
            ed.cur = 0;
            ed.clear_selection();
        }
        VimMotion::BufferEnd => {
            ed.cur = ed.buf.len();
            ed.clear_selection();
        }
    }
}

/// Executes a motion in Visual mode `count` times, expanding the active selection.
pub fn execute_visual_motion(
    ed: &mut Editor,
    lines: &[VisualLine],
    motion: VimMotion,
    is_visual_line: bool,
    count: usize,
) {
    let iterations = count.max(1);

    match motion {
        VimMotion::Left => {
            for _ in 0..iterations {
                ed.left_select();
            }
        }
        VimMotion::Right => {
            for _ in 0..iterations {
                ed.right_select();
            }
        }
        VimMotion::UpVisual => {
            for _ in 0..iterations {
                ed.up_visual_select(lines);
            }
            if is_visual_line {
                let (start, _) = ed.current_line_span();
                ed.cur = start;
            }
        }
        VimMotion::DownVisual => {
            for _ in 0..iterations {
                ed.down_visual_select(lines);
            }
            if is_visual_line {
                let (_, end) = ed.current_line_span();
                ed.cur = end;
            }
        }
        VimMotion::WordForward => {
            for _ in 0..iterations {
                ed.word_right_select();
            }
        }
        VimMotion::WordBackward => {
            for _ in 0..iterations {
                ed.word_left_select();
            }
        }
        VimMotion::LineStart => {
            ed.home_visual_select(lines);
        }
        VimMotion::LineEnd => {
            ed.end_visual_select(lines);
        }
        VimMotion::BufferStart => {
            ed.cur = 0;
            if is_visual_line {
                let (start, _) = ed.current_line_span();
                ed.cur = start;
            }
        }
        VimMotion::BufferEnd => {
            ed.cur = ed.buf.len();
            if is_visual_line {
                let (_, end) = ed.current_line_span();
                ed.cur = end;
            }
        }
    }
}
