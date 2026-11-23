//! Complete original products, including the two parser and publication boundaries.
use super::*;
use crate::source_project::SourceSnapshotCache;
use davinci_harness::alloc;
use std::{fmt::Write, hint::black_box, time::Instant};
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
    let time = |composed| {
        let start = Instant::now();
        for _ in 0..8 {
            drop(black_box(run(composed)));
        }
        start.elapsed().as_nanos() / 8
    };
    let mut pairs = Vec::new();
    for round in 0..35 {
        let flip = round % 2 != 0;
        let a = time(flip);
        let b = time(!flip);
        if round >= 4 {
            pairs.push(if flip { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "script_product_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );
    assert!(after.calls <= before.calls, "{name} allocation count");
    assert!(
        after.peak_bytes_over_start <= before.peak_bytes_over_start,
        "{name} peak bytes"
    );
    let median = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[samples.len() / 2]
    };
    let (baseline, composed) = pairs.into_iter().unzip();
    assert!(median(composed) <= median(baseline), "{name} median time");
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
            || name.ends_with("/ScriptEnd.vue.txt"))
            && let Ok(text) = std::fs::read_to_string(root.join(name))
        {
            compare(&state, &uri, name, &text);
        }
    }
}
