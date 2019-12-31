use std::cmp;

use crate::buffer;
use crate::context::{Context, Cursor, GoalColumn};
use crate::term::Term;

pub type Result = std::result::Result<(), ()>;

fn get_line_indentation(line: &str) -> usize {
    line.chars().position(|ch| !ch.is_whitespace()).unwrap_or(0)
}

pub fn move_beginning_of_line(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let line = buffer.get_line_unchecked(context.cursor.line);
    let indentation = get_line_indentation(line);
    context.cursor.column = if context.cursor.column <= indentation {
        0
    } else {
        indentation
    };
    Ok(())
}

pub fn move_end_of_line(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let eol = buffer.get_line_unchecked(context.cursor.line).len();
    context.cursor.column = eol;
    Ok(())
}

pub fn forward_char(context: &mut Context, term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let len = buffer.get_line_unchecked(context.cursor.line).len();
    if context.cursor.column < len {
        context.cursor.column += 1;
    } else {
        next_line(context, term)?;
        context.cursor.column = 0;
    }
    Ok(())
}

pub fn backward_char(context: &mut Context, term: &Term) -> Result {
    if context.cursor.column > 0 {
        context.cursor.column -= 1;
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

pub fn next_line(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    if context.cursor.line < buffer.lines_count() - 1 {
        let goal_column = get_or_set_gaol_column(&context.cursor, &mut context.goal_column);
        context.cursor.line += 1;
        context.cursor.column = cmp::min(
            buffer.get_line_unchecked(context.cursor.line).len(),
            goal_column,
        );
        Ok(())
    } else {
        context.buffer_list.minibuffer.set("End of buffer");
        Err(())
    }
}

pub fn previous_line(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    if context.cursor.line > 0 {
        let goal_column = get_or_set_gaol_column(&context.cursor, &mut context.goal_column);
        context.cursor.line -= 1;
        context.cursor.column = cmp::min(
            buffer.get_line_unchecked(context.cursor.line).len(),
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
    let idx = context.cursor.column;
    let line = buffer.get_line_mut_unchecked(context.cursor.line);
    line.insert(idx, ch);
    context.cursor.column += 1;
}

pub fn delete_char(context: &mut Context, term: &Term) -> Result {
    forward_char(context, term)?;
    delete_backward_char(context, term)?;
    Ok(())
}

pub fn delete_backward_char(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();

    if context.cursor.column > 0 {
        context.cursor.column -= 1;
        buffer.remove_char_at(context.cursor.line, context.cursor.column);
    } else if context.cursor.line > 0 {
        let line = buffer.remove_line(context.cursor.line);
        let previous_line = buffer.get_line_mut_unchecked(context.cursor.line - 1);
        let previous_line_original_length = previous_line.len();
        previous_line.push_str(&line);

        context.cursor.line -= 1;
        context.cursor.column = previous_line_original_length;
    }

    Ok(())
}

pub fn kill_line(context: &mut Context, term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let line = buffer.get_line_mut_unchecked(context.cursor.line);
    if context.cursor.column == line.len() {
        if context.cursor.line < buffer.lines_count() - 1 {
            delete_char(context, term)?;
        }
    } else {
        line.drain(context.cursor.column..);
    }

    Ok(())
}

pub fn newline(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer_as_mut();
    let line = buffer.get_line_mut_unchecked(context.cursor.line);
    let newline = line.split_off(context.cursor.column);
    buffer.insert_line_at(context.cursor.line + 1, newline);
    context.cursor.line += 1;
    context.cursor.column = 0;
    Ok(())
}

pub fn indent_line(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let line = buffer.get_line_unchecked(context.cursor.line);
    let indent = get_line_indentation(line);
    if context.cursor.column < indent {
        context.cursor.column = indent;
    }
    Ok(())
}

pub fn save_buffer(context: &mut Context, _term: &Term) -> Result {
    let buffer_list = &mut context.buffer_list;

    let result = {
        let buffer = buffer_list.get_current_buffer_as_mut();
        buffer.save()
    };

    match result {
        Ok(filename) => {
            buffer_list.minibuffer.set(&format!("Wrote {}", filename));
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

pub fn next_screen(context: &mut Context, term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let window = &context.window;
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    let target = window.scroll_line.get() + offset;
    if target < buffer.lines_count() {
        window.scroll_line.set(target);
        context.cursor.line = target;
        Ok(())
    } else {
        context.buffer_list.minibuffer.set("End of buffer");
        Err(())
    }
}

pub fn previous_screen(context: &mut Context, term: &Term) -> Result {
    let window = &context.window;
    if window.scroll_line.get() == 0 {
        context.buffer_list.minibuffer.set("Beginning of buffer");
        return Err(());
    }
    let offset = window.get_window_lines(term) - 1 - CONTEXT_LINES;
    context.cursor.line = window.scroll_line.get() + CONTEXT_LINES;
    window.scroll_line.set(
        if let Some(scroll_line) = window.scroll_line.get().checked_sub(offset) {
            scroll_line
        } else {
            0
        },
    );
    Ok(())
}

pub fn beginning_of_buffer(context: &mut Context, _term: &Term) -> Result {
    context.cursor.line = 0;
    context.cursor.column = 0;
    Ok(())
}

pub fn end_of_buffer(context: &mut Context, _term: &Term) -> Result {
    let buffer = context.buffer_list.get_current_buffer();
    let linenum = buffer.lines_count() - 1;
    context.cursor.line = linenum;
    let line = buffer.get_line_unchecked(linenum);
    context.cursor.column = line.len();
    Ok(())
}

pub fn kill_emacs(context: &mut Context, _term: &Term) -> Result {
    context.to_exit = true;
    Ok(())
}
