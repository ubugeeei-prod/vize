//! Real ordinary-owner destruction, with no concurrent libtest measurement.

use davinci_harness::alloc::{CountingAllocator, mark_installed, stats};
use oxc_parser::{EmbeddingGoal, EmbeddingInput};
use oxc_span::SourceType;
use vize_l0::Allocator;

#[global_allocator]
static GLOBAL: CountingAllocator<std::alloc::System> = CountingAllocator::system();

const CASE: &str = "original_embedding_diagnostics_drop_before_their_arena";
const DESCRIPTOR_CASE: &str =
    "original_vue2_descriptor_diagnostics_drop_before_arena_and_on_unwind";
type Case = (&'static str, fn() -> Result<(), &'static str>);
const CASES: [Case; 2] = [
    (CASE, original_embedding_diagnostics_drop_before_their_arena),
    (
        DESCRIPTOR_CASE,
        original_vue2_descriptor_diagnostics_drop_before_arena_and_on_unwind,
    ),
];

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

fn original_embedding_diagnostics_drop_before_their_arena() -> Result<(), &'static str> {
    mark_installed();
    for (goal, source) in [
        (EmbeddingGoal::Expr, "/x/uv /*keep*/"),
        (EmbeddingGoal::HandlerBody, "return /x/uv; //keep"),
        (EmbeddingGoal::Parameters, "item=/x/uv /*keep*/"),
    ] {
        let baseline = stats().live_bytes;
        let arena = Allocator::default();
        let input = EmbeddingInput::prepare_in(arena.as_oxc(), source, SourceType::mjs(), goal)
            .map_err(|_| "static valid explicit profile and length")?;
        let observation = input.observe();
        if !observation.diagnostics().has_errors() || observation.panicked() {
            return Err("original nonfatal diagnostics must remain observable");
        }
        let with_owner = stats().live_bytes;
        match goal {
            EmbeddingGoal::Expr => {
                let owner = observation
                    .into_expression()
                    .map_err(|_| "original Expr goal")?;
                if owner.admitted().is_some() {
                    return Err("nonfatal expression must not mint admission");
                }
                drop(owner);
            }
            EmbeddingGoal::HandlerBody => {
                let owner = observation
                    .into_handler_body()
                    .map_err(|_| "original body goal")?;
                if owner.admitted().is_some() {
                    return Err("nonfatal body must not mint admission");
                }
                drop(owner);
            }
            EmbeddingGoal::Parameters => {
                let owner = observation
                    .into_parameters()
                    .map_err(|_| "original parameters goal")?;
                if owner.admitted().is_some() {
                    return Err("nonfatal parameters must not mint admission");
                }
                drop(owner);
            }
        }
        let after_owner = stats().live_bytes;
        if after_owner >= with_owner {
            return Err("ordinary diagnostics must release heap before arena Drop");
        }
        drop(arena);
        if stats().live_bytes != baseline {
            return Err("no parser observation heap survives original owners");
        }
    }
    Ok(())
}

fn original_vue2_descriptor_diagnostics_drop_before_arena_and_on_unwind() -> Result<(), &'static str>
{
    use std::panic::{AssertUnwindSafe, catch_unwind};
    use vize_l0::config::{VueDialect, VueVersion};
    use vize_l1::{
        SurfaceParseOptions,
        container::{Vue, vue::DescriptorOptions},
    };
    mark_installed();
    // Warm only the standard unwind machinery before the isolated heap window.
    drop(catch_unwind(|| {
        panic!("initialize isolated unwind observation")
    }));
    let baseline = stats().live_bytes;
    let arena = Allocator::default();
    let source = "<template>{{ value + }}</template>";
    let options = DescriptorOptions {
        version: VueVersion::V2,
        dialect: VueDialect::Vue,
        template: SurfaceParseOptions::default(),
    };
    let owner = Vue.observe_vue2_descriptor(&arena, source, options);
    let selected = owner
        .selected()
        .map_err(|_| "supported original envelope")?;
    let binding = selected
        .component()
        .bindings()
        .first()
        .ok_or("original binding")?;
    if binding
        .chain()
        .ok_or("started original chain")?
        .base()
        .diagnostics()
        .count()
        == 0
    {
        return Err("original body diagnostics must remain normally owned");
    }
    let with_owner = stats().live_bytes;
    drop(owner);
    if stats().live_bytes >= with_owner {
        return Err("descriptor must release Component/NativeSyntax heap before arena Drop");
    }
    let mut with_unwind_owner = 0;
    let interrupted = catch_unwind(AssertUnwindSafe(|| {
        let owner = Vue.observe_vue2_descriptor(&arena, source, options);
        assert!(owner.selected().is_ok());
        assert!(
            owner
                .component()
                .unwrap()
                .bindings()
                .first()
                .unwrap()
                .chain()
                .unwrap()
                .base()
                .diagnostics()
                .count()
                > 0
        );
        with_unwind_owner = stats().live_bytes;
        panic!("unwind after genuine complete Component custody");
    }));
    if interrupted.is_ok() {
        return Err("original owner must actually unwind");
    }
    drop(interrupted);
    if stats().live_bytes >= with_unwind_owner {
        return Err("unwind must release the normal retained syntax diagnostics");
    }
    drop(arena);
    if stats().live_bytes != baseline {
        return Err("no original descriptor heap survives arena Drop");
    }
    Ok(())
}
