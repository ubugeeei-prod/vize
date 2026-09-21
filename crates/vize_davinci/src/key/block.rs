//! S0 source-block keys: the content identity of one SFC block before any
//! stage has parsed it — the key a resident firewall compares first.
//!
//! A block is its kind (`template`, `script`, `style`, or a custom block
//! type), its header attributes and its content bytes. Header attributes are
//! a **set**: `<script setup lang="ts">` and `<script lang="ts" setup>` are
//! the same block, so the attributes are fed in sorted order (identity
//! excludes presentation). The block's position is not an input at all — it
//! lives in the S0 side table beside the key.

use super::{ArtifactKey, KeySink, schema};
use crate::stage::Stage;

/// Presence markers for an attribute's value (`setup` vs `lang="ts"`).
const VALUE_ABSENT: u8 = 0;
const VALUE_PRESENT: u8 = 1;

type Attr<'a> = (&'a str, Option<&'a str>);

/// Key one SFC source block by kind, header attributes and content.
///
/// `attrs` are `(name, value)` pairs in any order (`None` for a bare
/// attribute); duplicates are kept, so a malformed header with a repeated
/// attribute keys differently from the deduplicated one. Headers carry a
/// handful of attributes, so the sorted order is produced by selection over
/// the borrowed slice — no allocation on the keying path.
#[must_use]
pub fn source_block_key(kind: &str, attrs: &[Attr<'_>], content: &str) -> ArtifactKey {
    let mut sink = KeySink::new(Stage::Source, schema::SOURCE_BLOCK, 0);
    sink.feed_str(kind);
    sink.feed_u32(attrs.len() as u32);
    let mut previous: Option<Attr<'_>> = None;
    while let Some(next) = attrs
        .iter()
        .copied()
        .filter(|attr| previous.is_none_or(|previous| *attr > previous))
        .min()
    {
        for _ in attrs.iter().filter(|attr| **attr == next) {
            feed_attr(&mut sink, next);
        }
        previous = Some(next);
    }
    sink.feed_str(content);
    sink.finish()
}

fn feed_attr(sink: &mut KeySink, (name, value): Attr<'_>) {
    sink.feed_str(name);
    match value {
        Some(value) => {
            sink.feed_tag(VALUE_PRESENT);
            sink.feed_str(value);
        }
        None => sink.feed_tag(VALUE_ABSENT),
    }
}
