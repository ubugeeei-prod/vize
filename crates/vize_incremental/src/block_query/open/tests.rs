#![expect(
    clippy::unwrap_used,
    reason = "the directed document and selected stages are valid"
)]

use super::*;
use crate::{DescriptorStats, ResidentDocuments, StageConfig, compute_file_artifacts};

const FIRST: &str =
    "<template><p>{{ total }}</p></template><style>.a { color: v-bind(tone) }</style>";
const SHIFTED: &str = "<script>const padding = '雪'</script><template><p>{{ total }}</p></template><style>.a { color: v-bind(tone) }</style>";

#[test]
fn blocks_and_existing_descriptor_requests_share_one_parse_per_revision() {
    let mut docs = ResidentDocuments::default();
    for source in [FIRST, SHIFTED] {
        let expected = compute_file_artifacts(source, StageConfig::default());
        let original = docs.parsed("editor", "component.vue", source);
        docs.with_document_blocks("editor", "component.vue", source, |view| {
            let named = view.unwrap();
            let ParsedSfc::Descriptor(parsed) = &original else {
                unreachable!()
            };
            assert!(core::ptr::eq(&**named.descriptor, &**parsed));
            let view = named.blocks;
            assert_eq!(view.iter().len(), expected.len());
            for (block, old) in view.iter().zip(&expected) {
                assert_eq!(block.source().key, old.source_key);
                assert_eq!(block.surface(), old.surface.as_ref());
                assert_eq!(block.page(), old.page.as_ref());
            }
        });
        assert_eq!(docs.parsed("editor", "component.vue", source), original);
        assert_eq!(
            docs.take_stats(),
            DescriptorStats {
                lookups: 3,
                parses: 1
            }
        );
    }
}

#[test]
fn unchanged_stage_artifacts_survive_a_move_and_close_releases_their_identities() {
    let mut docs = ResidentDocuments::default();
    let stages = |docs: &mut ResidentDocuments, source: &str| {
        docs.with_document_blocks("editor", "component.vue", source, |view| {
            let view = view.unwrap().blocks;
            let block = view
                .at(source.find("<p>").unwrap() as u32)
                .unwrap()
                .unwrap();
            let start = block.start();
            let surface = block.surface().unwrap().key;
            let page = block.page().unwrap().key;
            let accounting = view.db.take_accounting();
            let calls = |name| accounting.get(name).map_or(0, |v| v.executed);
            (
                (surface, page),
                start,
                (calls("s1_block"), calls("s2_page")),
            )
        })
    };
    let (keys, first, cold) = stages(&mut docs, FIRST);
    let (same_keys, shifted, reused) = stages(&mut docs, SHIFTED);
    assert_eq!(cold, (1, 1));
    assert_eq!(same_keys, keys);
    assert_eq!(shifted - first, (SHIFTED.len() - FIRST.len()) as u32);
    assert_eq!(reused, (0, 0));
    docs.close("editor");
    let (fresh_keys, reopened, compiled) = stages(&mut docs, SHIFTED);
    assert_eq!(
        (fresh_keys, reopened, compiled),
        (same_keys, shifted, (1, 1))
    );
}
