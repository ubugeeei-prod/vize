//! Pairing an authored SFC's blocks with the formatted document's blocks.
//!
//! Both `rangeFormatting` and `onTypeFormatting` answer by formatting the whole
//! document once and then projecting the result back onto the part of the file
//! the request is about. That projection needs the same blocks on both sides,
//! which is what this module provides: content spans in one fixed discovery
//! order, so `authored[i]` and `formatted[i]` are the same block.

/// Stable identity and byte span for one format-supported SFC block.
///
/// The descriptor's discovery order is independent of authored block order,
/// but an explicit identity still matters: a formatter regression that turns
/// one block kind into another must be refused instead of projecting the
/// formatted bytes through an unrelated authored range.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct BlockSpan {
    pub(super) identity: BlockIdentity,
    pub(super) start: usize,
    pub(super) end: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BlockIdentity {
    Template,
    ScriptSetup,
    Script,
    Style(usize),
    Art(usize),
}

pub(super) fn block_spans(source: &str, filename: &str) -> Option<Vec<BlockSpan>> {
    // The formatted buffer is not the open document, so this must not write
    // the resident document cache. `parse_descriptor` is that parse.
    let descriptor = vize_incremental::parse_descriptor(filename, source)?;

    Some(
        descriptor
            .template
            .as_ref()
            .map(|block| BlockSpan {
                identity: BlockIdentity::Template,
                start: block.loc.start,
                end: block.loc.end,
            })
            .into_iter()
            .chain(descriptor.script_setup.as_ref().map(|block| BlockSpan {
                identity: BlockIdentity::ScriptSetup,
                start: block.loc.start,
                end: block.loc.end,
            }))
            .chain(descriptor.script.as_ref().map(|block| BlockSpan {
                identity: BlockIdentity::Script,
                start: block.loc.start,
                end: block.loc.end,
            }))
            .chain(
                descriptor
                    .styles
                    .iter()
                    .enumerate()
                    .map(|(index, block)| BlockSpan {
                        identity: BlockIdentity::Style(index),
                        start: block.loc.start,
                        end: block.loc.end,
                    }),
            )
            .chain(
                descriptor
                    .custom_blocks
                    .iter()
                    // Glyph formats Musea examples through its template formatter.
                    // Other custom blocks have no supported embedded formatter.
                    .filter(|block| block.block_type.as_ref() == "art")
                    .enumerate()
                    .map(|(index, block)| BlockSpan {
                        identity: BlockIdentity::Art(index),
                        start: block.loc.start,
                        end: block.loc.end,
                    }),
            )
            .collect(),
    )
}

/// Both documents must expose the same supported blocks in the same semantic
/// discovery order before their source ranges can be paired.
pub(super) fn same_block_sequence(authored: &[BlockSpan], target: &[BlockSpan]) -> bool {
    authored.len() == target.len()
        && authored
            .iter()
            .zip(target)
            .all(|(left, right)| left.identity == right.identity)
}

#[cfg(test)]
mod tests {
    use super::{BlockIdentity, BlockSpan, block_spans, same_block_sequence};

    #[test]
    fn semantic_discovery_keeps_block_identity_across_profile_reordering() {
        let source = concat!(
            "<art><One   /></art>\n",
            "<style>.first{color:red}</style>\n",
            "<template><main   /></template>\n",
            "<script>const   ordinary=1</script>\n",
            "<style>.second{color:blue}</style>\n",
            "<script setup>const   setup=2</script>\n",
            "<art><Two   /></art>\n",
        );
        let formatted = vize_glyph::format_sfc(source, &vize_glyph::FormatOptions::default())
            .expect("format")
            .code;
        assert_ne!(
            formatted.as_str(),
            source,
            "fixture must exercise layout and reordering"
        );

        let authored = block_spans(source, "/Identity.vue").expect("authored descriptor");
        let target = block_spans(&formatted, "/Identity.vue").expect("formatted descriptor");
        assert_eq!(
            authored
                .iter()
                .map(|block| block.identity)
                .collect::<Vec<_>>(),
            [
                BlockIdentity::Template,
                BlockIdentity::ScriptSetup,
                BlockIdentity::Script,
                BlockIdentity::Style(0),
                BlockIdentity::Style(1),
                BlockIdentity::Art(0),
                BlockIdentity::Art(1),
            ]
        );
        assert!(same_block_sequence(&authored, &target));
    }

    #[test]
    fn a_different_block_identity_is_not_a_valid_source_range_pair() {
        let span = |identity| BlockSpan {
            identity,
            start: 1,
            end: 2,
        };
        let authored = [span(BlockIdentity::Template), span(BlockIdentity::Script)];
        let target = [
            span(BlockIdentity::Template),
            span(BlockIdentity::ScriptSetup),
        ];

        assert!(!same_block_sequence(&authored, &target));
        assert!(!same_block_sequence(&authored, &target[..1]));
    }
}
