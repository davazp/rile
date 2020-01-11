use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::buffer_list::{BufferList, BufferRef};
use crate::event_loop::EventLoopState;
use crate::window_list::WindowList;
use crate::{Buffer, Window};

pub struct GoalColumn {
    pub column: Option<usize>,
    pub to_preserve: bool,
}

/// The state of the editor.
pub struct Context {
    pub buffer_list: BufferList,
    pub window_list: WindowList,
    pub event_loop: EventLoopState,
    pub was_resized: Arc<AtomicBool>,
    pub goal_column: GoalColumn,
}

impl Context {
    pub fn new(buffer: Buffer) -> Context {
        Context {
            buffer_list: BufferList::new(buffer),

            window_list: WindowList {
                main: Window::new(BufferRef::main_window(), true),
                minibuffer: Window::new(BufferRef::minibuffer_window(), false),
                minibuffer_focused: false,
            },

            was_resized: Arc::new(AtomicBool::new(false)),

            event_loop: EventLoopState::new(),

            goal_column: GoalColumn {
                to_preserve: false,
                column: None,
            },
        }
    }
}
