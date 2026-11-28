use super::*;
use davinci_harness::{alloc, paired::assert_calibrated_paired_time};
use std::{fmt::Write, hint::black_box};

#[test]
#[ignore = "serialized paired resources with original lowering inside and outside the measured phase"]
fn native_editor_checker_resources() {
    alloc::mark_installed();
    let state = ServerState::new();
    let uri = Url::parse("file:///native/Resources.vue").unwrap();
    let mut inputs = vec![
        ("empty".to_owned(), "<script></script><template><i/></template>".to_owned()),
        ("template-only".to_owned(), "<template><span>🌱</span></template>".to_owned()),
        ("recovery".to_owned(), "<script setup lang='ts'>const invalid: = ;</script>".to_owned()),
        ("broken-sibling".to_owned(), "<script>const bad = ;</script><script setup lang='ts'>import { ref } from 'vue'; const value = ref(1);</script>".to_owned()),
        ("tsx".to_owned(), "<script setup lang='tsx'>import { A } from './A'; const view = <A/>;</script>".to_owned()),
    ];
    for lang in ["js", "ts"] {
        for count in [1, 8, 64] {
            let mut source = format!(
                "<script lang='{lang}'>export const first = '🌱';</script><script setup lang='{lang}'>\n"
            );
            for i in 0..count {
                writeln!(
                    source,
                    "import val{i} from './item{i}'; const x{i} = val{i};"
                )
                .unwrap();
            }
            source.push_str("</script><template><span/></template>");
            inputs.push((format!("{lang}-{count}"), source));
        }
    }
    for (name, source) in inputs {
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
                    .script_query_products_with_diagnostics_from(
                        black_box(&state),
                        black_box(native),
                    )
                    .unwrap()
            } else {
                black_box(&snapshot)
                    .script_query_products_with_diagnostics(black_box(&state))
                    .unwrap()
            }
        };
        same(
            read(true, &owner).as_ref(),
            read(false, &owner).as_ref(),
            &name,
        );
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
                "native_editor_checker_heap {name} cold={cold} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
            );
            assert_calibrated_paired_time(
                &format!("native editor checker {name} cold={cold}"),
                32,
                run,
            );
        }
    }
}
