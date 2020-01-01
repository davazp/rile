use crate::term::Term;
use crate::{commands, Buffer, Context, Keymap};

pub fn minibuffer_complete(context: &mut Context, _term: &mut Term) -> commands::Result {
    context.event_loop.complete(Ok(()));
    Ok(())
}

pub fn new() -> Buffer {
    let mut minibuffer = Buffer::new();

    minibuffer.keymap = Keymap::defaults();
    minibuffer.keymap.define_key("RET", minibuffer_complete);

    minibuffer
}
