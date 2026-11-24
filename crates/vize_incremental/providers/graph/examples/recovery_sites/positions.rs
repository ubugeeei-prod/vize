use core::{hint::black_box, ops::ControlFlow};
use davinci_harness::{alloc, paired::assert_paired_time};
use vize_l0::{Span, cstr};
use vize_resident_graph::{RecoveryRecord, TemplateRecovery};

fn for_each<'a>(
    recovery: &TemplateRecovery<'a>,
    points: &[u32],
    together: bool,
    mut consume: impl FnMut(usize, RecoveryRecord<'a>, Span, &'a str),
) {
    let mut scan = |points: &[u32], offset: usize| {
        let result = recovery.visit_positions(points, |row| {
            for index in row.positions {
                consume(offset + index, row.record, row.authored, row.text);
            }
            ControlFlow::<()>::Continue(())
        });
        assert_eq!(result, Ok(ControlFlow::Continue(())));
    };
    if together {
        scan(points, 0);
    } else {
        for (i, cursor) in points.iter().enumerate() {
            scan(core::slice::from_ref(cursor), i);
        }
    }
}

pub(crate) fn measure<const N: usize>(recovery: &TemplateRecovery<'_>, first: u32, width: usize) {
    // Include the start, interior and right endpoint of the damaged fragments.
    let mut sites = vec![first];
    for hole in recovery.holes() {
        let (range, spelling) = recovery.source_site(hole.span).unwrap();
        sites.push(range.start);
        if let Some((offset, _)) = spelling.char_indices().nth(1) {
            sites.push(range.start + offset as u32);
        }
        sites.push(range.end);
    }
    sites.sort_unstable();
    let points: [u32; N] = core::array::from_fn(|i| {
        if N == 1 {
            recovery
                .source_site(recovery.holes()[0].span)
                .unwrap()
                .0
                .start
        } else {
            sites[i * (sites.len() - 1) / (N - 1)]
        }
    });
    let collect = |together| {
        let mut rows: [_; N] = core::array::from_fn(|_| Vec::new());
        for_each(recovery, &points, together, |i, record, span, text| {
            rows[i].push((record, span, text));
        });
        rows
    };
    let expected = collect(false);
    assert_eq!(collect(true), expected);
    let count: usize = expected.iter().map(Vec::len).sum();
    assert!(count > 0);

    let run = |together| {
        let mut counts = [0usize; N];
        for_each(
            black_box(recovery),
            black_box(&points),
            together,
            |i, record, span, text| {
                black_box((record, span, text));
                counts[i] += 1;
            },
        );
        black_box(counts);
    };
    for together in [false, true] {
        let ((), cost) = alloc::measure_returning(|| run(together));
        let cost = cost.unwrap();
        assert_eq!((cost.calls, cost.peak_bytes_over_start), (0, 0));
    }
    // With one cursor both readers issue the same `visit_positions` request,
    // so a timing pair would only compare two inlined copies of this harness.
    // The output and allocation checks above still cover that request.
    if N > 1 {
        assert_paired_time(
            &cstr!("recovery position query width={width} cursors={N} hits={count}"),
            32,
            run,
        );
    }
}
