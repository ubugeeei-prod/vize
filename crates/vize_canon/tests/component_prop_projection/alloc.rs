//! Isolate allocation counters from the checker and the test runner.
#![expect(
    clippy::unwrap_used,
    clippy::panic,
    clippy::disallowed_types,
    reason = "private executable runs the same fixed inputs as the timing harness"
)]
mod input;
use davinci_harness::alloc::{CountingAllocator, mark_installed, measure};
use std::{alloc::System, hint::black_box, path::Path};
use vize_atelier_jsx::JsxLang;
use vize_canon::batch::generate_jsx_document_virtual_ts;
use vize_l0::Allocator;
use vize_l4::targets::ts::project_module_with_component_checks;

#[global_allocator]
static ALLOCATOR: CountingAllocator<System> = CountingAllocator::system();

fn main() {
    mark_installed();
    for roots in [1, 32, 128] {
        let source = input::repeated(roots);
        let arena = Allocator::default();
        let module = input::lower(&arena, &source);
        let original = || {
            black_box(
                generate_jsx_document_virtual_ts(Path::new("View.tsx"), &source, JsxLang::Tsx)
                    .unwrap(),
            );
        };
        let complete = || {
            let arena = Allocator::default();
            let module = input::lower(&arena, &source);
            black_box(project_module_with_component_checks(&module).unwrap());
        };
        let retained = || {
            black_box(project_module_with_component_checks(&module).unwrap());
        };
        original();
        complete();
        retained();
        let original = measure(original).unwrap();
        let complete = measure(complete).unwrap();
        let retained = measure(retained).unwrap();
        println!("roots={roots} original={original:?} complete={complete:?} retained={retained:?}");
        for current in [complete, retained] {
            assert!(
                current.calls <= original.calls,
                "{roots}: {original:?} / {current:?}"
            );
            assert!(
                current.peak_bytes_over_start <= original.peak_bytes_over_start,
                "{roots}: {original:?} / {current:?}"
            );
        }
    }
}
