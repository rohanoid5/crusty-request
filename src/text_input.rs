use serde::{Deserialize, Serialize, Serializer, Deserializer};
use std::ops::Deref;
use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct TextInput {
    pub text: String,
    pub cursor: usize,
}

impl Default for TextInput {
    fn default() -> Self {
        Self::new()
    }
}

impl TextInput {
    pub fn new() -> Self {
        Self {
            text: String::new(),
            cursor: 0,
        }
    }

    pub fn from(text: String) -> Self {
        let len = text.len();
        Self {
            text,
            cursor: len,
        }
    }

    pub fn insert_char(&mut self, c: char) {
        self.text.insert(self.cursor, c);
        self.cursor += c.len_utf8();
    }

    pub fn delete_back(&mut self) {
        if self.cursor > 0 {
            // Find start of previous char
            let mut new_cursor = self.cursor - 1;
            while !self.text.is_char_boundary(new_cursor) {
                new_cursor -= 1;
            }
            self.text.remove(new_cursor);
            self.cursor = new_cursor;
        }
    }

    pub fn delete_forward(&mut self) {
        if self.cursor < self.text.len() {
            self.text.remove(self.cursor);
        }
    }

    pub fn move_left(&mut self) {
        if self.cursor > 0 {
            let mut new_cursor = self.cursor - 1;
            while !self.text.is_char_boundary(new_cursor) {
                new_cursor -= 1;
            }
            self.cursor = new_cursor;
        }
    }

    pub fn move_right(&mut self) {
        if self.cursor < self.text.len() {
            let mut new_cursor = self.cursor + 1;
            while new_cursor < self.text.len() && !self.text.is_char_boundary(new_cursor) {
                new_cursor += 1;
            }
            self.cursor = new_cursor;
        }
    }

    pub fn as_str(&self) -> &str {
        &self.text
    }

    pub fn is_empty(&self) -> bool {
        self.text.is_empty()
    }

    pub fn render_with_cursor(&self, is_editing: bool) -> String {
        if !is_editing {
            return self.text.clone();
        }
        let mut s = self.text.clone();
        if self.cursor <= self.text.len() && self.text.is_char_boundary(self.cursor) {
            s.insert(self.cursor, '▏');
        } else {
            s.push('▏');
        }
        s
    }

    pub fn clear(&mut self) {
        self.text.clear();
        self.cursor = 0;
    }
}

impl Serialize for TextInput {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.text)
    }
}

impl<'de> Deserialize<'de> for TextInput {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(TextInput::from(s))
    }
}

impl Deref for TextInput {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.text
    }
}

impl fmt::Display for TextInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.text)
    }
}
