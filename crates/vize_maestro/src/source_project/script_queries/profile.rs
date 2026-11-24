//! Complete original products, including the two parser and publication boundaries.
use super::*;
use crate::source_project::SourceSnapshotCache;
use davinci_harness::{alloc, paired::assert_calibrated_paired_time};
use std::{fmt::Write, hint::black_box};
use tower_lsp::lsp_types::Url;

fn independent(snapshot: &SourceSnapshot, state: &ServerState) -> Option<ScriptQueryProducts> {
    let descriptor = snapshot
        .with_current(&state.documents, |_| {
            state.sfc_descriptor(snapshot.uri(), snapshot.source())
        })
        .unwrap()?;
    let symbols = ScriptQueryProducts::separate_symbols(snapshot.source(), &descriptor);
    let module_specifiers = ScriptQueryProducts::reference_specifiers(&descriptor);
    snapshot.check_current(&state.documents).unwrap();
    Some(ScriptQueryProducts {
        symbols,
        module_specifiers,
    })
}

fn compare(state: &ServerState, uri: &Url, name: &str, source: &str) {
    state
        .documents
        .open(uri.clone(), source.into(), 4, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let run = |composed| {
        if composed {
            black_box(&snapshot)
                .script_query_products(black_box(state))
                .unwrap()
        } else {
            independent(black_box(&snapshot), black_box(state))
        }
    };
    let old = run(false);
    assert_eq!(old, run(true), "{name}");
    if old.as_ref().is_none_or(|old| old.symbols.is_empty()) {
        return;
    }
    let before = alloc::measure(|| run(false)).unwrap();
    let after = alloc::measure(|| run(true)).unwrap();
    eprintln!(
        "script_product_heap {name} calls={}/{} peak={}/{}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );
    assert!(after.calls <= before.calls, "{name} allocation count");
    assert!(
        after.peak_bytes_over_start <= before.peak_bytes_over_start,
        "{name} peak bytes"
    );
    // Equal-cost reads move by about 1% with code layout, so time is compared
    // against an A/A control run in the same loop.
    assert_calibrated_paired_time(name, 8, |composed| drop(black_box(run(composed))));
}

#[test]
#[ignore = "paired allocations and time for both editor products over original inputs"]
fn combined_original_products() {
    alloc::mark_installed();
    let state = ServerState::new();
    let uri = Url::parse("file:///projects/Query.vue").unwrap();
    compare(
        &state,
        &uri,
        "whitespace-only script setup",
        "<script setup lang='ts'>\n</script>",
    );
    compare(
        &state,
        &uri,
        "upstream empty TypeScript setup",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#3373/NavBar.vue"
        )),
    );
    for lang in ["js", "ts", "jsx", "tsx"] {
        for count in [1, 8, 64] {
            let mut source = vize_l0::cstr!(
                "<script lang='{lang}'>export const first = '雪';</script><script setup lang='{lang}'>\n"
            );
            for i in 0..count {
                writeln!(
                    source,
                    "import value{i} from './leaf{i}'; const use{i} = value{i};"
                )
                .unwrap();
            }
            source.push_str("</script><template><div/></template>");
            compare(&state, &uri, &vize_l0::cstr!("{lang}-{count}"), &source);
        }
    }
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (i, entry) in manifest
        .split(|&b| b == 0)
        .filter(|p| !p.is_empty())
        .enumerate()
    {
        let name = std::str::from_utf8(entry).unwrap();
        if (i % 67 == 0
            || name.contains("document-symbol-outline")
            || name.ends_with("/transition-custom-child-6881/sources/Fullessential.vue")
            || name.ends_with("/content-mapper-module-default-export/Picker.vue.txt")
            || name.ends_with("/ScriptEnd.vue.txt"))
            && let Ok(text) = std::fs::read_to_string(root.join(name))
        {
            compare(&state, &uri, name, &text);
        }
    }
}
