use super::super::*;
use davinci_harness::{
    alloc::{mark_installed, measure_returning},
    paired::assert_paired_time,
};
use std::hint::black_box;

#[test]
#[ignore = "isolated complete shared compiler and linter requests"]
fn combined_compiler_and_rich_diagnostics() {
    mark_installed();
    for width in [1, 8, 32, 96] {
        let text = source(width, "a &amp;&amp; (b ? c : d)");
        // Both callers receive the original stock descriptor before timing.
        let stock = parse_sfc(&text, SfcParseOptions::default()).unwrap();
        let run = |shared| {
            let arena = Allocator::default();
            let owner = Vue.observe_descriptor(&arena, black_box(&text), options());
            let block = owner.admitted_template().unwrap().unwrap();
            let template = OriginalTemplate::analyze(&arena, block, &mut NoObserver).unwrap();
            let warnings = published(
                &arena,
                &stock,
                &text,
                if shared { Some(&template) } else { None },
            );
            black_box(&template);
            (template.analysis().static_facts.len(), warnings)
        };
        let (prior, old) = measure_returning(|| run(false));
        let (current, new) = measure_returning(|| run(true));
        assert_eq!((prior.0, wire(&prior.1)), (current.0, wire(&current.1)));
        assert_eq!(current.0, width);
        assert_eq!(current.1.len(), usize::from(width != 1));
        let (old, new) = (old.unwrap(), new.unwrap());
        let label = format!(
            "compiler+complexity lint width={width} allocations={}/{} peak_bytes={}/{}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        println!("{label}");
        assert!(
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
            "{label}"
        );
        assert_paired_time(&label, 16, |shared| {
            black_box(run(shared));
        });
    }
}
