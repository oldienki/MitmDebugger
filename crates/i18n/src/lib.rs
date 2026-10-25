use std::ops::Deref;

#[derive(Copy, Clone)]
pub struct Text {
    text: &'static str,
    id: Option<&'static str>,
}

impl Text {
    pub fn as_str(&self) -> &'static str {
        self.text
    }

    /// Optional lookup id for future translation tables.
    pub fn id(&self) -> Option<&'static str> {
        self.id
    }
}

pub const fn new(text: &'static str) -> Text {
    Text { text, id: None }
}

pub const fn with_id(text: &'static str, id: &'static str) -> Text {
    Text { text, id: Some(id) }
}

impl Deref for Text {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        self.as_str()
    }
}

impl AsRef<str> for Text {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl From<Text> for String {
    fn from(text: Text) -> Self {
        text.as_str().to_string()
    }
}

impl std::fmt::Display for Text {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[macro_export]
macro_rules! i18n {
    ($text:expr) => {
        $crate::new($text)
    };
    ($text:expr, $id:expr) => {
        $crate::with_id($text, $id)
    };
}
