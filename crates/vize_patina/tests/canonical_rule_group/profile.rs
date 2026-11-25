use super::*;
use davinci_harness::alloc::{self, CountingAllocator};
use std::{hint::black_box, time::Instant};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator::mimalloc();

#[derive(Debug, Clone, Copy)]
enum Kind {
    Clean,
    Images,
    Tabindex,
    Mixed,
    LateComment,
}

fn source(rows: usize, kind: Kind) -> String {
    let mut source = String::from("<template><main>");
    for _ in 0..rows {
        source.push_str(match kind {
            Kind::Clean => "<p tabindex='0'/><img alt=''/>",
            Kind::Images => "<img/><img/>",
            Kind::Tabindex => "<p tabindex='2'/><p tabindex='1'/>",
            Kind::Mixed | Kind::LateComment => "<img/><p tabindex='2'/>",
        });
    }
    if matches!(kind, Kind::LateComment) {
        source.push_str("<!-- suppress -->");
    }
    source.push_str("</main></template>");
    source
}

fn run(
    lint: &NativeCanonicalLint<'_, '_>,
    catalog: &impl MessageLookup,
    grouped: bool,
) -> (Answer, Answer) {
    if grouped {
        group::requested(black_box(lint), black_box(catalog))
    } else {
        separate(black_box(lint), black_box(catalog))
    }
}

#[test]
#[ignore = "counterbalanced timing and peak allocation on complete original canonical owners"]
fn paired_lint_requests() {
    alloc::mark_installed();
    let messages = translator().for_locale(Locale::En);
    for n in [1, 32, 128] {
        for kind in [
            Kind::Clean,
            Kind::Images,
            Kind::Tabindex,
            Kind::Mixed,
            Kind::LateComment,
        ] {
            let text = source(n, kind);
            let arena = Allocator::default();
            let owner = lower_sfc_native(&arena, &text, options());
            let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
            let (tab, img) = compare(&lint, &messages);
            let answers = (tab.as_ref().map(Vec::len), img.as_ref().map(Vec::len));
            let old = alloc::measure(|| black_box(run(&lint, &messages, false))).unwrap();
            let new = alloc::measure(|| black_box(run(&lint, &messages, true))).unwrap();
            let time = |group: bool| {
                let clock = Instant::now();
                for _ in 0..16 {
                    drop(black_box(run(&lint, &messages, group)));
                }
                clock.elapsed().as_nanos() / 16
            };
            let mut pairs = Vec::new();
            for i in 0..35 {
                let reverse = i % 2 == 1;
                let a = time(reverse);
                let b = time(!reverse);
                if i >= 4 {
                    pairs.push(if reverse { (b, a) } else { (a, b) });
                }
            }
            eprintln!(
                "rows={n} kind={kind:?} nodes={} answers={answers:?} calls={}/{} peak={}/{} pairs={pairs:?}",
                lint.l2ir().artifact().node_count(),
                old.calls,
                new.calls,
                old.peak_bytes_over_start,
                new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
                "{n} {kind:?} {old:?}/{new:?}"
            );
        }
    }
}

#[test]
#[ignore = "individual single-rule timings for source-built baseline and candidate observers"]
fn individual_lint_requests() {
    alloc::mark_installed();
    let messages = translator().for_locale(Locale::En);
    for n in [1, 32, 128] {
        for kind in [
            Kind::Clean,
            Kind::Images,
            Kind::Tabindex,
            Kind::Mixed,
            Kind::LateComment,
        ] {
            let text = source(n, kind);
            let arena = Allocator::default();
            let owner = lower_sfc_native(&arena, &text, options());
            let lint = NativeCanonicalLint::from_vue(owner.admitted().unwrap());
            for image in [false, true] {
                let run = || {
                    if image {
                        black_box(&lint).img_alt(black_box(&messages))
                    } else {
                        black_box(&lint).tabindex_no_positive(black_box(&messages))
                    }
                };
                let allocation = alloc::measure(run).unwrap();
                let mut times = Vec::new();
                for i in 0..35 {
                    let start = Instant::now();
                    for _ in 0..32 {
                        drop(black_box(run()));
                    }
                    if i >= 4 {
                        times.push(start.elapsed().as_nanos() / 32);
                    }
                }
                eprintln!(
                    "individual rows={n} kind={kind:?} rule={} calls={} peak={} ns={times:?}",
                    if image { "image" } else { "tabindex" },
                    allocation.calls,
                    allocation.peak_bytes_over_start
                );
            }
        }
    }
}

#[test]
#[ignore = "counterbalanced timings and peak allocation on admitted original files"]
fn original_request_costs() {
    alloc::mark_installed();
    let manifest = fs::read(std::env::var_os("VIZE_CANONICAL_LINT_INPUTS").unwrap()).unwrap();
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let messages = translator().for_locale(Locale::En);
    let mut count = Counts::default();
    let mut measured = 0usize;
    for raw in manifest.split(|&b| b == 0).filter(|s| !s.is_empty()) {
        let path = core::str::from_utf8(raw).unwrap();
        let bytes = fs::read(root.join(path)).unwrap();
        let Ok(text) = core::str::from_utf8(&bytes) else {
            continue;
        };
        with_original(text, path, &mut count, |lint, _| {
            let old = alloc::measure(|| black_box(run(lint, &messages, false))).unwrap();
            let new = alloc::measure(|| black_box(run(lint, &messages, true))).unwrap();
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
                "{path} {old:?}/{new:?}"
            );
            let time = |group| {
                let start = Instant::now();
                for _ in 0..32 {
                    drop(black_box(run(lint, &messages, group)));
                }
                start.elapsed().as_nanos() / 32
            };
            let mut pairs = Vec::new();
            for i in 0..19 {
                let reverse = i % 2 == 1;
                let a = time(reverse);
                let b = time(!reverse);
                if i >= 4 {
                    pairs.push(if reverse { (b, a) } else { (a, b) });
                }
            }
            eprintln!(
                "original {path:?} nodes={} calls={}/{} peak={}/{} pairs={pairs:?}",
                lint.l2ir().artifact().node_count(),
                old.calls,
                new.calls,
                old.peak_bytes_over_start,
                new.peak_bytes_over_start
            );
            measured += 1;
            Ok(())
        })
        .unwrap();
    }
    assert!(measured > 0);
    eprintln!("measured={measured}");
}
