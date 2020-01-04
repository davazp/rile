use std::cell::Cell;
use std::cmp;
use std::thread;
use std::time::Duration;

use crate::buffer_list::BufferRef;
use crate::term;
use crate::Context;

/// Adjust the scroll level so the cursor is on the screen.
pub fn adjust_scroll(term: &term::Term, context: &Context) {
    let window = &context.main_window;
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

pub struct Region {
    top: usize,
    height: usize,
}

pub struct Window {
    pub scroll_line: Cell<usize>,
    pub show_lines: bool,
    pub show_modeline: bool,

    pub buffer_ref: BufferRef,
}
impl Window {
    pub fn new(buffer_ref: BufferRef, show_modeline: bool, show_lines: bool) -> Window {
        Window {
            scroll_line: Cell::new(0),
            show_lines,
            show_modeline,
            buffer_ref,
        }
    }

    pub fn get_window_lines(&self, term: &term::Term) -> usize {
        let modeline_height = if self.show_modeline { 1 } else { 0 };
        let minibuffer_height = 1;
        term.rows - modeline_height - minibuffer_height
    }

    fn get_pad_width(&self, region: &Region) -> usize {
        if self.show_lines {
            let last_linenum_width = format!("{}", self.scroll_line.get() + region.height).len();
            last_linenum_width + 1
        } else {
            0
        }
    }

    fn render_cursor(&self, term: &mut term::Term, context: &Context, region: &Region) {
        let buffer = context
            .buffer_list
            .resolve_ref(self.buffer_ref)
            .expect("can't render window because the buffer does not exist anymore.");

        term.set_cursor(
            region.top + buffer.cursor.line - self.scroll_line.get() + 1,
            buffer.cursor.column + self.get_pad_width(region) + 1,
        );
    }

    fn render_window(
        &self,
        term: &mut term::Term,
        context: &Context,
        region: &Region,
        _flashed: bool,
    ) {
        let offset = self.get_pad_width(region);
        let window_columns = term.columns - offset;

        let buffer = context
            .buffer_list
            .resolve_ref(self.buffer_ref)
            .expect("can't render a buffer that has been removed.");

        // Main window
        for row in 0..region.height {
            let linenum = row + self.scroll_line.get();

            let (line_content, line_present) = if let Some(line) = buffer.get_line(linenum) {
                (&line[..cmp::min(line.len(), window_columns)], true)
            } else {
                ("", false)
            };

            if self.show_lines && line_present {
                term.csi("38;5;240m");
                term.write(&format!("{:width$} ", linenum + 1, width = offset - 1));
            } else {
                term.write(&format!("{:width$}", "", width = offset))
            }

            term.csi("m");
            write_line(term, line_content, window_columns);
        }

        term.csi("m");
    }

    fn render_modeline(&self, term: &mut term::Term, context: &Context) {
        let buffer = &context
            .buffer_list
            .resolve_ref(self.buffer_ref)
            .expect("can't render a buffer that has been deleted.");

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

    // last: if this window is being rendered over the last
    fn render(&self, term: &mut term::Term, context: &Context, region: &Region, flashed: bool) {
        if self.show_modeline {
            self.render_window(
                term,
                context,
                &Region {
                    top: region.top,
                    height: region.height - 1,
                },
                flashed,
            );
            self.render_modeline(term, context);
        } else {
            self.render_window(
                term,
                context,
                &Region {
                    top: region.top,
                    height: region.height,
                },
                flashed,
            );
        }
    }
}

fn render_screen(term: &mut term::Term, context: &Context, flashed: bool) {
    let main_window = &context.main_window;
    let minibuffer_window = &context.minibuffer_window;

    term.hide_cursor();

    let minibuffer_height = context.buffer_list.minibuffer.lines_count();

    let minibuffer_region = Region {
        top: term.rows - minibuffer_height,
        height: minibuffer_height,
    };

    let main_window_region = Region {
        top: 0,
        height: term.rows - minibuffer_height,
    };

    term.set_cursor(1, 1);

    main_window.render(term, context, &main_window_region, flashed);
    context
        .minibuffer_window
        .render(term, context, &minibuffer_region, flashed);

    if context.buffer_list.minibuffer_focused {
        minibuffer_window.render_cursor(term, context, &minibuffer_region);
    } else {
        main_window.render_cursor(term, context, &main_window_region);
    }

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
