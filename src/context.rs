use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::buffer::BufferList;
use crate::event_loop;
use crate::window::Window;

pub struct GoalColumn {
    pub column: Option<usize>,
    pub to_preserve: bool,
}

/// The state of the editor.
pub struct Context {
    pub buffer_list: BufferList,

    pub window: Window,

    pub event_loop: event_loop::EventLoopState,

    pub was_resized: Arc<AtomicBool>,

    pub goal_column: GoalColumn,
}
