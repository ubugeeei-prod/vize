//! Retained document text that can be the native payload allocation itself.
//!
//! A materialized leaf's `(uri, code)` payload reaches the owner's document
//! map and the editor mirror as one shared allocation; buffers that arrive
//! as plain text own a fresh one.
#![expect(
    clippy::disallowed_types,
    reason = "immutable payload bytes are shared with materialized leaves"
)]

use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use serde::{Serialize, Serializer};
use vize_l0::String;

#[derive(Clone)]
pub(crate) struct RetainedText(Arc<(String, String)>);

impl RetainedText {
    /// Share a native `(uri, text)` payload without copying its text.
    pub(crate) fn shared(payload: &Arc<(String, String)>) -> Self {
        Self(Arc::clone(payload))
    }

    pub(crate) fn as_str(&self) -> &str {
        self.0.1.as_str()
    }

    #[cfg(test)]
    pub(crate) fn shares(&self, payload: &Arc<(String, String)>) -> bool {
        Arc::ptr_eq(&self.0, payload)
    }
}

impl From<String> for RetainedText {
    fn from(text: String) -> Self {
        Self(Arc::new((String::default(), text)))
    }
}

impl From<&RetainedText> for RetainedText {
    fn from(text: &RetainedText) -> Self {
        text.clone()
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_types,
    reason = "tests build fixtures from std strings"
)]
impl From<std::string::String> for RetainedText {
    fn from(text: std::string::String) -> Self {
        Self::from(text.as_str())
    }
}

#[cfg(test)]
#[expect(
    clippy::disallowed_types,
    reason = "tests build fixtures from std strings"
)]
impl From<&std::string::String> for RetainedText {
    fn from(text: &std::string::String) -> Self {
        Self::from(text.as_str())
    }
}

impl From<&str> for RetainedText {
    fn from(text: &str) -> Self {
        Self::from(String::from(text))
    }
}

impl From<&String> for RetainedText {
    fn from(text: &String) -> Self {
        Self::from(text.as_str())
    }
}

impl Deref for RetainedText {
    type Target = str;

    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for RetainedText {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl PartialEq for RetainedText {
    fn eq(&self, other: &Self) -> bool {
        Arc::ptr_eq(&self.0, &other.0) || self.as_str() == other.as_str()
    }
}

impl Eq for RetainedText {}

impl PartialEq<str> for RetainedText {
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialEq<&str> for RetainedText {
    fn eq(&self, other: &&str) -> bool {
        self.as_str() == *other
    }
}

impl PartialEq<String> for RetainedText {
    fn eq(&self, other: &String) -> bool {
        self.as_str() == other.as_str()
    }
}

impl PartialEq<RetainedText> for str {
    fn eq(&self, other: &RetainedText) -> bool {
        self == other.as_str()
    }
}

impl PartialEq<RetainedText> for &str {
    fn eq(&self, other: &RetainedText) -> bool {
        *self == other.as_str()
    }
}

impl fmt::Debug for RetainedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), formatter)
    }
}

impl fmt::Display for RetainedText {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}

impl Serialize for RetainedText {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}

/// Retained document texts by URI.
pub(crate) type DocumentTexts = vize_l0::FxHashMap<String, RetainedText>;
