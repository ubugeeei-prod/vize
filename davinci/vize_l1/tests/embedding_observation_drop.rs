//! Real ordinary-owner destruction, with no concurrent libtest measurement.

use davinci_harness::alloc::{CountingAllocator, mark_installed, stats};
use oxc_parser::{EmbeddingGoal, EmbeddingInput};
use oxc_span::SourceType;
use vize_l0::Allocator;

#[global_allocator]
static GLOBAL: CountingAllocator<std::alloc::System> = CountingAllocator::system();

fn main() -> Result<(), &'static str> {
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
