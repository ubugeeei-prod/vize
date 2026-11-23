//! Paired requests return the complete editor response on each side.
use davinci_harness::{
    alloc::{self, CountingAllocator},
    paired::assert_paired_time,
};
use std::{hint::black_box, sync::Arc};
use vize_incremental::template::{TemplateDocument, TemplateReport};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
    pass::NoObserver,
};
use vize_l1::container::{Vue, vue::DescriptorOptions};
use vize_l1_to_l2::pass::template::OriginalTemplate;

#[global_allocator]
static ALLOCATOR: CountingAllocator<std::alloc::System> = CountingAllocator::system();

fn direct(source: &str) -> TemplateReport {
    let arena = Allocator::default();
    let descriptor = Vue.observe_descriptor(
        &arena,
        source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: Default::default(),
        },
    );
    let selected = descriptor.admitted_template().unwrap().unwrap();
    let template = OriginalTemplate::analyze(&arena, selected, &mut NoObserver).unwrap();
    TemplateReport {
        opening: template.open_tag(),
        content: template.block().span(),
        complexity: template.analysis().complexity.clone(),
        diagnostics: template.l2ir().diagnostics.clone(),
    }
}

fn read_document(document: &mut TemplateDocument, source: &Arc<String>) -> TemplateReport {
    document
        .read(black_box(source), &mut NoObserver)
        .unwrap()
        .unwrap()
        .report()
        .unwrap()
}

#[test]
#[ignore = "first requested document on a worker and complete editor sessions"]
fn first_request_and_document_lifecycle() {
    alloc::mark_installed();
    for (name, body) in [
        ("one", "<p v-if='a && b'>雪</p>".to_owned()),
        ("wide", "<i v-if='a && b'>雪</i>".repeat(64)),
    ] {
        let original = Arc::new(format!("<template>{body}</template>"));
        let moved = Arc::new(format!("<!--🧭-->\r\n{original}"));
        let damaged = Arc::new(moved.replace("v-if='a && b'", ":x="));
        let repaired = Arc::new(moved.replace("a && b", "ready || done"));
        let revisions = [&original, &original, &moved, &damaged, &repaired];
        let session = |resident, revisions: &[&Arc<String>]| {
            let mut document = TemplateDocument::default();
            let mut latest = None;
            for source in revisions {
                latest = Some(if resident {
                    read_document(&mut document, source)
                } else {
                    direct(black_box(source))
                });
                black_box(&latest);
            }
            latest.unwrap()
        };
        vize_l0::pool::clear();
        let cold_old = alloc::measure(|| session(false, &revisions[..1])).unwrap();
        let cold_new = alloc::measure(|| session(true, &revisions[..1])).unwrap();
        eprintln!(
            "resident_template {name} edit=empty-worker calls={}/{} peak={}/{}",
            cold_old.calls,
            cold_new.calls,
            cold_old.peak_bytes_over_start,
            cold_new.peak_bytes_over_start
        );
        assert!(
            cold_new.calls <= cold_old.calls
                && cold_new.peak_bytes_over_start <= cold_old.peak_bytes_over_start
        );
        for (phase, inputs) in [
            ("first", &revisions[..1]),
            ("lifecycle", revisions.as_slice()),
        ] {
            assert_eq!(session(false, inputs), session(true, inputs));
            let old = alloc::measure(|| session(false, inputs)).unwrap();
            let new = alloc::measure(|| session(true, inputs)).unwrap();
            eprintln!(
                "resident_template {name} edit={phase} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert!(
                new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start
            );
            assert_paired_time(
                &format!("resident_template {name} edit={phase}"),
                32,
                |resident| {
                    drop(black_box(session(resident, inputs)));
                },
            );
        }
        let mut document = TemplateDocument::default();
        let before = alloc::stats().live_bytes;
        drop(read_document(&mut document, &original));
        let held = alloc::stats().live_bytes - before;
        drop(document);
        assert_eq!(alloc::stats().live_bytes, before);
        eprintln!("resident_template {name} heap_retained_per_document={held}");
    }
}

#[test]
#[ignore = "optimized whole requests on current and moving original source frames"]
fn unchanged_and_edited_revisions() {
    alloc::mark_installed();
    let mut regressions = Vec::new();
    for (name, body) in [
        ("one", "<p v-if='a && b'>雪</p>".into()),
        (
            "branching",
            "<section v-if='a'><b v-for='item in xs'>{{item}}</b></section>".repeat(12),
        ),
        ("wide", "<i v-if='a && b' class='x'>雪</i>".repeat(64)),
        ("incomplete", "<p :x=>雪</p>".into()),
    ] {
        let first = format!("<script src='old.ts'></script><template>{body}</template>");
        let moved = format!(
            "<!--🧭-->\r\n<script lang='coffee'>value = -> 2</script><template>{body}</template>"
        );
        let changed = first
            .replace("a && b", "b || ready")
            .replace("v-if='a'", "v-if='ready && b'")
            .replace(":x=>", ":x='ok'>");
        assert_ne!(first, changed);
        let first = Arc::new(first);
        let moved = Arc::new(moved);
        let changed = Arc::new(changed);
        for (edit, second) in [("stable", &first), ("prefix", &moved), ("body", &changed)] {
            let mut document = TemplateDocument::default();
            let mut reads = [0usize; 2];
            let mut request = |resident: bool| {
                let count = &mut reads[usize::from(resident)];
                let source = if count.is_multiple_of(2) {
                    &first
                } else {
                    second
                };
                *count += 1;
                if resident {
                    read_document(&mut document, source)
                } else {
                    direct(black_box(source))
                }
            };
            for _ in 0..4 {
                assert_eq!(request(false), request(true));
            }
            let (before, old) = alloc::measure_returning(|| request(false));
            let (after, new) = alloc::measure_returning(|| request(true));
            assert_eq!(before, after);
            let old = old.unwrap();
            let new = new.unwrap();
            eprintln!(
                "resident_template {name} edit={edit} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            if new.calls > old.calls || new.peak_bytes_over_start > old.peak_bytes_over_start {
                regressions.push(format!("{name} {edit} memory"));
            }
            let timing = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
                assert_paired_time(
                    &format!("resident_template {name} edit={edit}"),
                    32,
                    |resident| {
                        drop(black_box(request(resident)));
                    },
                )
            }));
            if timing.is_err() {
                regressions.push(format!("{name} {edit} time"));
            }
        }
    }
    assert!(
        regressions.is_empty(),
        "request regressions: {regressions:?}"
    );
}
