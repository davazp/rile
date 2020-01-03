use std::cell::Cell;
use std::cmp;
use std::thread;
use std::time::Duration;

use crate::term;
use crate::Context;

/// Adjust the scroll level so the cursor is on the screen.
pub fn adjust_scroll(term: &term::Term, context: &Context) {
    let window = &context.window;
    let buffer = context.buffer_list.get_current_buffer();
    if buffer.cursor.line < window.scroll_line.get() {
        window.scroll_line.set(buffer.cursor.line);
    }
    if buffer.cursor.line > window.scroll_line.get() + window.get_window_lines(term) - 1 {
        window
            .scroll_line
            .set(buffer.cursor.line - (window.get_window_lines(term) - 1));
    }
}

pub struct Window {
    pub scroll_line: Cell<usize>,
    pub show_lines: bool,
}
impl Window {
    pub fn new() -> Window {
        Window {
            scroll_line: Cell::new(0),
            show_lines: false,
        }
    }

    pub fn get_window_lines(&self, term: &term::Term) -> usize {
        term.rows - 2
    }

    fn get_pad_width(&self, term: &term::Term) -> usize {
        if self.show_lines {
            let last_linenum_width =
                format!("{}", self.scroll_line.get() + self.get_window_lines(term)).len();
            last_linenum_width + 1
        } else {
            0
        }
    }

    fn render_cursor(&self, term: &mut term::Term, context: &Context) {
        let base = if context.buffer_list.minibuffer_focused {
            term.rows - 1
        } else {
            0
        };

        let buffer = context.buffer_list.get_current_buffer();

        term.set_cursor(
            base + buffer.cursor.line - self.scroll_line.get() + 1,
            buffer.cursor.column + self.get_pad_width(term) + 1,
        );
    }

    fn render_window(&self, term: &mut term::Term, context: &Context) {
        let offset = self.get_pad_width(term);
        let window_columns = term.columns - offset;

        term.set_cursor(1, 1);

        let window_lines = term.rows - 2;

        let offset = self.get_pad_width(term);

        let buffer = context.buffer_list.get_main_buffer();

        // Main window
        for row in 0..window_lines {
            let linenum = row + self.scroll_line.get();

            if let Some(line) = buffer.get_line(linenum) {
                if self.show_lines {
                    term.csi("38;5;240m");
                    term.write(&format!("{:width$} ", linenum + 1, width = offset - 1));
                }

                term.csi("m");
                term.write(&line[..cmp::min(line.len(), window_columns)]);
            }

            term.erase_line(term::ErasePart::ToEnd);
            term.write("\r\n");
        }
        term.csi("m");
    }

    fn render_modeline(&self, term: &mut term::Term, context: &Context) {
        let buffer = &context.buffer_list.get_main_buffer();

        term.csi("38;5;15m");
        term.csi("48;5;236m");

        let scroll_line = self.scroll_line.get();

        let buffer_progress = if scroll_line == 0 {
            "Top".to_string()
        } else if scroll_line + self.get_window_lines(term) >= buffer.lines_count() {
            "Bot".to_string()
        } else {
            format!("{}%", 100 * (buffer.cursor.line + 1) / buffer.lines_count())
        };

        // On MacOsX's terminal, when you erase a line it won't fill the
        // full line with the current attributes, unlike ITerm. So we use
        // `write_line` to pad the string with spaces.
        write_line(
            term,
            format!(
                "  {}  {} L{}",
                buffer.filename.as_ref().unwrap_or(&"*scratch*".to_string()),
                buffer_progress,
                buffer.cursor.line + 1
            ),
            term.columns,
        );
    }
}

fn render_minibuffer(term: &mut term::Term, context: &Context, flashed: bool) {
    if flashed {
        term.csi(";7m");
    } else {
        term.csi("m");
    }
    write_line(
        term,
        format!("{}", context.buffer_list.minibuffer.to_string()),
        term.columns,
    );
}

fn render_screen(term: &mut term::Term, context: &Context, flashed: bool) {
    let win = &context.window;

    term.hide_cursor();

    win.render_window(term, context);
    win.render_modeline(term, context);
    render_minibuffer(term, context, flashed);

    win.render_cursor(term, context);

    term.show_cursor();
    term.flush()
}

/// Refresh the screen.
///
/// Ensure the terminal reflects the latest state of the editor.
pub fn refresh_screen(term: &mut term::Term, context: &Context) {
    render_screen(term, context, false);
}

fn write_line<T: AsRef<str>>(term: &mut term::Term, str: T, width: usize) {
    let str = str.as_ref();
    assert!(str.len() <= width);
    let padded = format!("{:width$}", str, width = width);
    term.write(&padded);
}

pub fn ding(term: &mut term::Term, context: &Context) {
    render_screen(term, context, true);
    thread::sleep(Duration::from_millis(100));
    // Discard pending output. This avoids the situation where keeping
    // C-g press will overwhelm the event loop and hang the system
    // compmletely until completed.
    term::discard_input_buffer();
    render_screen(term, context, false);
}

/// Show a message in the minibuffer.
pub fn message<S: AsRef<str>>(context: &mut Context, str: S) {
    context.buffer_list.minibuffer.set(str);
}
