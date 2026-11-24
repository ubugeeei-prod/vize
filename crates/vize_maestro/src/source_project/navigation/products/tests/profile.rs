use super::*;
use davinci_harness::alloc;
use std::{hint::black_box, time::Instant};

pub(super) fn measure(
    project: &NativeNavigationProject<'_>,
    uri: &Url,
    pos: Position,
    name: &str,
    failures: &mut Vec<String>,
) {
    let run = |joined| {
        if joined {
            block_on(black_box(project).symbol_products(black_box(uri), black_box(pos), true))
        } else {
            separate(black_box(project), black_box(uri), black_box(pos), true)
        }
    };
    let original = run(false);
    assert_eq!(run(true), original, "{name}");
    if !original.is_ok_and(|p| p.definition.is_some()) {
        return;
    }
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |joined| {
        let start = Instant::now();
        for _ in 0..8 {
            drop(black_box(run(joined)));
        }
        start.elapsed().as_nanos() / 8
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
        "native_navigation_time {name} calls={}/{} peak={}/{} ns={pairs:?}",
        old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    let median = |mut values: Vec<u128>| {
        values.sort_unstable();
        values[values.len() / 2]
    };
    let (a, b) = pairs.into_iter().unzip();
    if new.calls > old.calls
        || new.peak_bytes_over_start > old.peak_bytes_over_start
        || median(b) > median(a)
    {
        failures.push(name.into());
    }
}

#[test]
#[ignore = "complete three-product requests across the genuine mailbox and publication checks"]
fn paired_original_navigation_requests() {
    alloc::mark_installed();
    let state = Arc::new(ServerState::new());
    let uri = Url::parse("file:///navigation/Measure.vue").unwrap();
    let mut failures = Vec::new();
    let mut count = 0;
    let mut compare = |source: &str, name: &str, at: u32| {
        state
            .documents
            .open(uri.clone(), source.into(), 1, "vue".into());
        let project =
            NativeNavigationProject::new(SourceQueryProject::new_server(Arc::clone(&state)));
        measure(
            &project,
            &uri,
            corpus::position(source, at),
            name,
            &mut failures,
        );
        state.documents.close(&uri);
        project.notify_host_change(&uri);
    };
    for repeats in [1, 8, 64] {
        let mut source = vize_l0::String::from("<script setup>let value=0;");
        for _ in 0..repeats {
            source.push_str("value++;");
        }
        source.push_str("</script><template><p :title='value'>{{value}}</p></template>");
        for (index, at) in [
            source.find("value").unwrap(),
            source.rfind("value").unwrap(),
        ]
        .into_iter()
        .enumerate()
        {
            compare(
                &source,
                &vize_l0::cstr!("uses-{repeats}-{index}"),
                at as u32,
            );
        }
    }
    corpus::originals(|_, name, source| {
        let arena = vize_l0::Allocator::default();
        let owner = vize_l1_to_l2::native_file::lower_sfc_native(&arena, source, corpus::options());
        let Some(native) = owner.admitted() else {
            return;
        };
        if let Some(at) = corpus::sites(&native)
            .into_iter()
            .find(|&at| native.binding_at_offset(at).is_ok_and(|b| b.is_some()))
        {
            count += 1;
            compare(source, name, at);
        }
    });
    eprintln!("native_navigation_measured_originals={count}");
    assert!(failures.is_empty(), "regressions on {failures:?}");
}
