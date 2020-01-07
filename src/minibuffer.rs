use crate::term::Term;
use crate::{commands, Buffer, Context, Keymap};

pub fn minibuffer_complete(context: &mut Context, _term: &mut Term) -> commands::Result {
    context.event_loop.complete(Ok(()));
    Ok(())
}

pub fn new() -> Buffer {
    let mut minibuffer = Buffer::new();
    let mut keymap = Keymap::new();
    keymap.define_key("RET", minibuffer_complete);
    keymap.define_key("C-a", commands::beginning_of_buffer);
    keymap.define_key("C-e", commands::end_of_buffer);
    keymap.define_key("C-g", commands::keyboard_quit);
    keymap.define_key("DEL", commands::delete_backward_char);

    minibuffer.keymap = keymap;
    minibuffer
}
