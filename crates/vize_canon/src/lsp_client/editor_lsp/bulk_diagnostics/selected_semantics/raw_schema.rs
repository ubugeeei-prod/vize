//! Borrowed validation of omitted rows against the original diagnostic schema.

use serde::{
    Deserialize, Deserializer,
    de::{SeqAccess, Visitor},
};
use serde_json::Value;
use std::fmt;

// Keep the original field types, order, defaults and unknown-field policy.
// JSON Value supplies borrowed strings; omitted rows never need owned text.
#[derive(Deserialize)]
pub(super) struct Diagnostic<'a> {
    #[serde(default, rename = "fileName")]
    _file_name: &'a str,
    #[serde(rename = "pos")]
    _pos: u32,
    #[serde(rename = "end")]
    _end: u32,
    #[serde(rename = "code")]
    _code: i32,
    #[serde(rename = "category")]
    _category: u8,
    #[serde(rename = "text")]
    _text: &'a str,
    #[serde(default, rename = "messageChain")]
    _message_chain: Children,
    #[serde(default, rename = "relatedInformation")]
    _related_information: Children,
}

#[derive(Default)]
struct Children;

impl<'de> Deserialize<'de> for Children {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ChildVisitor;

        impl<'de> Visitor<'de> for ChildVisitor {
            type Value = Children;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a sequence")
            }

            fn visit_seq<A: SeqAccess<'de>>(self, mut seq: A) -> Result<Self::Value, A::Error> {
                while seq.next_element::<Diagnostic<'de>>()?.is_some() {}
                Ok(Children)
            }
        }

        deserializer.deserialize_seq(ChildVisitor)
    }
}

pub(super) fn main_name(row: &Value) -> Option<&str> {
    // serde_json's original derived struct decoder accepts maps and positional
    // sequences. Both identify the main filename without consuming the row.
    match row {
        Value::Object(row) => row.get("fileName")?.as_str(),
        Value::Array(row) => row.first()?.as_str(),
        _ => None,
    }
}
