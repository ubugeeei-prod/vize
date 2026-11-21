//! Compare complete public projection requests and record reuse separately.
//! Both generators see the same module and their owned outputs are consumed
//! and dropped inside the timed interval.
use super::*;
use std::{hint::black_box, time::Instant};

#[test]
#[ignore = "run alone with an optimized build to measure component projection"]
fn paired_component_projection() {
    for roots in [1, 32, 128] {
        let source = super::input::repeated(roots);
        let arena = Allocator::default();
        let module = lower(&arena, &source);
        let old = || {
            black_box(
                generate_jsx_document_virtual_ts(
                    Path::new("View.tsx"),
                    black_box(&source),
                    JsxLang::Tsx,
                )
                .unwrap(),
            );
        };
        let cold = || {
            let arena = Allocator::default();
            let module = lower(&arena, black_box(&source));
            black_box(project_module_with_component_checks(&module).unwrap());
        };
        let retained = || {
            black_box(project_module_with_component_checks(black_box(&module)).unwrap());
        };
        measure("full", roots, old, cold);
        measure("retained", roots, old, retained);
    }
}

fn measure(label: &str, roots: usize, old: impl Fn(), new: impl Fn()) {
    let runs = if roots == 1 { 128 } else { 8 };
    let elapsed = |f: &dyn Fn()| {
        let start = Instant::now();
        for _ in 0..runs {
            f();
        }
        start.elapsed().as_nanos() / runs
    };
    old();
    new();
    let pairs: Vec<_> = (0..31)
        .map(|i| {
            if i % 2 == 0 {
                (elapsed(&old), elapsed(&new))
            } else {
                let b = elapsed(&new);
                (elapsed(&old), b)
            }
        })
        .collect();
    println!("mode={label} roots={roots} runs={runs} pairs={pairs:?}");
}
