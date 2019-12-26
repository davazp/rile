use std::char;

/// A key press.
#[derive(PartialEq, Debug)]
pub struct Key {
    // `meta` is true if the meta modified key (usually alt) is active
    // during this key press as well.
    //
    // Note tht we do not have a field for `ctrl`. Instead, this is
    // encoded directly in the `code` field.
    meta: bool,
    code: u32,
}

impl Key {
    /// Parse a single key name without any modifiers.
    fn parse_unmodified(key: &str) -> Option<Key> {
        if key.len() == 1 {
            Some(Key::from_code(key.chars().next().unwrap() as u32))
        } else {
            match key {
                "DEL" => Some(Key::from_code(127)),
                "RET" => Some(Key::from_code(13)),
                "TAB" => Some(Key::from_code(9)),
                _ => None,
            }
        }
    }

    /// Parse a key description with possible modifiers.
    ///
    /// ## Examples
    ///
    /// - `C-a`
    /// - `M-f`
    /// - `C-M-x`
    pub fn parse(key: &str) -> Option<Key> {
        if let Some(suffix) = starts_with("C-M-", key) {
            Some(Key::parse_unmodified(suffix)?.ctrl().meta())
        } else if let Some(suffix) = starts_with("C-", key) {
            Some(Key::parse_unmodified(suffix)?.ctrl())
        } else if let Some(suffix) = starts_with("M-", key) {
            Some(Key::parse_unmodified(suffix)?.meta())
        } else {
            Key::parse_unmodified(key)
        }
    }

    /// Parse a key press and panics in case of an error.
    pub fn parse_unchecked(key: &str) -> Key {
        Key::parse(key).unwrap()
    }

    /// Create a key from a terminal code.
    pub fn from_code(code: u32) -> Key {
        Key { code, meta: false }
    }

    /// Modify a key to add the meta modifier.
    pub fn meta(mut self) -> Key {
        self.meta = true;
        self
    }

    /// Modify a key to add the ctrl modifier.
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

/// Check if `str` starts with `prefix` and strip it.
fn starts_with<'a>(prefix: &str, str: &'a str) -> Option<&'a str> {
    if str.starts_with(prefix) {
        Some(&str[prefix.len()..])
    } else {
        None
    }
}
