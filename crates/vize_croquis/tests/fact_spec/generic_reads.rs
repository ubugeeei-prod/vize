use super::runner::{Planes, run_source};

#[test]
fn authored_generic_type_reads_match_independently_combined_semantics() {
    let mut planes = Planes::default();
    for source in [
        "<script setup lang=\"ts\" generic=\"T extends Kind.A\">import { Kind } from './kind';</script>",
        "<script setup lang=\"ts\" generic=\"T extends typeof Kind\">import { Kind } from './kind';</script>",
        "<script setup lang=\"ts\" generic=\"Kind, T extends Kind\">import { Kind } from './kind';</script>",
        "<script setup lang=\"ts\" generic=\"T extends { Kind: string }\">import { Kind } from './kind';</script>",
    ] {
        run_source("authored-generic-read", source, &mut planes);
    }
    planes.assert_verdicts("authored-generic-read");
    if cfg!(debug_assertions) {
        planes
            .unused
            .verdict("unused-bindings", "authored-generic-read")
            .unwrap();
        assert_eq!(
            (
                planes.unused.artifacts,
                planes.unused.compared,
                planes.unused.facts
            ),
            (4, 4, 2)
        );
    }
}
