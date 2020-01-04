use crate::minibuffer;
use crate::Buffer;

#[derive(Copy, Clone)]
pub struct BufferRef(u64);

impl BufferRef {
    pub fn main_window() -> BufferRef {
        BufferRef(0)
    }
    pub fn minibuffer_window() -> BufferRef {
        BufferRef(1)
    }
}

pub struct BufferList {
    main_buffer: Buffer,
    pub minibuffer: Buffer,
}

impl BufferList {
    pub fn new(main: Buffer) -> BufferList {
        BufferList {
            main_buffer: main,
            minibuffer: minibuffer::new(),
        }
    }

    pub fn resolve_ref(&self, buffer_ref: BufferRef) -> &Buffer {
        if buffer_ref.0 == 0 {
            &self.main_buffer
        } else if buffer_ref.0 == 1 {
            &self.minibuffer
        } else {
            panic!("Can't resolve a buffer that does not exist anymore.")
        }
    }

    pub fn resolve_ref_as_mut(&mut self, buffer_ref: BufferRef) -> &mut Buffer {
        if buffer_ref.0 == 0 {
            &mut self.main_buffer
        } else if buffer_ref.0 == 1 {
            &mut self.minibuffer
        } else {
            panic!("Can't resolve a buffer that does not exist anymore.")
        }
    }

    pub fn get_main_buffer(&self) -> &Buffer {
        &self.main_buffer
    }
}
