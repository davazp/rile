use crate::minibuffer;
use crate::Buffer;

pub struct BufferList {
    pub minibuffer_focused: bool,
    main_buffer: Buffer,
    pub minibuffer: Buffer,
}

impl BufferList {
    pub fn new(main: Buffer) -> BufferList {
        BufferList {
            minibuffer_focused: false,
            main_buffer: main,
            minibuffer: minibuffer::new(),
        }
    }

    pub fn get_current_buffer_as_mut(&mut self) -> &mut Buffer {
        if self.minibuffer_focused {
            &mut self.minibuffer
        } else {
            &mut self.main_buffer
        }
    }

    pub fn get_current_buffer(&self) -> &Buffer {
        if self.minibuffer_focused {
            &self.minibuffer
        } else {
            &self.main_buffer
        }
    }

    pub fn get_main_buffer(&self) -> &Buffer {
        &self.main_buffer
    }
}
