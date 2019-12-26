use std::cmp;
use std::fs;

use crate::context::Context;
use crate::term::Term;
use crate::window::Window;

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

pub fn move_beginning_of_line(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_unchecked(context.cursor.line);
    let indentation = get_line_indentation(line);
    context.cursor.column = if context.cursor.column <= indentation {
        0
    } else {
        indentation
    };
}

pub fn move_end_of_line(context: &mut Context) {
    let eol = context
        .current_buffer
        .get_line_unchecked(context.cursor.line)
        .len();
    context.cursor.column = eol;
}

pub fn forward_char(context: &mut Context) {
    let len = context
        .current_buffer
        .get_line_unchecked(context.cursor.line)
        .len();
    if context.cursor.column < len {
        context.cursor.column += 1;
    } else {
        if next_line(context) {
            context.cursor.column = 0;
        };
    }
}

pub fn backward_char(context: &mut Context) {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
    } else {
        if previous_line(context) {
            move_end_of_line(context);
        };
    }
}

pub fn get_or_set_gaol_column(context: &mut Context) -> usize {
    // We set `to_preserve_goal_column` to ensure the goal_column is
    // not lost for the next command.
    context.to_preserve_goal_column = true;
    *context.goal_column.get_or_insert(context.cursor.column)
}

pub fn next_line(context: &mut Context) -> bool {
    if context.cursor.line < context.current_buffer.lines_count() - 1 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line += 1;
        context.cursor.column = cmp::min(
            context
                .current_buffer
                .get_line_unchecked(context.cursor.line)
                .len(),
            goal_column,
        );
        true
    } else {
        context.minibuffer.set("End of buffer");
        false
    }
}

pub fn previous_line(context: &mut Context) -> bool {
    if context.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(context);
        context.cursor.line -= 1;
        context.cursor.column = cmp::min(
            context
                .current_buffer
                .get_line_unchecked(context.cursor.line)
                .len(),
            goal_column,
        );
        true
    } else {
        context.minibuffer.set("Beginning of buffer");
        false
    }
}

pub fn insert_char(context: &mut Context, ch: char) {
    let idx = context.cursor.column;
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    line.insert(idx, ch);
    context.cursor.column += 1;
}

pub fn delete_char(context: &mut Context) {
    forward_char(context);
    delete_backward_char(context);
}

pub fn delete_backward_char(context: &mut Context) {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
        context
            .current_buffer
            .remove_char_at(context.cursor.line, context.cursor.column);
    } else if context.cursor.line > 0 {
        let line = context.current_buffer.remove_line(context.cursor.line);
        let previous_line = context
            .current_buffer
            .get_line_mut_unchecked(context.cursor.line - 1);
        let previous_line_original_length = previous_line.len();
        previous_line.push_str(&line);

        context.cursor.line -= 1;
        context.cursor.column = previous_line_original_length;
    }
}

pub fn kill_line(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    if context.cursor.column == line.len() {
        if context.cursor.line < context.current_buffer.lines_count() - 1 {
            delete_char(context);
        }
    } else {
        line.drain(context.cursor.column..);
    }
}

pub fn newline(context: &mut Context) {
    let line = context
        .current_buffer
        .get_line_mut_unchecked(context.cursor.line);
    let newline = line.split_off(context.cursor.column);
    context
        .current_buffer
        .insert_at(context.cursor.line + 1, newline);

    context.cursor.line += 1;
    context.cursor.column = 0;
}

pub fn indent_line(context: &mut Context) {
    let line = &context
        .current_buffer
        .get_line_unchecked(context.cursor.line);
    let indent = get_line_indentation(line);
    if context.cursor.column < indent {
        context.cursor.column = indent;
    }
}

pub fn save_buffer(context: &mut Context) {
    let buffer = &context.current_buffer;
    let contents = buffer.to_string();
    if let Some(filename) = &buffer.filename {
        match fs::write(filename, contents) {
            Ok(_) => {
                context.minibuffer.set(&format!("Wrote {}", filename));
            }
            Err(_) => {
                context.minibuffer.set("Could not save file");
            }
        }
    } else {
        context.minibuffer.set("No file");
    }
}

const CONTEXT_LINES: usize = 2;

pub fn next_screen(context: &mut Context, window: &mut Window, term: &Term) {
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    let target = window.scroll_line + offset;
    if target < context.current_buffer.lines_count() {
        window.scroll_line = target;
        context.cursor.line = target;
    } else {
        context.minibuffer.set("End of buffer");
    }
}

pub fn previous_screen(context: &mut Context, window: &mut Window, term: &Term) {
    if window.scroll_line == 0 {
        context.minibuffer.set("Beginning of buffer");
        return;
    }
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    context.cursor.line = window.scroll_line + CONTEXT_LINES;
    window.scroll_line = if let Some(scroll_line) = window.scroll_line.checked_sub(offset) {
        scroll_line
    } else {
        0
    };
}
