//! A private executable keeps the allocator counters isolated from other tests.

#![expect(
    clippy::expect_used,
    reason = "fixed inputs and the installed test allocator are invariants of this executable"
)]

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use std::{alloc::System, hint::black_box};
use vize_l0::{Allocator, SourceRoot, pass::NoRemarks};
use vize_l1::markup::parse;
use vize_l1_to_l2::lower_source_block;
use vize_l2::{
    analysis::{static_facts, template, template_complexity},
    provenance::NoProvenance,
};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn main() {
    mark_installed();
    for (name, markup, copies) in [
        ("empty", "", 1),
        ("one owner", "<i>value</i>", 1),
        ("native", "<i class='row'>value</i>", 128),
        (
            "control",
            "<template v-for='x in xs'><i v-if='x.ok && flag'>{{x.name}}</i><b v-else>no</b></template>",
            64,
        ),
        (
            "slots",
            "<Card :a='a' @click='act'><template #default='{row}'><i v-if='row.ok'>{{row.name}}</i></template></Card>",
            64,
        ),
    ] {
        let source = markup.repeat(copies);
        let arena = Allocator::default();
        let file = SourceRoot::new(&source).expect("bounded template");
        let (tree, errors) = parse(&arena, &source);
        let lowered = lower_source_block(
            &arena,
            &tree,
            &errors,
            file.block(&source, 0).expect("entire authored file"),
        );
        let root = &lowered.artifact.root;
        let count = lowered.op_count;
        let original = measure(|| {
            black_box((
                static_facts::run(root, count, &mut NoProvenance),
                template_complexity::run(root, count),
            ));
        })
        .expect("installed counting allocator");
        let shared = measure(|| {
            black_box(template::run(
                root,
                count,
                &mut NoProvenance,
                &mut NoRemarks,
            ));
        })
        .expect("installed counting allocator");
        assert!(
            shared.calls <= original.calls,
            "{name}: {original:?} / {shared:?}"
        );
        assert!(
            shared.peak_bytes_over_start <= original.peak_bytes_over_start,
            "{name}: {original:?} / {shared:?}"
        );
    }
}
