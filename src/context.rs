use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::buffer::BufferList;
use crate::event_loop;
use crate::window::Window;

/// A cursor into a buffer content
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

pub struct GoalColumn {
    pub column: Option<usize>,
    pub to_preserve: bool,
}

/// The state of the editor.
pub struct Context {
    pub cursor: Cursor,
    pub buffer_list: BufferList,

    pub window: Window,

    pub event_loop: event_loop::EventLoopState,

    pub was_resized: Arc<AtomicBool>,

    pub goal_column: GoalColumn,
}
