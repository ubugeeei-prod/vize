use super::*;
use davinci_harness::{alloc, paired::assert_paired_time};
use std::{fmt::Write, hint::black_box, sync::Arc};

struct Tools {
    state: ServerState,
    linter: ScriptLinter,
    options: FormatOptions,
}

fn run(snapshot: &SourceSnapshot, tools: &Tools, joined: bool) -> ScriptToolProducts {
    let arena = Allocator::default();
    let formatter = GlyphFormatter::new(black_box(&tools.options), &arena);
    let (state, linter) = (black_box(&tools.state), black_box(&tools.linter));
    if joined {
        black_box(snapshot)
            .script_tool_products(state, linter, &formatter)
            .unwrap()
    } else {
        separate(black_box(snapshot), state, linter, &formatter)
    }
}

/// Compare both products and their deterministic heap use, then return the
/// snapshot so its time can be paired too.
fn measure(
    tools: &Tools,
    uri: &Url,
    name: &str,
    source: &str,
    failures: &mut Vec<String>,
) -> Arc<SourceSnapshot> {
    let documents = &tools.state.documents;
    documents.open(uri.clone(), source.into(), 1, "vue".into());
    let snapshot = SourceSnapshotCache::default()
        .capture(documents, uri)
        .unwrap();
    same(
        &run(&snapshot, tools, true),
        &run(&snapshot, tools, false),
        name,
    );
    let old = alloc::measure(|| run(&snapshot, tools, false)).unwrap();
    let new = alloc::measure(|| run(&snapshot, tools, true)).unwrap();
    eprintln!(
        "script_tool_heap {name} calls={}/{} peak={}/{}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    if new.calls > old.calls || new.peak_bytes_over_start > old.peak_bytes_over_start {
        failures.push(name.into());
    }
    snapshot
}

// Every original input is compared and its heap use checked. Wall time is
// paired per controlled or named input and once over all other original
// inputs, so adding a fixture never changes which files are measured.
#[test]
#[ignore = "alternating complete formatter, linter, checker and editor responses"]
fn complete_tool_products_on_ordinary_setup_and_recovered_inputs() {
    alloc::mark_installed();
    let tools = Tools {
        state: ServerState::new(),
        linter: ScriptLinter::with_vapor_rules(),
        options: FormatOptions::default(),
    };
    let mut failures = Vec::new();
    let mut timed = Vec::new();
    // A distinct URI per input keeps every retained snapshot current.
    let mut uris = 0..;
    let mut uri = || {
        Url::parse(&vize_l0::cstr!(
            "file:///editor/{}.vue",
            uris.next().unwrap()
        ))
        .unwrap()
    };
    let mut controlled = |name: &str, source: &str, failures: &mut Vec<String>| {
        let snapshot = measure(&tools, &uri(), name, source, failures);
        timed.push((name.to_owned(), snapshot));
    };
    for lang in ["js", "jsx", "ts", "tsx"] {
        for count in [1, 12] {
            let mut source = vize_l0::cstr!(
                "<script lang='{lang}'>export {{ v }} from './dependency'</script>\n<script setup lang='{lang}'>\n"
            );
            for i in 0..count {
                writeln!(source, "import x{i} from './local{i}'; const v{i} = x{i};").unwrap();
            }
            source.push_str("</script><template><span>🌱</span></template>");
            controlled(&vize_l0::cstr!("{lang}-{count}"), &source, &mut failures);
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
        controlled(name, source, &mut failures);
    }
    let mut corpus = Vec::new();
    for_originals(|_, name, source| {
        let snapshot = measure(&tools, &uri(), name, source, &mut failures);
        if name.ends_with("/ScriptEnd.vue.txt") {
            timed.push((name.to_owned(), snapshot));
        } else {
            corpus.push(snapshot);
        }
    });
    assert!(failures.is_empty(), "heap regressions on {failures:?}");
    for (name, snapshot) in &timed {
        assert_paired_time(name, 8, |joined| {
            drop(run(snapshot, &tools, joined));
        });
    }
    assert_paired_time(
        &vize_l0::cstr!("{} other original inputs", corpus.len()),
        1,
        |joined| {
            for snapshot in &corpus {
                drop(run(snapshot, &tools, joined));
            }
        },
    );
}
