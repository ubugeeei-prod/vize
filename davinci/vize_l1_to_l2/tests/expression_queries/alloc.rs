//! Isolate the optional index from the global allocator activity of libtest.
#![expect(clippy::unwrap_used, reason = "fixed allocator qualification inputs")]
use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use std::{alloc::System, hint::black_box};
use vize_l0::{
    Allocator,
    config::{VueDialect, VueVersion},
};
use vize_l1::{container::vue::DescriptorOptions, markup::SurfaceParseOptions};
use vize_l1_to_l2::native_file::lower_sfc_native;
use vize_l2::query::{ExpressionPositionIndex, Model};
mod input;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn main() {
    mark_installed();
    for repeats in [1, 32, 128] {
        let source = input::repeated(repeats);
        let arena = Allocator::default();
        let original = lower_sfc_native(
            &arena,
            &source,
            DescriptorOptions {
                version: VueVersion::V3,
                dialect: VueDialect::Vue,
                template: SurfaceParseOptions::default(),
            },
        );
        let native = original.admitted().unwrap();
        let file = native.file().file();
        let offset = source.rfind("value").unwrap() as u32;
        let model = Model::new(file);
        let index = ExpressionPositionIndex::new(file);
        let native_lookup = || native.reference_at_offset(black_box(offset)).unwrap();
        let scan_lookup = || {
            model
                .expression_reference_at_offset(black_box(offset))
                .unwrap()
        };
        let indexed_lookup = || {
            index
                .expression_reference_at_offset(black_box(offset))
                .unwrap()
        };
        let old = measure(|| black_box(native_lookup())).unwrap();
        let scan = measure(|| black_box(scan_lookup())).unwrap();
        let first = measure(|| black_box(indexed_lookup())).unwrap();
        let prepare = measure(|| black_box(indexed_lookup())).unwrap();
        let repeated = measure(|| black_box(indexed_lookup())).unwrap();
        for demand in [old, scan, first, repeated] {
            assert_eq!((demand.calls, demand.peak_bytes_over_start), (0, 0));
        }
        if repeats == 1 {
            assert_eq!((prepare.calls, prepare.peak_bytes_over_start), (0, 0));
        }
        eprintln!(
            "repeats={repeats} base={old:?} scan={scan:?} first={first:?} second={prepare:?} repeated={repeated:?}"
        );
    }
}
