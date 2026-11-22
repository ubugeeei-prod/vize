use super::*;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "paired cold and retained module projections"]
fn paired_exports() {
    alloc::mark_installed();
    for size in [0, 1, 8, 32, 128] {
        let mut source = String::from("<script lang='ts'>");
        for n in 0..size {
            source.push_str(&format!("export const {{field:[a{n},,...b{n}]}}=input;"));
        }
        source.push_str("</script>");
        // Preserve the empty ordinary block in the ground-truth descriptor.
        if size == 0 {
            source = "<script>/* empty */</script>".into()
        }
        let descriptor = parse_sfc(&source, Default::default()).unwrap();
        let arena = Allocator::new();
        let (ts, retry) = parses(&arena, frame(&descriptor));
        let summary = analyze_sfc_descriptor(&descriptor, None, SfcCroquisOptions::full());
        for cold in [false, true] {
            assert!(measure(
                &descriptor,
                &summary,
                (&ts, retry.as_ref()),
                &size.to_string(),
                cold,
                8
            ));
        }
    }
}

pub(super) fn measure(
    descriptor: &SfcDescriptor<'_>,
    summary: &Croquis,
    parsed: (&AuthoredProgram<'_>, Option<&AuthoredProgram<'_>>),
    label: &str,
    cold: bool,
    iterations: u128,
) -> bool {
    let block = parsed.0.block();
    let run = |joined| {
        let plan = if joined && cold {
            let fresh = Allocator::new();
            let (ts, retry) = parses(&fresh, black_box(block));
            PlainScriptBridge::new(black_box(descriptor), &ts, retry.as_ref()).unwrap()
        } else if joined {
            PlainScriptBridge::new(black_box(descriptor), black_box(parsed.0), parsed.1).unwrap()
        } else {
            previous(black_box(block))
        };
        finish(plan, summary)
    };
    // In the real checker another reader may have already warmed its scoped
    // Program cache. Give the original collectors that same opportunity.
    let _scope = vize_croquis::script_parser::SharedParseScope::enter();
    assert_eq!(run(false), run(true));
    let old = alloc::measure(|| run(false)).unwrap();
    let new = alloc::measure(|| run(true)).unwrap();
    let time = |joined, count| {
        let start = Instant::now();
        for _ in 0..count {
            drop(black_box(run(joined)));
        }
        start.elapsed().as_nanos() / count
    };
    let sample = |count, reverse| {
        let mut pairs = Vec::with_capacity(31);
        for round in 0..35 {
            let first = (round % 2 == 0) != reverse;
            let a = time(first, count);
            let b = time(!first, count);
            if round >= 4 {
                pairs.push(if first { (b, a) } else { (a, b) });
            }
        }
        pairs
    };
    let pairs = sample(iterations, false);
    let slower = |p: &[(u128, u128)]| p.iter().filter(|(a, b)| b > a).count() >= 23;
    let mut pass = new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start;
    if slower(&pairs) {
        let confirm = sample(iterations * 8, true);
        eprintln!("bridge_confirmation input={label} cold={cold} ns={confirm:?}");
        pass &= !slower(&confirm);
    }
    if iterations > 1 || !pass {
        eprintln!("bridge_pairs input={label} cold={cold} ns={pairs:?}");
    }
    let (mut a, mut b): (Vec<_>, Vec<_>) = pairs.into_iter().unzip();
    a.sort_unstable();
    b.sort_unstable();
    eprintln!(
        "bridge_time input={label} cold={cold} ns={}/{} calls={}/{} peak={}/{} pass={pass}",
        a[15], b[15], old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    pass
}

fn finish(
    plan: PlainScriptBridge<'_>,
    summary: &Croquis,
) -> (
    usize,
    PlainScriptExportText,
    Vec<VizeMapping>,
    Vec<CompactString>,
) {
    let mut mappings = vec![VizeMapping::new(
        0..plan.block.source().len(),
        plan.block.start() as usize..plan.block.end() as usize,
    )];
    let text = plan.masked_script();
    let prefix = CompactString::new(text.as_ref());
    let mut fields = Vec::new();
    plan.append_setup_fields(&mut fields);
    let published = plan.project(summary, prefix, &mut mappings);
    (
        plan.relocated_declarations().count(),
        published,
        mappings,
        fields,
    )
}
