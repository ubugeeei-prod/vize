use super::*;
use davinci_harness::{alloc, paired::assert_calibrated_paired_time};
use std::{fmt::Write, hint::black_box};

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
    eprintln!(
        "script_tool_heap {name} calls={}/{} peak={}/{}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    if new.calls > old.calls || new.peak_bytes_over_start > old.peak_bytes_over_start {
        failures.push(name.into());
    }
    // Equal-cost reads move by about 1% with code layout, so time is compared
    // against an A/A control run in the same loop.
    assert_calibrated_paired_time(name, 8, |joined| drop(black_box(run(joined))));
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
