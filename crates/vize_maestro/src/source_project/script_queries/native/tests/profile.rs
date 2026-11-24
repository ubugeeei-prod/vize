use super::*;
use davinci_harness::{alloc, paired::assert_paired_time};
use std::hint::black_box;

#[test]
#[ignore = "measures editor reads with a resident native File and with its construction"]
fn original_lowering_and_script_products_across_edits() {
    alloc::mark_installed();
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Profile.vue").unwrap();
    for (name, source) in [
        (
            "declarations",
            "<script lang='ts'>export class Pair { leaf() { return import('./leaf') } }</script><script setup lang='ts'>import type { Pair } from './types'; const value = { selected: 1 };</script>",
        ),
        (
            "broken-sibling",
            "<script>const invalid = ;</script><script setup>import Leaf from './Leaf'; const menu = { open() { return require('./menu') } };</script>",
        ),
        ("empty", "<script></script><template><i/></template>"),
        ("template-only", "<template><span>🌸</span></template>"),
        (
            "tsx",
            "<script setup lang='tsx'>import { A } from './A'; const view = <A/>;</script>",
        ),
        (
            "unsupported",
            "<custom>original</custom><script>const answer = import('./answer');</script>",
        ),
    ] {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let snapshot = SourceSnapshotCache::default()
            .capture(&state.documents, &uri)
            .unwrap();
        let arena = Allocator::default();
        let owner = lower_sfc_native(&arena, snapshot.source(), options());
        let read = |shared, native: &NativeSfcObservation<'_>| {
            if shared {
                black_box(&snapshot)
                    .script_query_products_from(black_box(&state), black_box(native))
                    .unwrap()
            } else {
                black_box(&snapshot)
                    .script_query_products(black_box(&state))
                    .unwrap()
            }
        };
        assert_eq!(read(false, &owner), read(true, &owner));
        for cold in [false, true] {
            let run = |shared| {
                if cold {
                    let arena = Allocator::default();
                    let native = lower_sfc_native(&arena, black_box(snapshot.source()), options());
                    black_box(read(shared, black_box(&native)));
                } else {
                    black_box(read(shared, &owner));
                }
            };
            let old = alloc::measure(|| run(false)).unwrap();
            let new = alloc::measure(|| run(true)).unwrap();
            eprintln!(
                "retained_script_heap {name} cold={cold} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
            );
            assert_paired_time(
                &vize_l0::cstr!("retained script {name} cold={cold}"),
                64,
                run,
            );
        }
    }
}
