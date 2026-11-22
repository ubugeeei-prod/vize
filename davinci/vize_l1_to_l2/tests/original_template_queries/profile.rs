use super::super::*;
use davinci_harness::{
    alloc::{mark_installed, measure_returning},
    paired::assert_paired_time,
};
use std::hint::black_box;

#[test]
#[ignore = "isolated complete source request, lossless output, diagnostics and both analyses"]
fn complete_source_request() {
    mark_installed();
    for (name, text, sibling) in [
        ("small", "<div class='x'>雪</div>".to_owned(), ""),
        (
            "medium",
            "<p v-if='a && b' class='x'>雪</p>".repeat(16),
            "<script setup>const a=true</script>",
        ),
        (
            "wide-partial",
            "<i v-if='a && b' class='x'>雪</i>".repeat(96),
            "<script src='outside.ts'></script>",
        ),
        (
            "recovery",
            "<p :x=>雪</p>".to_owned(),
            "<style src='external.css'></style>",
        ),
    ] {
        let source = format!("<!--🧭-->\r\n{sibling}<template>{text}</template>");
        let run = |candidate: bool| {
            let arena = Allocator::default();
            let descriptor = Vue.observe_descriptor(&arena, black_box(&source), options());
            let selected = descriptor.admitted_template().unwrap().unwrap();
            if candidate {
                let session = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
                let output = (
                    render_source(session.l1ir()),
                    session.analysis().static_facts.len(),
                    session.analysis().complexity.contributions.len(),
                    session.l2ir().diagnostics.len(),
                );
                black_box(&session);
                output
            } else {
                // Give the independent queries the same shared descriptor, single
                // parse and single lowering as the combined public request.
                let block = selected.block();
                let (tree, errors) =
                    parse_with_options(&arena, block.source(), selected.surface_options());
                let mut owner = lower_source_block(&arena, &tree, &errors, block);
                let statics = hoist::run(&mut owner);
                let complexity = cfg::run(&owner);
                let output = (
                    render_source(&tree),
                    statics.len(),
                    complexity.contributions.len(),
                    owner.diagnostics.len(),
                );
                black_box((&owner, &statics, &complexity));
                output
            }
        };
        let (before, old) = measure_returning(|| run(false));
        let (after, new) = measure_returning(|| run(true));
        assert_eq!(before, after);
        assert_eq!(before.0, text);
        let (old, new) = (old.unwrap(), new.unwrap());
        let label = format!(
            "original template request={name} allocations={}/{} peak_bytes={}/{}",
            old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
        );
        println!("{label}");
        assert!(
            new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
            "{label}"
        );
        assert_paired_time(&label, 32, |candidate| {
            black_box(run(candidate));
        });
    }
}
