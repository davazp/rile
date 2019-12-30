use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use crate::buffer::Buffer;
use crate::keymap::Keymap;
use crate::window::Window;

/// A cursor into a buffer content
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

/// The state of the editor.
pub struct Context {
    /// The column that a following [`next-line`](fn.next_line.html) or
    /// [`previous_line`](fn.previous_line.html) should try to move
    /// to. This is automatically reset to `None` after each user
    /// command is processed, unless
    /// [`to_preserve_goal_column`](#structfield.to_preserve_goal_column)
    /// is set to true by the command.
    pub goal_column: Option<usize>,

    pub cursor: Cursor,
    pub current_buffer: Buffer,

    pub minibuffer: Buffer,

    pub window: Window,
    pub keymap: Keymap,

    // Result of a command. They will take effect once a full command
    // has been processed.
    pub to_exit: bool,

    pub was_resized: Arc<AtomicBool>,

    /// If set by a command, [`goal_column`](#structfield.goal_column) won't be reset after it.
    pub to_preserve_goal_column: bool,
}
