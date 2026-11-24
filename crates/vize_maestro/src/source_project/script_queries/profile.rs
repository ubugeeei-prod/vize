//! Complete original products, including the two parser and publication boundaries.
use super::*;
use crate::source_project::SourceSnapshotCache;
use davinci_harness::{alloc, paired::assert_paired_time};
use std::{fmt::Write, hint::black_box, sync::Arc};
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

fn run(
    snapshot: &SourceSnapshot,
    state: &ServerState,
    composed: bool,
) -> Option<ScriptQueryProducts> {
    if composed {
        black_box(snapshot)
            .script_query_products(black_box(state))
            .unwrap()
    } else {
        independent(black_box(snapshot), black_box(state))
    }
}

/// Compare both products and their deterministic heap use. Return the
/// snapshot when it publishes an outline, so its time can be paired too.
fn compare(
    state: &ServerState,
    uri: &Url,
    name: &str,
    source: &str,
    failures: &mut Vec<String>,
) -> Option<Arc<SourceSnapshot>> {
    state
        .documents
        .open(uri.clone(), source.into(), 4, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let old = run(&snapshot, state, false);
    assert_eq!(old, run(&snapshot, state, true), "{name}");
    if old.as_ref().is_none_or(|old| old.symbols.is_empty()) {
        return None;
    }
    let before = alloc::measure(|| run(&snapshot, state, false)).unwrap();
    let after = alloc::measure(|| run(&snapshot, state, true)).unwrap();
    eprintln!(
        "script_product_heap {name} calls={}/{} peak={}/{}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );
    if after.calls > before.calls || after.peak_bytes_over_start > before.peak_bytes_over_start {
        failures.push(name.to_owned());
    }
    Some(snapshot)
}

// Every original input is compared and its heap use checked. Wall time is
// paired per controlled or named input and once over all other original
// inputs, so adding a fixture never changes which files are measured.
#[test]
#[ignore = "paired allocations and time for both editor products over original inputs"]
fn combined_original_products() {
    alloc::mark_installed();
    let state = ServerState::new();
    let mut failures = Vec::new();
    let mut timed = Vec::new();
    // A distinct URI per input keeps every retained snapshot current.
    let mut uris = 0..;
    let mut uri = || {
        Url::parse(&vize_l0::cstr!(
            "file:///projects/{}.vue",
            uris.next().unwrap()
        ))
        .unwrap()
    };
    let mut controlled = |name: &str, source: &str, failures: &mut Vec<String>| {
        if let Some(snapshot) = compare(&state, &uri(), name, source, failures) {
            timed.push((name.to_owned(), snapshot));
        }
    };
    controlled(
        "whitespace-only script setup",
        "<script setup lang='ts'>\n</script>",
        &mut failures,
    );
    controlled(
        "upstream empty TypeScript setup",
        include_str!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#3373/NavBar.vue"
        )),
        &mut failures,
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
            controlled(&vize_l0::cstr!("{lang}-{count}"), &source, &mut failures);
        }
    }
    let manifest = std::fs::read(std::env::var_os("VIZE_SCRIPT_QUERY_INPUTS").unwrap()).unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut corpus = Vec::new();
    for entry in manifest.split(|&b| b == 0).filter(|p| !p.is_empty()) {
        let name = std::str::from_utf8(entry).unwrap();
        let Ok(text) = std::fs::read_to_string(root.join(name)) else {
            continue;
        };
        let Some(snapshot) = compare(&state, &uri(), name, &text, &mut failures) else {
            continue;
        };
        if name.contains("document-symbol-outline")
            || name.ends_with("/transition-custom-child-6881/sources/Fullessential.vue")
            || name.ends_with("/content-mapper-module-default-export/Picker.vue.txt")
            || name.ends_with("/ScriptEnd.vue.txt")
        {
            timed.push((name.to_owned(), snapshot));
        } else {
            corpus.push(snapshot);
        }
    }
    assert!(failures.is_empty(), "heap regressions on {failures:?}");
    assert!(!corpus.is_empty(), "no original inputs");
    for (name, snapshot) in &timed {
        assert_paired_time(name, 8, |composed| drop(run(snapshot, &state, composed)));
    }
    assert_paired_time(
        &vize_l0::cstr!("{} other original inputs", corpus.len()),
        1,
        |composed| {
            for snapshot in &corpus {
                drop(run(snapshot, &state, composed));
            }
        },
    );
}
