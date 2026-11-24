//! Measure a full cursor request on the same original script for each arm.
use davinci_harness::{alloc::measure, paired::assert_paired_time};
use std::hint::black_box;
use vize_l0::{
    Allocator, String, append,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::query::{IndexedModel, Model};

fn query(model: IndexedModel<'_, '_>, offsets: &[u32], joined: bool) {
    for &offset in offsets {
        let offset = black_box(offset);
        let results = if joined {
            let answer = model.script_position_at_offset(offset);
            (answer.reference, answer.binding, answer.scope)
        } else {
            (
                model.reference_at_offset(offset),
                model.binding_at_offset(offset),
                model.scope_at_offset(offset),
            )
        };
        let _ = black_box(results);
    }
}

pub(super) fn run() {
    for functions in [1, 32, 128] {
        let mut source = String::from("<script setup>const seed = 1;\n");
        for n in 0..functions {
            append!(
                &mut source,
                "function f{n}(p) {{ const value = seed + p; return value; }}\n"
            );
        }
        source.push_str("</script><template>{{ seed }}</template>");
        let arena = Allocator::default();
        let observed = lower_sfc_native(
            &arena,
            &source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let native = observed.admitted().unwrap();
        let scan = Model::new(native.file().file());
        let index = scan.index_positions();
        let offsets = [
            source.find("seed =").unwrap(),
            source.rfind("seed +").unwrap(),
            source.find("function").unwrap(),
            source.rfind("value;").unwrap(),
            source.rfind("+ p").unwrap() + 2,
            source.rfind(" }").unwrap(),
            source.find("<template>").unwrap(),
            source.len(),
        ]
        .map(|at| u32::try_from(at).unwrap());
        for cold in [false, true] {
            let read = |joined| {
                if cold {
                    query(black_box(scan).index_positions().model(), &offsets, joined);
                } else {
                    query(black_box(index.model()), &offsets, joined);
                }
            };
            let old = measure(|| read(false)).unwrap();
            let new = measure(|| read(true)).unwrap();
            assert_eq!(new, old, "functions={functions} cold={cold}");
            if !cold {
                assert_eq!((new.calls, new.peak_bytes_over_start), (0, 0));
            }
            eprintln!(
                "script_position_heap functions={functions} cold={cold} calls={}/{} peak={}/{}",
                old.calls, new.calls, old.peak_bytes_over_start, new.peak_bytes_over_start
            );
            assert_paired_time(
                &format!(
                    "script position functions={functions} cursors={} cold={cold}",
                    offsets.len()
                ),
                if cold { 16 } else { 128 },
                read,
            );
        }
    }
}
