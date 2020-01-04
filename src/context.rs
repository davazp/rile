use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::buffer_list::BufferList;

use crate::event_loop;
use crate::Window;

pub struct GoalColumn {
    pub column: Option<usize>,
    pub to_preserve: bool,
}

/// The state of the editor.
pub struct Context {
    pub buffer_list: BufferList,

    pub main_window: Window,
    pub minibuffer_window: Window,

    pub event_loop: event_loop::EventLoopState,

    pub was_resized: Arc<AtomicBool>,

    pub goal_column: GoalColumn,
}
