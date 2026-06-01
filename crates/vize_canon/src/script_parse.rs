use oxc_allocator::Allocator as OxcAllocator;
use oxc_parser::Parser as OxcParser;
use oxc_span::SourceType;
use vize_carton::{String, ToCompactString, profile};

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ScriptParseDiagnostic {
    pub message: String,
    pub start: u32,
    pub end: u32,
}

pub(crate) fn collect_script_parse_diagnostics(
    source: &str,
    source_offset: u32,
) -> Vec<ScriptParseDiagnostic> {
    let allocator = OxcAllocator::default();
    let source_type = SourceType::from_path("script.ts").unwrap_or_default();
    let parsed = profile!(
        "canon.script.parse_errors",
        OxcParser::new(&allocator, source, source_type).parse()
    );

    let mut diagnostics: Vec<ScriptParseDiagnostic> = parsed
        .errors
        .iter()
        .map(|error| {
            let (local_start, local_end) = diagnostic_span(error, source.len());
            ScriptParseDiagnostic {
                message: error.to_compact_string(),
                start: source_offset + local_start,
                end: source_offset + local_end,
            }
        })
        .collect();

    if parsed.panicked && diagnostics.is_empty() {
        let fallback_end = source_offset + (source.len() as u32).max(1);
        diagnostics.push(ScriptParseDiagnostic {
            message: "Parser panicked while parsing script".into(),
            start: source_offset,
            end: fallback_end,
        });
    }

    diagnostics
}

fn diagnostic_span(error: &oxc_diagnostics::OxcDiagnostic, source_len: usize) -> (u32, u32) {
    let fallback_end = source_len.max(1);
    let Some(label) = error.labels.as_ref().and_then(|labels| {
        labels
            .iter()
            .find(|label| label.primary())
            .or_else(|| labels.first())
    }) else {
        return (0, fallback_end as u32);
    };

    let start = label.offset().min(source_len);
    let end = start.saturating_add(label.len().max(1)).min(fallback_end);
    (start as u32, end.max(start + 1) as u32)
}
