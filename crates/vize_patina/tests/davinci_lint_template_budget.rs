//! The public template lint entry's allocation gate runs directly on the
//! process main thread, so libtest's result-channel bookkeeping stays outside
//! the measured window.

use davinci_harness::alloc::{CountingAllocator, mark_installed, measure_returning};
use vize_patina::Linter;

#[global_allocator]
static GLOBAL: CountingAllocator = CountingAllocator::mimalloc();

const SECTION: &str = r#"<section v-if="ready"><!-- note --><b :title="heading">Hello {{ name }}!</b><ul><li v-for="item in items" :key="item.id">{{ item }}</li></ul><button @click="save">Save</button></section>"#;

const CASE: &str = "lint_template_allocation_budget";

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
    let selected = filter.as_ref().is_none_or(|filter| {
        if exact {
            filter == CASE
        } else {
            CASE.contains(filter)
        }
    });
    if ignored || !selected {
        return Ok(());
    }
    if list {
        println!("{CASE}: test");
        return Ok(());
    }
    lint_template_allocation_budget();
    Ok(())
}

fn lint_template_allocation_budget() {
    mark_installed();
    let linter = Linter::new();
    let source = format!("<main>{}</main>", SECTION.repeat(96));
    drop(linter.lint_template(&source, "fixture.vue"));
    let (result, metrics) = measure_returning(|| linter.lint_template(&source, "fixture.vue"));
    assert!(result.diagnostics.is_empty());
    let calls = metrics.expect("counting allocator installed").calls;
    // Recording every lowering decision took 4,417 calls on this fixture.
    // The product entry used 3,831 after switching to the unobserved lowering.
    assert!(
        calls <= 4_000,
        "{calls} allocations while linting a template"
    );
}
