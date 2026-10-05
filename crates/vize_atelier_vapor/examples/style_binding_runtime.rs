//! Source-built public compiler output for the retained #7600 runtime regression.

use serde_json::json;
use vize_atelier_core::{WhitespaceStrategy, parser::with_whitespace_strategy};
use vize_atelier_vapor::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let source = include_str!("../tests/fixtures/static-style-merged-binding.input.txt");
    let cases = [
        ("condense", WhitespaceStrategy::Condense),
        ("preserve", WhitespaceStrategy::Preserve),
    ]
    .into_iter()
    .map(|(whitespace, strategy)| {
        let allocator = Allocator::new();
        let result = with_whitespace_strategy(strategy, || {
            compile_vapor(
                &allocator,
                source,
                VaporCompilerOptions {
                    prefix_identifiers: true,
                    davinci_retained_lane: true,
                    ..Default::default()
                },
            )
        });
        json!({
            "whitespace": whitespace,
            "source": source,
            "retainedLane": true,
            "prefixIdentifiers": true,
            "code": result.code.as_str(),
            "errorMessages": result.error_messages,
        })
    })
    .collect::<Vec<_>>();
    println!("{}", serde_json::to_string(&cases)?);
    Ok(())
}
