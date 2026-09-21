//! S0 source-block keys: the content identity of one SFC block before any
//! stage has parsed it — the key a resident firewall compares first.
//!
//! A block is its kind (`template`, `script`, `style`, or a custom block
//! type), its header attributes and its content bytes. Header attributes are
//! a **set**: `<script setup lang="ts">` and `<script lang="ts" setup>` are
//! the same block, so the attributes are sorted before they are fed
//! (identity excludes presentation). The block's position is not an input
//! at all — it lives in the S0 side table beside the key.

use alloc::vec::Vec;

use super::{ArtifactKey, KeySink, schema};
use crate::stage::Stage;

/// Presence markers for an attribute's value (`setup` vs `lang="ts"`).
const VALUE_ABSENT: u8 = 0;
const VALUE_PRESENT: u8 = 1;

/// Key one SFC source block by kind, header attributes and content.
///
/// `attrs` are `(name, value)` pairs in any order (`None` for a bare
/// attribute); duplicates are kept, so a malformed header with a repeated
/// attribute keys differently from the deduplicated one.
#[must_use]
pub fn source_block_key(kind: &str, attrs: &[(&str, Option<&str>)], content: &str) -> ArtifactKey {
    let mut sorted: Vec<(&str, Option<&str>)> = attrs.to_vec();
    sorted.sort_unstable();

    let mut sink = KeySink::new(Stage::Source, schema::SOURCE_BLOCK, 0);
    sink.feed_str(kind);
    sink.feed_u32(sorted.len() as u32);
    for (name, value) in sorted {
        sink.feed_str(name);
        match value {
            Some(value) => {
                sink.feed_tag(VALUE_PRESENT);
                sink.feed_str(value);
            }
            None => sink.feed_tag(VALUE_ABSENT),
        }
    }
    sink.feed_str(content);
    sink.finish()
}
