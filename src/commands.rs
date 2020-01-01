use std::cmp;

use crate::buffer;
use crate::buffer::Cursor;
use crate::context::{Context, GoalColumn};
use crate::read;
use crate::term::Term;

pub type Result = std::result::Result<(), ()>;

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

pub fn move_beginning_of_line(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
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
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let eol = buffer.get_line_unchecked(buffer.cursor.line).len();
    buffer.cursor.column = eol;
    Ok(())
}

pub fn forward_char(context: &mut Context, term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
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
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    if buffer.cursor.column > 0 {
        buffer.cursor.column -= 1;
    } else {
        previous_line(context, term)?;
        move_end_of_line(context, term)?;
    }
    Ok(())
}

fn get_or_set_gaol_column(cursor: &Cursor, goal_column: &mut GoalColumn) -> usize {
    // We set `to_preserve` to ensure the goal_column is
    // not lost for the next command.
    goal_column.to_preserve = true;
    *goal_column.column.get_or_insert(cursor.column)
}

pub fn next_line(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    if buffer.cursor.line < buffer.lines_count() - 1 {
        let goal_column = get_or_set_gaol_column(&buffer.cursor, &mut context.goal_column);
        buffer.cursor.line += 1;
        buffer.cursor.column = cmp::min(
            buffer.get_line_unchecked(buffer.cursor.line).len(),
            goal_column,
        );
        Ok(())
    } else {
        context.buffer_list.minibuffer.set("End of buffer");
        Err(())
    }
}

pub fn previous_line(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    if buffer.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(&buffer.cursor, &mut context.goal_column);
        buffer.cursor.line -= 1;
        buffer.cursor.column = cmp::min(
            buffer.get_line_unchecked(buffer.cursor.line).len(),
            goal_column,
        );
        Ok(())
    } else {
        context.buffer_list.minibuffer.set("Beginning of buffer");
        Err(())
    }
}

pub fn insert_char(context: &mut Context, ch: char) {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
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
    let buffer = context.buffer_list.get_current_buffer_as_mut();

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
    let buffer = context.buffer_list.get_current_buffer_as_mut();
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
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let Cursor { line, column } = buffer.cursor;
    let line = buffer.get_line_mut_unchecked(line);
    let newline = line.split_off(column);
    buffer.insert_line_at(buffer.cursor.line + 1, newline);
    buffer.cursor.line += 1;
    buffer.cursor.column = 0;
    Ok(())
}

pub fn indent_line(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let line = buffer.get_line_unchecked(buffer.cursor.line);
    let indent = get_line_indentation(line);
    if buffer.cursor.column < indent {
        buffer.cursor.column = indent;
    }
    Ok(())
}

pub fn save_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let buffer_list = &mut context.buffer_list;

    let result = {
        let buffer = buffer_list.get_current_buffer_as_mut();
        buffer.save()
    };

    match result {
        Ok(filename) => {
            buffer_list.minibuffer.set(format!("Wrote {}", filename));
            Ok(())
        }
        Err(buffer::SaveError::NoFile) => {
            buffer_list.minibuffer.set("No file");
            Err(())
        }
        Err(buffer::SaveError::IoError(_)) => {
            buffer_list.minibuffer.set("Could not save file");
            Err(())
        }
    }
}

const CONTEXT_LINES: usize = 2;

pub fn next_screen(context: &mut Context, term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let window = &context.window;
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    let target = window.scroll_line.get() + offset;
    if target < buffer.lines_count() {
        window.scroll_line.set(target);
        buffer.cursor.line = target;
        Ok(())
    } else {
        context.buffer_list.minibuffer.set("End of buffer");
        Err(())
    }
}

pub fn previous_screen(context: &mut Context, term: &mut Term) -> Result {
    let window = &context.window;
    let buffer = context.buffer_list.get_current_buffer_as_mut();

    if window.scroll_line.get() == 0 {
        context.buffer_list.minibuffer.set("Beginning of buffer");
        return Err(());
    }
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    buffer.cursor.line = window.scroll_line.get() + CONTEXT_LINES;
    window.scroll_line.set(
        if let Some(scroll_line) = window.scroll_line.get().checked_sub(offset) {
            scroll_line
        } else {
            0
        },
    );
    Ok(())
}

pub fn beginning_of_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    buffer.cursor.line = 0;
    buffer.cursor.column = 0;
    Ok(())
}

pub fn end_of_buffer(context: &mut Context, _term: &mut Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let linenum = buffer.lines_count() - 1;
    buffer.cursor.line = linenum;
    let line = buffer.get_line_unchecked(linenum);
    buffer.cursor.column = line.len();
    Ok(())
}

pub fn kill_emacs(context: &mut Context, _term: &mut Term) -> Result {
    context.event_loop.complete(Ok(()));
    Ok(())
}

pub fn m_x(context: &mut Context, term: &mut Term) -> Result {
    if let Ok(str) = read::read_string(term, context, "M-x ") {
        context.buffer_list.get_current_buffer_as_mut().set(str);
    }
    Ok(())
}

pub fn keyboard_quit(context: &mut Context, _term: &mut Term) -> Result {
    context.event_loop.complete(Err(()));
    Ok(())
}
