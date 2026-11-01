//! Retain authored bytes after parsing without copying an owned read buffer.

use std::borrow::Cow;

use vize_carton::{String as CompactString, ToCompactString};

pub(in crate::batch::virtual_project) fn retain_source_content(
    content: Cow<'_, str>,
) -> CompactString {
    match content {
        // Editor callers lend their unsaved text; retain the existing copy.
        Cow::Borrowed(content) => content.to_compact_string(),
        // compact_str's From<String> reuses long buffers and inlines short ones.
        Cow::Owned(content) => CompactString::from(content),
    }
}
