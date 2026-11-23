use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

fn sample(label: &str, ctx: &IdeContext<'_>, imports: &SetupComponentImports<'_, '_>) {
    let read = alloc::measure(|| {
        for site in black_box(imports).sites() {
            site.visit_static_listeners(|row| {
                black_box(row);
            })
            .unwrap();
        }
    })
    .unwrap();
    assert_eq!((read.calls, read.peak_bytes_over_start), (0, 0), "{label}");
    let input = input_edits(ctx);
    let run = |shared| {
        let mut edits = black_box(&input).clone();
        if shared {
            RenameService::rewrite_component_event_edits(
                black_box(ctx),
                black_box(imports),
                edits,
                black_box("nextValue"),
            )
            .unwrap()
        } else {
            rewrite_text_edits(
                black_box(ctx),
                ctx.uri,
                &mut edits,
                black_box("nextValue"),
                RenameKind::Event,
            );
            edits
        }
    };
    assert_eq!(run(false), run(true), "{label}");
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |shared, n| {
        let start = Instant::now();
        for _ in 0..n {
            drop(black_box(run(shared)));
        }
        start.elapsed().as_nanos() / n
    };
    let measure = |iterations, reverse| {
        let (mut before, mut after) = (Vec::new(), Vec::new());
        let mut slower = 0;
        for i in 0..35 {
            let first = (i % 2 == 0) != reverse;
            let a = time(first, iterations);
            let b = time(!first, iterations);
            if i >= 4 {
                let (old, new) = if first { (b, a) } else { (a, b) };
                before.push(old);
                after.push(new);
                slower += usize::from(new > old);
            }
        }
        before.sort_unstable();
        after.sort_unstable();
        eprintln!(
            "component_event_rewrite case={label} edits={} ns={}/{} allocations={}/{} peak={}/{} slower={slower}/31",
            input.len(),
            before[15],
            after[15],
            old.calls,
            new.calls,
            old.peak_bytes_over_start,
            new.peak_bytes_over_start
        );
        slower >= 23
    };
    assert!(!(measure(8, false) && measure(32, true)), "{label}");
    assert!(
        new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
        "{label}"
    );
}

#[test]
#[ignore = "per-input comparisons of complete semantic rename edits from the resident original File"]
fn paired_event_rewrites() {
    alloc::mark_installed();
    let measure = |name: &str, source: &str, require_events: bool| {
        document(source, |ctx| {
            let arena = Allocator::new();
            let observed = observe(&arena, &ctx.content);
            let Ok(imports) = observed.setup_component_imports() else {
                return false;
            };
            let mut has_events = false;
            for site in imports.sites() {
                site.visit_static_listeners(|_| has_events = true).unwrap();
            }
            if require_events && !has_events {
                return false;
            }
            sample(name, ctx, &imports);
            true
        })
    };
    assert!(measure("Unicode nested references", INPUT, false));
    assert!(measure(
        "empty component",
        "<script setup>import A from './a';</script><template><A/></template>",
        false
    ));
    let list = std::fs::read_to_string(std::env::var_os("VIZE_EXPRESSION_QUERY_MANIFEST").unwrap())
        .unwrap();
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut measured = 0;
    for file in list.split_terminator('\0') {
        let bytes = std::fs::read(root.join(file)).unwrap();
        let Ok(source) = std::str::from_utf8(&bytes) else {
            continue;
        };
        if source.contains('@') || source.contains("v-on:") {
            measured += usize::from(measure(file, source, true));
        }
    }
    if std::env::var_os("VIZE_REQUIRE_REPOSITORY_LISTENERS").is_some() {
        assert!(measured >= 12);
    }
    eprintln!("component_event_rewrite real_files={measured}");
}
