use super::*;
use davinci_harness::alloc;
use std::{fmt::Write, hint::black_box, time::Instant};

fn compare(state: &ServerState, uri: &Url, name: &str, source: &str, failures: &mut Vec<String>) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let run = |joined| {
        if joined {
            black_box(&snapshot)
                .script_query_products_with_diagnostics(black_box(state))
                .unwrap()
        } else {
            separate(black_box(&snapshot), black_box(state))
        }
    };
    let old = run(false);
    same(run(true).as_ref(), old.as_ref(), name);
    if old
        .as_ref()
        .is_none_or(|(editor, _)| editor.symbols.is_empty())
    {
        return;
    }
    let before = alloc::measure(|| run(false)).unwrap();
    let after = alloc::measure(|| run(true)).unwrap();
    let time = |joined| {
        let start = Instant::now();
        for _ in 0..12 {
            drop(black_box(run(joined)));
        }
        start.elapsed().as_nanos() / 12
    };
    let mut pairs = Vec::new();
    for round in 0..35 {
        let first = round % 2 != 0;
        let a = time(first);
        let b = time(!first);
        if round >= 4 {
            pairs.push(if first { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "checker_product_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
        before.calls, after.calls, before.peak_bytes_over_start, after.peak_bytes_over_start
    );

    let median = |mut samples: Vec<u128>| {
        samples.sort_unstable();
        samples[samples.len() / 2]
    };
    let (baseline, composed) = pairs.into_iter().unzip();
    if after.calls > before.calls
        || after.peak_bytes_over_start > before.peak_bytes_over_start
        || median(composed) > median(baseline)
    {
        failures.push(name.to_owned());
    }
}

#[test]
#[ignore = "paired complete responses over controlled grammars and original Vue inputs"]
fn shared_and_distinct_checking_grammars() {
    alloc::mark_installed();
    let mut failures = Vec::new();
    let state = ServerState::new();
    let uri = Url::parse("file:///checker/Measure.vue").unwrap();
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
            compare(
                &state,
                &uri,
                &vize_l0::cstr!("{lang}-{count}"),
                &source,
                &mut failures,
            );
        }
    }
    for (name, source) in [
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
        compare(&state, &uri, name, source, &mut failures);
    }
    for_originals(|index, name, source| {
        if index % 67 == 0
            || name.contains("document-symbol-outline")
            || name.ends_with("/ScriptEnd.vue.txt")
        {
            compare(&state, &uri, name, source, &mut failures);
        }
    });
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
