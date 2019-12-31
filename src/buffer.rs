use std::fs;

/// A buffer contains text that can be edited.
pub struct Buffer {
    pub filename: Option<String>,
    /// All lines of this buffer.
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            lines: Vec::new(),
            filename: None,
        }
    }
    pub fn from_string(str: &str) -> Buffer {
        let mut buffer = Buffer::new();
        buffer.set(str);
        buffer
    }

    pub fn from_file(file: &str) -> Buffer {
        let content = match fs::read_to_string(&file) {
            Ok(content) => content,
            Err(_) => String::from(""),
        };
        let mut buffer = Buffer::from_string(&content);
        buffer.filename = Some(file.to_string());
        buffer
    }

    pub fn get_line(&self, nth: usize) -> Option<&str> {
        self.lines.get(nth).map(|s| &s[..])
    }

    pub fn get_line_unchecked(&self, nth: usize) -> &str {
        &self.lines[nth]
    }

    pub fn get_line_mut_unchecked(&mut self, nth: usize) -> &mut String {
        &mut self.lines[nth]
    }

    pub fn lines_count(&self) -> usize {
        self.lines.len()
    }

    pub fn insert_line_at(&mut self, nth: usize, line: String) {
        self.lines.insert(nth, line);
    }

    pub fn remove_line(&mut self, nth: usize) -> String {
        self.lines.remove(nth)
    }

    pub fn remove_char_at(&mut self, line: usize, column: usize) {
        self.lines[line].remove(column);
    }

    pub fn set(&mut self, str: &str) {
        // Note that we can't use .lines() here because it would
        // ignore trailing new lines.
        //
        // .split() on the other hand will always be non-empty and it
        // will allow us to recover the original content by adding a
        // \n between each line.
        self.lines = str.split('\n').map(String::from).collect();
    }

    pub fn truncate(&mut self) {
        self.lines.clear();
    }

    pub fn to_string(&self) -> String {
        self.lines.join("\n")
    }

    pub fn save(&self) -> Result<String, SaveError> {
        let contents = self.to_string();
        if let Some(filename) = &self.filename {
            fs::write(filename, contents)
                .map(|_| filename.clone())
                .map_err(SaveError::IoError)
        } else {
            Err(SaveError::NoFile)
        }
    }
}

pub enum SaveError {
    NoFile,
    IoError(std::io::Error),
}

pub struct BufferList {
    minibuffer_focused: bool,
    main_buffer: Buffer,
    pub minibuffer: Buffer,
}

impl BufferList {
    pub fn new(main: Buffer) -> BufferList {
        BufferList {
            minibuffer_focused: false,
            main_buffer: main,
            minibuffer: Buffer::new(),
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
