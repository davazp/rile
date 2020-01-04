use std::cmp;
use std::thread;
use std::time::Duration;

use crate::buffer_list::BufferRef;
use crate::layout;
use crate::term;
use crate::Context;

/// Adjust the scroll level so the cursor is on the screen.
pub fn adjust_scroll(term: &term::Term, context: &mut Context) {
    let region = layout::get_current_window_region(term, context);
    let window = context.window_list.get_current_window_as_mut();
    let buffer = context.buffer_list.resolve_ref(window.buffer_ref);

    if buffer.cursor.line < window.first_visible_line() {
        window.scroll_line = buffer.cursor.line;
    }

    let last_visible_line = window.last_visible_line(&region);
    if buffer.cursor.line > last_visible_line {
        window.scroll_line = buffer.cursor.line - window.window_lines(&region) + 1;
    }
}

pub struct Window {
    pub scroll_line: usize,
    pub show_lines: bool,
    pub show_modeline: bool,

    pub buffer_ref: BufferRef,
}
impl Window {
    pub fn new(buffer_ref: BufferRef, show_modeline: bool) -> Window {
        Window {
            scroll_line: 0,
            show_lines: false,
            show_modeline,
            buffer_ref,
        }
    }

    fn get_pad_width(&self, region: &layout::Region) -> usize {
        if self.show_lines {
            let last_linenum_width = format!("{}", self.scroll_line + region.height).len();
            last_linenum_width + 1
        } else {
            0
        }
    }

    fn render_cursor(&self, term: &mut term::Term, context: &Context, region: &layout::Region) {
        let buffer = context.buffer_list.resolve_ref(self.buffer_ref);

        let screen_line = buffer.cursor.line.checked_sub(self.scroll_line);

        if let Some(row) = screen_line {
            term.set_cursor(
                region.top + row + 1,
                buffer.cursor.column + self.get_pad_width(region) + 1,
            );
        }
    }

    fn render_window(
        &self,
        term: &mut term::Term,
        context: &Context,
        region: &layout::Region,
        _flashed: bool,
    ) {
        let offset = self.get_pad_width(region);
        let window_columns = term.columns - offset;

        let buffer = context.buffer_list.resolve_ref(self.buffer_ref);

        // Main window
        for row in 0..self.window_lines(region) {
            let linenum = row + self.scroll_line;

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
            term.write_line(line_content, window_columns);
        }

        term.csi("m");
    }

    fn render_modeline(&self, term: &mut term::Term, context: &Context, region: &layout::Region) {
        let buffer = &context.buffer_list.resolve_ref(self.buffer_ref);

        term.csi("38;5;15m");
        term.csi("48;5;236m");

        let scroll_line = self.scroll_line;

        let buffer_progress = if scroll_line == 0 {
            "Top".to_string()
        } else if self.last_visible_line(region) >= buffer.lines_count() - 1 {
            "Bot".to_string()
        } else {
            format!("{}%", 100 * (buffer.cursor.line + 1) / buffer.lines_count())
        };

        // On MacOsX's terminal, when you erase a line it won't fill the
        // full line with the current attributes, unlike ITerm. So we use
        // `write_line` to pad the string with spaces.
        term.write_line(
            format!(
                "  {}  {} L{}",
                buffer.filename.as_ref().unwrap_or(&"*scratch*".to_string()),
                buffer_progress,
                buffer.cursor.line + 1
            ),
            term.columns,
        );
    }

    fn first_visible_line(&self) -> usize {
        self.scroll_line
    }

    pub fn window_lines(&self, region: &layout::Region) -> usize {
        if self.show_modeline {
            region.height - 1
        } else {
            region.height
        }
    }

    fn last_visible_line(&self, region: &layout::Region) -> usize {
        self.scroll_line + self.window_lines(region) - 1
    }

    // last: if this window is being rendered over the last
    fn render(
        &self,
        term: &mut term::Term,
        context: &Context,
        region: &layout::Region,
        flashed: bool,
    ) {
        self.render_window(term, context, region, flashed);
        if self.show_modeline {
            self.render_modeline(term, context, region);
        }
    }
}

fn render_screen(term: &mut term::Term, context: &Context, flashed: bool) {
    let main_window = &context.window_list.main;
    let minibuffer_window = &context.window_list.minibuffer;

    term.hide_cursor();

    let layout = layout::get_layout(term, context);

    term.set_cursor(1, 1);

    main_window.render(term, context, &layout.main_window_region, flashed);
    context
        .window_list
        .minibuffer
        .render(term, context, &layout.minibuffer_region, flashed);

    if context.window_list.minibuffer_focused {
        minibuffer_window.render_cursor(term, context, &layout.minibuffer_region);
    } else {
        main_window.render_cursor(term, context, &layout.main_window_region);
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
