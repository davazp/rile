use crate::term;
use crate::Context;

use std::cmp;

#[derive(Clone)]
pub struct Region {
    pub top: usize,
    pub height: usize,
}

pub struct Layout {
    pub main_window_region: Region,
    pub minibuffer_region: Region,
}

pub fn get_layout(term: &term::Term, context: &Context) -> Layout {
    let minibuffer_height = cmp::min(context.buffer_list.minibuffer.lines_count(), term.rows / 3);

    let minibuffer_region = Region {
        top: term.rows - minibuffer_height,
        height: minibuffer_height,
    };

    let main_window_region = Region {
        top: 0,
        height: term.rows - minibuffer_height,
    };

    Layout {
        main_window_region,
        minibuffer_region,
    }
}

pub fn get_current_window_region(term: &term::Term, context: &Context) -> Region {
    let layout = get_layout(term, context);
    if context.window_list.minibuffer_focused {
        layout.minibuffer_region
    } else {
        layout.main_window_region
    }
}
