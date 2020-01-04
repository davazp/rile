use crate::Window;

pub struct WindowList {
    pub minibuffer_focused: bool,
    pub main: Window,
    pub minibuffer: Window,
}

impl WindowList {
    pub fn get_current_window(&self) -> &Window {
        if self.minibuffer_focused {
            &self.minibuffer
        } else {
            &self.main
        }
    }

    pub fn get_current_window_as_mut(&mut self) -> &mut Window {
        if self.minibuffer_focused {
            &mut self.minibuffer
        } else {
            &mut self.main
        }
    }
}
