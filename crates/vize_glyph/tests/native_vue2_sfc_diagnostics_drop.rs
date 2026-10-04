//! Isolated authentic whole-owner diagnostic heap drop and unwind.

use davinci_harness::alloc::{CountingAllocator, mark_installed, stats};
use vize_glyph::native_doc::{NativeVue2SfcOptions, observe_native_vue2_sfc_in};
use vize_l0::Allocator;

#[global_allocator]
static GLOBAL: CountingAllocator<std::alloc::System> = CountingAllocator::system();
type Case = (&'static str, fn() -> Result<(), &'static str>);
const CASES: [Case; 1] = [(
    "original_vue2_whole_sfc_diagnostics_drop_and_unwind",
    original_vue2_whole_sfc_diagnostics_drop_and_unwind,
)];

fn main() -> Result<(), &'static str> {
    let mut args = std::env::args().skip(1);
    let (mut list, mut ignored, mut exact) = (false, false, false);
    let mut filter = None;
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--list" => list = true,
            "--ignored" => ignored = true,
            "--exact" => exact = true,
            "--nocapture" => {}
            "--format" if args.next().as_deref() == Some("terse") => {}
            value if !value.starts_with('-') && filter.is_none() => filter = Some(arg),
            _ => return Err("unsupported harness argument"),
        }
    }
    if ignored {
        return Ok(());
    }
    for (case, run) in CASES {
        let selected = filter.as_ref().is_none_or(|filter| {
            if exact {
                filter == case
            } else {
                case.contains(filter)
            }
        });
        if !selected {
            continue;
        }
        if list {
            println!("{case}: test");
        } else {
            run()?;
            println!("{case}: ok");
        }
    }
    Ok(())
}

fn original_vue2_whole_sfc_diagnostics_drop_and_unwind() -> Result<(), &'static str> {
    use std::panic::{AssertUnwindSafe, catch_unwind};
    mark_installed();
    drop(catch_unwind(|| panic!("warm isolated unwind machinery")));
    let baseline = stats().live_bytes;
    let arena = Allocator::default();
    let source = "<template><div>{{ value + }}</div></template>";
    let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
    let syntax = owner
        .descriptor()
        .component()
        .ok_or("genuine Component")?
        .bindings()[0]
        .chain()
        .ok_or("original started chain")?
        .base();
    if syntax.diagnostics().count() == 0 || owner.document().is_ok() {
        return Err("real nonfatal diagnostics must refuse whole Doc");
    }
    let with_owner = stats().live_bytes;
    drop(owner);
    if stats().live_bytes >= with_owner {
        return Err("whole owner must release original diagnostic heap before arena");
    }
    let mut with_unwind_owner = 0;
    let interrupted = catch_unwind(AssertUnwindSafe(|| {
        let owner = observe_native_vue2_sfc_in(&arena, source, NativeVue2SfcOptions::default());
        assert!(
            owner.descriptor().component().unwrap().bindings()[0]
                .chain()
                .unwrap()
                .base()
                .diagnostics()
                .count()
                > 0
        );
        assert!(owner.document().is_err());
        with_unwind_owner = stats().live_bytes;
        let _owner = core::hint::black_box(owner);
        panic!("unwind actual whole Vue2 owner");
    }));
    if interrupted.is_ok() {
        return Err("actual whole owner must unwind");
    }
    drop(interrupted);
    if stats().live_bytes >= with_unwind_owner {
        return Err("whole owner unwind must release diagnostic heap");
    }
    drop(arena);
    if stats().live_bytes != baseline {
        return Err("no historical whole owner heap survives arena drop");
    }
    Ok(())
}
