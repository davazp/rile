use crate::buffer::Buffer;
use crate::commands::Result;
use crate::context::Context;
use crate::keymap::Keymap;
use crate::term::Term;

pub fn minibuffer_complete(context: &mut Context, _term: &mut Term) -> Result {
    context.event_loop.complete(Ok(()));
    Ok(())
}

pub fn new() -> Buffer {
    let mut minibuffer = Buffer::new();

    minibuffer.keymap = Keymap::defaults();
    minibuffer.keymap.define_key("RET", minibuffer_complete);

    minibuffer
}
