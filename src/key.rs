use std::char;

#[derive(PartialEq, Debug)]
pub struct Key {
    meta: bool,
    code: u32,
}

impl Key {
    fn parse_unmodified(key: &str) -> Option<Key> {
        if key.len() == 1 {
            Some(Key::from_char(key.chars().next().unwrap()))
        } else {
            match key {
                "DEL" => Some(Key::from_code(127)),
                "RET" => Some(Key::from_code(13)),
                "TAB" => Some(Key::from_code(9)),
                _ => None,
            }
        }
    }

    pub fn parse(key: &str) -> Option<Key> {
        if let Some(suffix) = starts_with("C-M-", key) {
            Some(Key::parse_unmodified(suffix)?.ctrl().alt())
        } else if let Some(suffix) = starts_with("C-", key) {
            Some(Key::parse_unmodified(suffix)?.ctrl())
        } else if let Some(suffix) = starts_with("M-", key) {
            Some(Key::parse_unmodified(suffix)?.alt())
        } else {
            Key::parse_unmodified(key)
        }
    }

    pub fn parse_unchecked(key: &str) -> Key {
        Key::parse(key).unwrap()
    }

    pub fn from_code(code: u32) -> Key {
        Key { code, meta: false }
    }

    pub fn from_char(ch: char) -> Key {
        Key::from_code(ch as u32)
    }

    pub fn alt(mut self) -> Key {
        self.meta = true;
        self
    }

    pub fn ctrl(mut self) -> Key {
        self.code = 0x1f & self.code;
        self
    }

    /// Return a character if the key represents a non-control character.
    pub fn as_char(&self) -> Option<char> {
        if self.meta {
            None
        } else {
            char::from_u32(self.code).filter(|ch| !ch.is_control())
        }
    }
}

fn starts_with<'a>(prefix: &str, str: &'a str) -> Option<&'a str> {
    if str.starts_with(prefix) {
        Some(&str[prefix.len()..])
    } else {
        None
    }
}
