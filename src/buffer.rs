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
}
