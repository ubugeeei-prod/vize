use super::*;
use davinci_harness::{
    alloc,
    paired::{assert_calibrated_paired_time, assert_paired_time},
};
use std::{fmt::Write, hint::black_box, sync::Arc};

fn run(snapshot: &SourceSnapshot, state: &ServerState, joined: bool) -> Option<Products> {
    if joined {
        black_box(snapshot)
            .script_query_products_with_diagnostics(black_box(state))
            .unwrap()
    } else {
        separate(black_box(snapshot), black_box(state))
    }
}

/// Compare both responses and their deterministic heap use. Return the
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
        .open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let old = run(&snapshot, state, false);
    same(run(&snapshot, state, true).as_ref(), old.as_ref(), name);
    if old
        .as_ref()
        .is_none_or(|(editor, _)| editor.symbols.is_empty())
    {
        return None;
    }
    let before = alloc::measure(|| run(&snapshot, state, false)).unwrap();
    let after = alloc::measure(|| run(&snapshot, state, true)).unwrap();
    eprintln!(
        "checker_product_heap {name} calls={}/{} peak={}/{}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );
    if after.calls > before.calls || after.peak_bytes_over_start > before.peak_bytes_over_start {
        failures.push(name.to_owned());
    }
    Some(snapshot)
}

/// Named originals keep their own paired time. A JavaScript block parses with
/// two grammars, so the shared read saves no parse there and must not lose time.
fn timed_alone(name: &str) -> bool {
    name.contains("document-symbol-outline")
        || name.ends_with("/ScriptEnd.vue.txt")
        || name.ends_with("/template-context-undefined-10667/sources/javascript.vue.txt")
}

// Every original input is compared and its heap use checked, so a new fixture
// never shifts which inputs are measured. Wall time is paired per controlled or
// named input and once over all other originals. A strict median per file made
// equal-cost reads (both sides parse twice) fail at random.
// Each controlled or named input is timed against an A/A control in the same
// loop, so a layout-sized shift on an equal-cost read is not a regression; the
// pass over all other originals keeps the strict sign test.
#[test]
#[ignore = "paired complete responses over controlled grammars and original Vue inputs"]
fn shared_and_distinct_checking_grammars() {
    alloc::mark_installed();
    let mut failures = Vec::new();
    let mut timed = Vec::new();
    let state = ServerState::new();
    // A distinct URI per input keeps every retained snapshot current.
    let mut uris = 0..;
    let mut uri = || {
        Url::parse(&vize_l0::cstr!(
            "file:///checker/{}.vue",
            uris.next().unwrap()
        ))
        .unwrap()
    };
    let mut controlled = |name: &str, source: &str, failures: &mut Vec<String>| {
        if let Some(snapshot) = compare(&state, &uri(), name, source, failures) {
            timed.push((name.to_owned(), snapshot));
        }
    };
    for lang in ["js", "ts", "jsx", "tsx"] {
        for count in [1, 8, 64] {
            let mut source = vize_l0::cstr!(
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
            controlled(&vize_l0::cstr!("{lang}-{count}"), &source, &mut failures);
        }
    }
    for (name, source) in [
        (
            "upstream empty TypeScript setup",
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsc/#3373/NavBar.vue"
            )),
        ),
        (
            "checker-ts-recovery",
            "<script lang='ts'>const bad: = ;</script>",
        ),
        (
            "checker-js-recovery",
            "<script setup>const bad: = ;</script>",
        ),
        (
            "mixed-profiles",
            "<script>const first = 1;</script><script setup lang='tsx'>const node = <div/>;</script>",
        ),
    ] {
        controlled(name, source, &mut failures);
    }
    let mut corpus = Vec::new();
    for_originals(|_, name, source| {
        let Some(snapshot) = compare(&state, &uri(), name, source, &mut failures) else {
            return;
        };
        if timed_alone(name) {
            timed.push((name.to_owned(), snapshot));
        } else {
            corpus.push(snapshot);
        }
    });
    assert!(failures.is_empty(), "heap regressions on {failures:?}");
    assert!(!corpus.is_empty(), "no original inputs");
    for (name, snapshot) in &timed {
        assert_calibrated_paired_time(name, 12, |joined| drop(run(snapshot, &state, joined)));
    }
    assert_paired_time(
        &vize_l0::cstr!("{} other original inputs", corpus.len()),
        1,
        |joined| {
            for snapshot in &corpus {
                drop(run(snapshot, &state, joined));
            }
        },
    );
}
