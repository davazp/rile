use crate::term;
use crate::Context;

#[derive(Clone)]
pub struct Region {
    pub top: usize,
    pub height: usize,
}

pub struct Layout {
    main_window_region: Region,
    minibuffer_region: Region,
}

pub fn get_layout(term: &term::Term, context: &Context) -> Layout {
    let minibuffer_height = context.buffer_list.minibuffer.lines_count();

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
    if context.buffer_list.minibuffer_focused {
        layout.minibuffer_region
    } else {
        layout.main_window_region
    }
}
