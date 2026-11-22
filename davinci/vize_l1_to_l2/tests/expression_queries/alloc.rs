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
#[cfg(feature = "setup-component-imports")]
mod component_projection;
mod input;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn main() {
    mark_installed();
    #[cfg(feature = "setup-component-imports")]
    {
        component_imports();
        streaming_component_spelling();
    }
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
        #[cfg(feature = "template-operand-queries")]
        {
            let stats = measure(|| {
                let mut operands = 0;
                let mut uses = 0;
                model
                    .for_each_resolved_template_operand(|op| {
                        black_box((op.kind(), op.span(), op.text(), op.scope()));
                        op.for_each_reference(|site| {
                            black_box(site);
                            uses += 1
                        })
                        .unwrap();
                        operands += 1;
                    })
                    .unwrap();
                assert_eq!((operands, uses), (2 * repeats, 2 * repeats));
            })
            .unwrap();
            assert_eq!((stats.calls, stats.peak_bytes_over_start), (0, 0));
        }
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

#[cfg(feature = "setup-component-imports")]
fn streaming_component_spelling() {
    let tail = "long".repeat(40);
    let source = format!(
        "<script setup>import Card{tail}Child from './c'</script><template><card{tail}-child/></template>"
    );
    let arena = Allocator::default();
    let owner = lower_sfc_native(
        &arena,
        &source,
        DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        },
    );
    let selected = owner.descriptor().admitted().unwrap();
    let program = owner.scripts()[0]
        .syntax()
        .unwrap()
        .admitted_program()
        .unwrap();
    let file = owner.rejected_file().unwrap().file().unwrap();
    let mut count = 0;
    let stats = measure(|| {
        Model::new(file)
            .visit_setup_component_imports(
                selected.setup().unwrap(),
                &program,
                selected.template().unwrap(),
                |site| {
                    assert_eq!((site.module(), site.export()), ("./c", "default"));
                    count += 1;
                },
            )
            .unwrap();
    })
    .unwrap();
    assert_eq!((count, stats.calls, stats.peak_bytes_over_start), (1, 0, 0));
}

#[cfg(feature = "setup-component-imports")]
fn component_imports() {
    use component_projection::{draw, original, product, retained_product};
    for (size, kebab) in [1, 8, 32, 128]
        .into_iter()
        .flat_map(|size| [(size, false), (size, true)])
    {
        let source = input::component_imports(size, kebab);
        let opts = DescriptorOptions {
            version: VueVersion::V3,
            dialect: VueDialect::Vue,
            template: SurfaceParseOptions::default(),
        };
        let arena = Allocator::default();
        let owner = lower_sfc_native(&arena, &source, opts);
        let imports = owner.setup_component_imports().unwrap();
        let (croquis, start, _) = draw(&source);
        assert_eq!(
            original(&source, &croquis, start),
            retained_product(&imports)
        );
        let reads = measure(|| {
            assert_eq!(black_box(&imports).sites().len(), size);
            for site in imports.sites() {
                assert_eq!(
                    imports
                        .at_offset(black_box(site.name_span().start))
                        .unwrap()
                        .node(),
                    site.node()
                );
            }
        })
        .unwrap();
        assert_eq!((reads.calls, reads.peak_bytes_over_start), (0, 0));
        for cold in [false, true] {
            let original_stats = measure(|| {
                if cold {
                    let (c, s, _) = draw(black_box(&source));
                    black_box(original(&source, &c, s));
                } else {
                    black_box(original(&source, black_box(&croquis), start));
                }
            })
            .unwrap();
            let native_stats = measure(|| {
                if cold {
                    let a = Allocator::default();
                    black_box(product(&lower_sfc_native(&a, black_box(&source), opts)).unwrap());
                } else {
                    black_box(retained_product(black_box(&imports)));
                }
            })
            .unwrap();
            eprintln!(
                "component_import_alloc size={size} kebab={kebab} cold={cold} calls={}/{} peak={}/{} lookup_calls={} lookup_peak={}",
                original_stats.calls,
                native_stats.calls,
                original_stats.peak_bytes_over_start,
                native_stats.peak_bytes_over_start,
                reads.calls,
                reads.peak_bytes_over_start
            );
            assert!(
                native_stats.calls <= original_stats.calls
                    && native_stats.peak_bytes_over_start <= original_stats.peak_bytes_over_start
            );
        }
    }
}
