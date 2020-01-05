use std::fs;

use crate::Keymap;

/// A cursor into a buffer content
pub struct Cursor {
    pub line: usize,
    pub column: usize,
}

impl Cursor {
    fn new() -> Cursor {
        Cursor { line: 0, column: 0 }
    }
}

/// A buffer contains text that can be edited.
pub struct Buffer {
    pub keymap: Keymap,
    pub filename: Option<String>,

    /// Substrings to highlight in the buffer.
    pub highlight: Option<String>,

    /// The cursor should always be a valid reference to the buffer.
    pub cursor: Cursor,

    /// All lines of this buffer.
    lines: Vec<String>,
}

impl Buffer {
    pub fn new() -> Buffer {
        Buffer {
            cursor: Cursor::new(),
            lines: vec!["".to_string()],
            filename: None,
            highlight: None,
            keymap: Keymap::defaults(),
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

    pub fn set<T: AsRef<str>>(&mut self, str: T) {
        // Note that we can't use .lines() here because it would
        // ignore trailing new lines.
        //
        // .split() on the other hand will always be non-empty and it
        // will allow us to recover the original content by adding a
        // \n between each line.
        self.lines = str.as_ref().split('\n').map(String::from).collect();
        self.cursor.line = 0;
        self.cursor.column = 0;
    }

    pub fn truncate(&mut self) {
        self.lines.clear();
        self.lines.push("".to_string());
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
