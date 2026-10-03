//! Mapping laws use the actual L1→L2→L4 provider; backend laws are separate.

#[path = "../../tests/native_program/support.rs"]
mod support;

use super::mapping;
use lsp_types::Diagnostic;
use vize_l0::{Allocator, Span, line_index::LineBreaks};
use vize_l1::embed::Lang;
use vize_l4::targets::ts::{MappingError, project_program, project_program_no_links};

type TestResult = Result<(), Box<dyn std::error::Error>>;

fn require(observation: bool, failure: &'static str) -> TestResult {
    if observation {
        Ok(())
    } else {
        Err(failure.into())
    }
}

fn diagnostic(start: (u32, u32), end: (u32, u32)) -> Result<Diagnostic, serde_json::Error> {
    serde_json::from_value(serde_json::json!({
        "range": {
            "start": {"line": start.0, "character": start.1},
            "end": {"line": end.0, "character": end.1}
        },
        "severity": 1,
        "code": 2339,
        "source": "ts",
        "message": "the original message\nwith detail",
        "codeDescription": {"href": "https://actual-project.example/2339"},
        "tags": [1, 2],
        "data": {"uri": "file:///actual-project/opaque.ts", "original": [true, 1]},
        "relatedInformation": [{
            "location": {
                "uri": "file:///actual-project/related.ts",
                "range": {
                    "start": { "line": 3, "character": 2 },
                    "end": { "line": 3, "character": 7 }
                }
            },
            "message": "the original related message"
        }]
    }))
}

#[test]
fn exact_lsp_mapping_preserves_payload_unicode_and_every_supported_line_convention() -> TestResult {
    for newline in ["\n", "\r", "\r\n", "\u{2028}", "\u{2029}"] {
        let source = vize_l0::cstr!("/*😀*/{newline}const 日本語='é'; 日本語.length;");
        let arena = Allocator::default();
        let file = support::file(&arena, &source, Lang::Js).ok_or("original complete File")?;
        let projection = project_program(&file).map_err(|_| "original projection")?;
        let start = source.find("length").ok_or("authored property")?;
        let end = start + "length".len();
        let input = diagnostic(
            LineBreaks::Lsp.offset_to_position(&source, start),
            LineBreaks::Lsp.offset_to_position(&source, end),
        )?;
        let original = serde_json::to_value(&input)?;
        let mapped = mapping::observe(&projection, input);
        require(
            mapped.span() == Ok(Span::new(u32::try_from(start)?, u32::try_from(end)?)),
            "exact authored bytes across supported line conventions",
        )?;
        require(
            serde_json::to_value(mapped.backend())? == original,
            "whole original message/code/related payload retained",
        )?;
        require(
            serde_json::to_value(mapped.original_range())?
                == original.get("range").cloned().ok_or("original range")?,
            "exact authored LSP range",
        )?;
    }
    Ok(())
}

#[test]
fn invalid_surrogates_lines_reversed_and_generated_ranges_are_retained_as_refusals() -> TestResult {
    let source = "/*😀*/ const value=1;";
    let arena = Allocator::default();
    let file = support::file(&arena, source, Lang::Js).ok_or("original complete File")?;
    let projection = project_program(&file).map_err(|_| "original projection")?;
    let source_end = u32::try_from(source.encode_utf16().count())?;
    for (input, expected) in [
        (
            diagnostic((0, 3), (0, 3))?,
            MappingError::InvalidUtf16Boundary,
        ),
        (
            diagnostic((99, 0), (99, 0))?,
            MappingError::InvalidUtf16Boundary,
        ),
        (diagnostic((0, 6), (0, 5))?, MappingError::InvalidRange),
        (diagnostic((1, 0), (1, 1))?, MappingError::GeneratedOnly),
        (
            diagnostic((0, source_end - 1), (1, 0))?,
            MappingError::CrossesBoundary,
        ),
    ] {
        let original = serde_json::to_value(&input)?;
        let observed = mapping::observe(&projection, input);
        require(
            observed.span() == Err(expected),
            "precise coordinate refusal",
        )?;
        require(
            observed.original_range().is_none(),
            "refusal has no fabricated authored range",
        )?;
        require(
            serde_json::to_value(observed.backend())? == original,
            "refused whole backend payload retained",
        )?;
    }
    Ok(())
}

#[test]
fn unrecorded_projection_and_authored_end_point_have_distinct_observations() -> TestResult {
    let source = "const value=1;";
    let arena = Allocator::default();
    let file = support::file(&arena, source, Lang::Ts).ok_or("original complete File")?;
    let plain = project_program_no_links(&file).map_err(|_| "unrecorded projection")?;
    require(
        mapping::observe(&plain, diagnostic((0, 0), (0, 0))?).span()
            == Err(MappingError::Unrecorded),
        "unrecorded projection explicitly refused",
    )?;
    let projection = project_program(&file).map_err(|_| "original projection")?;
    let end = u32::try_from(source.len())?;
    let observed = mapping::observe(&projection, diagnostic((0, end), (0, end))?);
    require(
        observed.span() == Ok(Span::new(end, end)),
        "authored endpoint remains valid",
    )?;
    require(
        observed.original_range().is_some(),
        "authored endpoint retains exact original range",
    )?;
    Ok(())
}
