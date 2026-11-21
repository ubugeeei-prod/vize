use super::*;
use davinci_harness::alloc;
use std::{fmt::Write, hint::black_box, time::Instant};

fn measure(
    state: &ServerState,
    uri: &Url,
    name: &str,
    source: &str,
    linter: &ScriptLinter,
    failures: &mut Vec<String>,
) {
    state
        .documents
        .open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(&state.documents, uri)
        .unwrap();
    let options = FormatOptions::default();
    let run = |joined| {
        let arena = Allocator::default();
        let formatter = GlyphFormatter::new(black_box(&options), &arena);
        if joined {
            black_box(&snapshot)
                .script_tool_products(black_box(state), black_box(linter), &formatter)
                .unwrap()
        } else {
            separate(
                black_box(&snapshot),
                black_box(state),
                black_box(linter),
                &formatter,
            )
        }
    };
    same(&run(true), &run(false), name);
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let timing = |joined| {
        let start = Instant::now();
        for _ in 0..8 {
            drop(black_box(run(joined)));
        }
        start.elapsed().as_nanos() / 8
    };
    let mut pairs = Vec::new();
    for round in 0..35 {
        let first = round % 2 == 0;
        let a = timing(first);
        let b = timing(!first);
        if round >= 4 {
            pairs.push(if first { (b, a) } else { (a, b) });
        }
    }
    eprintln!(
        "script_tool_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    let median = |mut ns: Vec<u128>| {
        ns.sort_unstable();
        ns[ns.len() / 2]
    };
    let (old_ns, new_ns) = pairs.into_iter().unzip();
    if new.calls > old.calls
        || new.peak_bytes_over_start > old.peak_bytes_over_start
        || median(new_ns) > median(old_ns)
    {
        failures.push(name.into());
    }
}

#[test]
#[ignore = "alternating complete formatter, linter, checker and editor responses"]
fn complete_tool_products_on_ordinary_setup_and_recovered_inputs() {
    alloc::mark_installed();
    let state = ServerState::new();
    let uri = Url::parse("file:///editor/Profile.vue").unwrap();
    let linter = ScriptLinter::with_vapor_rules();
    let mut failures = Vec::new();
    for lang in ["js", "jsx", "ts", "tsx"] {
        for count in [1, 12] {
            let mut source = vize_l0::cstr!(
                "<script lang='{lang}'>export {{ v }} from './dependency'</script>\n<script setup lang='{lang}'>\n"
            );
            for i in 0..count {
                writeln!(source, "import x{i} from './local{i}'; const v{i} = x{i};").unwrap();
            }
            source.push_str("</script><template><span>🌱</span></template>");
            measure(
                &state,
                &uri,
                &vize_l0::cstr!("{lang}-{count}"),
                &source,
                &linter,
                &mut failures,
            );
        }
    }
    for (name, source) in [
        (
            "syntax-recovery",
            "<script lang='ts'>import './valid';const broken: = ;</script>",
        ),
        ("template-only", "<template><span>🌱</span></template>"),
        (
            "both-blocks",
            "<script>export default {}</script><script setup lang='tsx'>const v=<span/>;</script>",
        ),
    ] {
        measure(&state, &uri, name, source, &linter, &mut failures);
    }
    for_originals(|i, name, source| {
        if i % 179 == 0 || name.ends_with("/ScriptEnd.vue.txt") {
            measure(&state, &uri, name, source, &linter, &mut failures);
        }
    });
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
