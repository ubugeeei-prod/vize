use super::*;
use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use std::{alloc::System, hint::black_box, time::Instant};
use vize_l1::{container::vue::Vue, css::lang::StyleLanguage};
use vize_l2::lang::scss::ScssProgram;
use vize_l4::{targets::scss, write::EmitDocument};

mod map_corpus;
mod prepared;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn elapsed(mut run: impl FnMut()) -> u128 {
    let start = Instant::now();
    for _ in 0..24 {
        run();
    }
    start.elapsed().as_nanos() / 24
}

fn paired(label: &str, run: impl Fn(bool)) -> (u128, u128) {
    let old = measure(|| run(false)).unwrap();
    let new = measure(|| run(true)).unwrap();
    let (mut a, mut b) = (Vec::new(), Vec::new());
    for round in 0..35 {
        let first = round % 2 == 0;
        let x = elapsed(|| run(first));
        let y = elapsed(|| run(!first));
        if round >= 4 {
            a.push(if first { y } else { x });
            b.push(if first { x } else { y });
        }
    }
    a.sort_unstable();
    b.sort_unstable();
    println!(
        "whole_scss {label} ns={}/{} calls={}/{} peak={}/{}",
        a[15], b[15], old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
    );
    assert!(b[15] <= a[15], "{label}: median {} vs {}", b[15], a[15]);
    assert!(
        new.calls <= old.calls && new.peak_bytes_over_start <= old.peak_bytes_over_start,
        "{label}"
    );
    (a[15], b[15])
}

#[test]
#[ignore = "optimized isolated allocator and paired timings"]
fn composed_css_documents_and_full_component_cost() {
    mark_installed();
    for count in [1, 8, 48] {
        let mut source =
            "<script setup>let n=1</script><template><p>{{n}}</p></template>".to_owned();
        for i in 0..count {
            source.push_str(&format!(
                "<style lang='scss'>$v:4px;.c{i}{{width:$v;.child{{margin:$v 0}}}}</style>"
            ));
        }
        let arena = Allocator::default();
        let owner = Vue.observe_descriptor(
            &arena,
            &source,
            NativeSfcCompileOptions::default().descriptor,
        );
        let programs: Vec<_> = owner
            .admitted()
            .unwrap()
            .styles()
            .map(|style| {
                StyleLanguage::Scss
                    .with_original_structure(style.block(), ScssProgram::lower)
                    .unwrap()
            })
            .collect();
        for mapped in [false, true] {
            let docs = |direct| {
                let mut css = EmitDocument::new(mapped);
                for p in &programs {
                    if !css.is_empty() {
                        css.push_str("\n");
                    }
                    if direct {
                        scss::append(&mut css, p);
                    } else {
                        let mut fragment = EmitDocument::new(mapped);
                        scss::append(&mut fragment, p);
                        css.push_spanned(&fragment);
                    }
                }
                css
            };
            assert_eq!(docs(false), docs(true));
            paired(
                &format!("style-assembly styles={count} mapped={mapped}"),
                |direct| drop(black_box(docs(direct))),
            );
            let options = NativeSfcCompileOptions {
                source_map: mapped,
                ..Default::default()
            };
            if mapped {
                let result = compile_native_sfc_with_scss(&arena, &source, options);
                let output = result.result().unwrap();
                let css = output.css_document().unwrap();
                let js = output.document();
                let maps = |indexed| {
                    if indexed {
                        let index = vize_l4::write::source_map::SourceMapIndex::new(&source);
                        (
                            index.source_map(js, FILENAME),
                            index.source_map(css, FILENAME),
                        )
                    } else {
                        (
                            js.source_map(FILENAME, &source),
                            css.source_map(FILENAME, &source),
                        )
                    }
                };
                assert_eq!(maps(false), maps(true));
                paired(&format!("complete-maps styles={count}"), |indexed| {
                    drop(black_box(maps(indexed)))
                });
            }
            let complete = || {
                let arena = Allocator::default();
                let result = compile_native_sfc_with_scss(&arena, black_box(&source), options);
                black_box(result.result().unwrap());
            };
            let stats = measure(complete).unwrap();
            let mut times: Vec<_> = (0..31).map(|_| elapsed(complete)).collect();
            times.sort_unstable();
            println!(
                "whole_scss native styles={count} mapped={mapped} ns={} calls={} peak={}",
                times[15], stats.calls, stats.peak_bytes_over_start
            );
        }
    }
}
