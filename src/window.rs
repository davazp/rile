use std::cmp;

use crate::context::Context;
use crate::term::{ErasePart, Term};

/// Adjust the scroll level so the cursor is on the screen.
pub fn adjust_scroll(term: &Term, window: &mut Window, context: &mut Context) {
    if context.cursor.line < window.scroll_line {
        window.scroll_line = context.cursor.line;
    }
    if context.cursor.line > window.scroll_line + window.get_window_lines(term) - 1 {
        window.scroll_line = context.cursor.line - (window.get_window_lines(term) - 1);
    }
}

pub struct Window {
    pub scroll_line: usize,
    pub show_lines: bool,
}
impl Window {
    pub fn get_window_lines(&self, term: &Term) -> usize {
        term.rows - 2
    }

    fn get_pad_width(&self, term: &Term) -> usize {
        if self.show_lines {
            let last_linenum_width =
                format!("{}", self.scroll_line + self.get_window_lines(term)).len();
            last_linenum_width + 1
        } else {
            0
        }
    }

    fn render_cursor(&self, term: &mut Term, context: &Context) {
        term.set_cursor(
            context.cursor.line - self.scroll_line + 1,
            context.cursor.column + self.get_pad_width(term) + 1,
        );
    }

    fn render_window(&self, term: &mut Term, context: &Context) {
        let offset = self.get_pad_width(term);
        let window_columns = term.columns - offset;

        term.set_cursor(1, 1);

        let window_lines = term.rows - 2;

        let offset = self.get_pad_width(term);

        let buffer = &context.current_buffer;

        // Main window
        for row in 0..window_lines {
            let linenum = row + self.scroll_line;

            if let Some(line) = buffer.get_line(linenum) {
                if self.show_lines {
                    term.csi("38;5;240m");
                    term.write(&format!("{:width$} ", linenum + 1, width = offset - 1));
                }

                term.csi("m");
                term.write(&line[..cmp::min(line.len(), window_columns)]);
            }

            term.erase_line(ErasePart::ToEnd);
            term.write("\r\n");
        }
        term.csi("m");
    }

    fn render_modeline(&self, term: &mut Term, context: &Context) {
        term.csi("38;5;15m");
        term.csi("48;5;236m");

        let buffer_progress = if self.scroll_line == 0 {
            "Top".to_string()
        } else if self.scroll_line + self.get_window_lines(term)
            >= context.current_buffer.lines_count()
        {
            "Bot".to_string()
        } else {
            format!(
                "{}%",
                100 * (context.cursor.line + 1) / context.current_buffer.lines_count()
            )
        };

        // On MacOsX's terminal, when you erase a line it won't fill the
        // full line with the current attributes, unlike ITerm. So we use
        // `write_line` to pad the string with spaces.
        write_line(
            term,
            &format!(
                "  {}  {} L{}",
                context
                    .current_buffer
                    .filename
                    .as_ref()
                    .unwrap_or(&"*scratch*".to_string()),
                buffer_progress,
                context.cursor.line + 1
            ),
            term.columns,
        );
    }
}

fn render_minibuffer(term: &mut Term, context: &Context) {
    term.csi("m");
    write_line(
        term,
        &format!("{}", context.minibuffer.to_string()),
        term.columns,
    );
}

/// Refresh the screen.
///
/// Ensure the terminal reflects the latest state of the editor.
pub fn refresh_screen(term: &mut Term, win: &Window, context: &Context) {
    term.hide_cursor();

    win.render_window(term, context);
    win.render_modeline(term, context);
    render_minibuffer(term, context);

    win.render_cursor(term, context);

    term.show_cursor();
    term.flush()
}

fn write_line(term: &mut Term, str: &str, width: usize) {
    assert!(str.len() <= width);
    let padded = format!("{:width$}", str, width = width);
    term.write(&padded);
}
