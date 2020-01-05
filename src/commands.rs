use std::cmp;

use crate::buffer;
use crate::context;
use crate::layout;
use crate::read;
use crate::term::Term;
use crate::window::{self, message};
use crate::{Context, Cursor};

pub type Result = std::result::Result<(), ()>;

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

pub fn move_beginning_of_line(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let mut buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let line = buffer.get_line_unchecked(buffer.cursor.line);
    let indentation = get_line_indentation(line);
    buffer.cursor.column = if buffer.cursor.column <= indentation {
        0
    } else {
        indentation
    };
    Ok(())
}

pub fn move_end_of_line(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let eol = buffer.get_line_unchecked(buffer.cursor.line).len();
    buffer.cursor.column = eol;
    Ok(())
}

pub fn forward_char(context: &mut Context, term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let len = buffer.get_line_unchecked(buffer.cursor.line).len();
    if buffer.cursor.column < len {
        buffer.cursor.column += 1;
    } else {
        buffer.cursor.column = 0;
        next_line(context, term)?;
    }
    Ok(())
}

pub fn backward_char(context: &mut Context, term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    if buffer.cursor.column > 0 {
        buffer.cursor.column -= 1;
    } else {
        previous_line(context, term)?;
        move_end_of_line(context, term)?;
    }
    Ok(())
}

fn get_or_set_gaol_column(cursor: &Cursor, goal_column: &mut context::GoalColumn) -> usize {
    // We set `to_preserve` to ensure the goal_column is
    // not lost for the next command.
    goal_column.to_preserve = true;
    *goal_column.column.get_or_insert(cursor.column)
}

pub fn next_line(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    if buffer.cursor.line < buffer.lines_count() - 1 {
        let goal_column = get_or_set_gaol_column(&buffer.cursor, &mut context.goal_column);
        buffer.cursor.line += 1;
        buffer.cursor.column = cmp::min(
            buffer.get_line_unchecked(buffer.cursor.line).len(),
            goal_column,
        );
        Ok(())
    } else {
        message(context, "End of buffer");
        Err(())
    }
}

pub fn previous_line(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    if buffer.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(&buffer.cursor, &mut context.goal_column);
        buffer.cursor.line -= 1;
        buffer.cursor.column = cmp::min(
            buffer.get_line_unchecked(buffer.cursor.line).len(),
            goal_column,
        );
        Ok(())
    } else {
        message(context, "Beginning of buffer");
        Err(())
    }
}

pub fn insert_char(context: &mut Context, ch: char) {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let idx = buffer.cursor.column;
    let line = buffer.get_line_mut_unchecked(buffer.cursor.line);
    line.insert(idx, ch);
    buffer.cursor.column += 1;
}

pub fn delete_char(context: &mut Context, term: &mut Term) -> Result {
    forward_char(context, term)?;
    delete_backward_char(context, term)?;
    Ok(())
}

pub fn delete_backward_char(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);

    if buffer.cursor.column > 0 {
        buffer.cursor.column -= 1;
        buffer.remove_char_at(buffer.cursor.line, buffer.cursor.column);
    } else if buffer.cursor.line > 0 {
        let line = buffer.remove_line(buffer.cursor.line);
        let previous_line = buffer.get_line_mut_unchecked(buffer.cursor.line - 1);
        let previous_line_original_length = previous_line.len();
        previous_line.push_str(&line);

        buffer.cursor.line -= 1;
        buffer.cursor.column = previous_line_original_length;
    }

    Ok(())
}

pub fn kill_line(context: &mut Context, term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let Cursor { line, column } = buffer.cursor;
    let line = buffer.get_line_mut_unchecked(line);
    if column == line.len() {
        if buffer.cursor.line < buffer.lines_count() - 1 {
            delete_char(context, term)?;
        }
    } else {
        line.drain(column..);
    }

    Ok(())
}

pub fn newline(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let Cursor { line, column } = buffer.cursor;
    let line = buffer.get_line_mut_unchecked(line);
    let newline = line.split_off(column);
    buffer.insert_line_at(buffer.cursor.line + 1, newline);
    buffer.cursor.line += 1;
    buffer.cursor.column = 0;
    Ok(())
}

pub fn indent_line(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let line = buffer.get_line_unchecked(buffer.cursor.line);
    let indent = get_line_indentation(line);
    if buffer.cursor.column < indent {
        buffer.cursor.column = indent;
    }
    Ok(())
}

pub fn save_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);

    match buffer.save() {
        Ok(filename) => {
            message(context, format!("Wrote {}", filename));
            Ok(())
        }
        Err(buffer::SaveError::NoFile) => {
            message(context, "No file");
            Err(())
        }
        Err(buffer::SaveError::IoError(_)) => {
            message(context, "Could not save file");
            Err(())
        }
    }
}

const CONTEXT_LINES: usize = 2;

pub fn next_screen(context: &mut Context, term: &mut Term) -> Result {
    let region = layout::get_current_window_region(term, context);
    let window = context.window_list.get_current_window_as_mut();
    let offset = window.window_lines(&region) - 1 - CONTEXT_LINES;

    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);

    let target = window.scroll_line + offset;
    if target < buffer.lines_count() {
        window.scroll_line = target;
        buffer.cursor.line = target;
        Ok(())
    } else {
        message(context, "End of buffer");
        Err(())
    }
}

pub fn previous_screen(context: &mut Context, term: &mut Term) -> Result {
    let region = layout::get_current_window_region(term, context);
    let window = context.window_list.get_current_window_as_mut();

    if window.scroll_line == 0 {
        message(context, "Beginning of buffer");
        return Err(());
    }

    let offset = window.window_lines(&region) - 1 - CONTEXT_LINES;

    let mut buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    buffer.cursor.line = window.scroll_line + CONTEXT_LINES;

    window.scroll_line = if let Some(scroll_line) = window.scroll_line.checked_sub(offset) {
        scroll_line
    } else {
        0
    };

    Ok(())
}

pub fn beginning_of_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    buffer.cursor.line = 0;
    buffer.cursor.column = 0;
    Ok(())
}

pub fn end_of_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let window = context.window_list.get_current_window();
    let buffer = context.buffer_list.resolve_ref_as_mut(window.buffer_ref);
    let linenum = buffer.lines_count() - 1;
    buffer.cursor.line = linenum;
    let line = buffer.get_line_unchecked(linenum);
    buffer.cursor.column = line.len();
    Ok(())
}

pub fn kill_rile(context: &mut Context, _term: &mut Term) -> Result {
    context.event_loop.complete(Ok(()));
    Ok(())
}

pub fn isearch_forward(context: &mut Context, term: &mut Term) -> Result {
    let _ = read::read_string(term, context, "Search: ", |_, _| {})?;
    Ok(())
}

pub fn keyboard_quit(context: &mut Context, term: &mut Term) -> Result {
    message(context, "Quit");
    window::ding(term, context);
    context.event_loop.complete(Err(()));
    Ok(())
}
