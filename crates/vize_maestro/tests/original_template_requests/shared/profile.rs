use super::*;
use davinci_harness::{
    alloc::{CountingAllocator, mark_installed, measure_returning},
    paired::assert_paired_time,
};
use std::{alloc::System, hint::black_box};

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

#[test]
#[ignore = "isolated source requests, including publication, retained decisions and mapped CSS"]
fn shared_template_and_style_requests_match_two_independent_publications() {
    mark_installed();
    for (width, map) in [(1, false), (1, true), (16, true), (64, true)] {
        let docs = DocumentStore::new();
        let cache = SourceSnapshotCache::default();
        let uri = Url::parse("file:///joint.vue").unwrap();
        let markup = "<p v-if='ok && next'>雪</p>".repeat(width);
        let source = vize_l0::cstr!(
            "<script src='external.ts'></script>\r\n<template>😀{markup}</template><style>.x{{color:red}}</style>"
        );
        docs.open(uri.clone(), source.into(), 1, "vue".into());
        cache.capture(&docs, &uri).unwrap();
        let run = |combined: bool| {
            if combined {
                query(&docs, &cache, &uri, |_, arena, owner| {
                    (template(arena, owner), style(owner, map).unwrap())
                })
                .publish(&docs, |out| out)
                .unwrap()
            } else {
                let template = query(&docs, &cache, &uri, |_, arena, owner| {
                    template(arena, owner)
                })
                .publish(&docs, |out| out)
                .unwrap();
                let css = query(&docs, &cache, &uri, |_, _, owner| {
                    style(owner, map).unwrap()
                })
                .publish(&docs, |out| out)
                .unwrap();
                (template, css)
            }
        };
        let (prior, before) = measure_returning(|| run(false));
        let (shared, after) = measure_returning(|| run(true));
        assert_eq!(prior, shared);
        assert_eq!(prior.0.decisions.len(), width * 2);
        assert_eq!(prior.1.css(), Some(".x{color:red}"));
        let (before, after) = (before.unwrap(), after.unwrap());
        let label = vize_l0::cstr!(
            "editor template+style width={width} map={map} allocations={}/{} peak_bytes={}/{}",
            before.calls,
            after.calls,
            before.peak_bytes_over_start,
            after.peak_bytes_over_start
        );
        println!("{label}");
        assert!(
            after.calls <= before.calls
                && after.peak_bytes_over_start <= before.peak_bytes_over_start,
            "{label}"
        );
        assert_paired_time(&label, 24, |together| {
            black_box(run(together));
        });
    }
}
