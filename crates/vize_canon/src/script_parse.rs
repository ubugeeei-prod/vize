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
    lang: Option<&str>,
) -> Vec<ScriptParseDiagnostic> {
    let allocator = OxcAllocator::default();
    let source_type = script_source_type(lang);
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

/// Resolve the oxc [`SourceType`] for an SFC `<script>` block's `lang`.
///
/// `<script lang="tsx">` / `<script lang="jsx">` must parse with JSX enabled so
/// embedded JSX (a Vue JSX/TSX render function in a `.vue` script block) is
/// accepted rather than reported as a spurious parse error — which would
/// otherwise collapse the whole SFC to the typed fallback stub and silently drop
/// type-checking of the script body (#1498). Every other `lang` (the absent /
/// empty / `ts` / `js` / unknown case) keeps the prior plain-TypeScript dialect
/// unchanged, so an accidental `<` in a non-JSX script still surfaces as an
/// error and existing SFCs parse byte-identically.
///
/// The JSX dialects stay TypeScript-flavored (`tsx()` keeps TS syntax; `jsx()`
/// is JS-flavored) to match how the Vize JSX virtual-TS lowering re-emits the
/// script body, and mirrors `vize_maestro`'s `script_source_type` so the LSP
/// single-document parse-diagnostic lane and this batch lane agree.
fn script_source_type(lang: Option<&str>) -> SourceType {
    match lang.map(|value| value.trim()) {
        Some("tsx") => SourceType::tsx(),
        Some("jsx") => SourceType::jsx(),
        _ => SourceType::ts(),
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    // A Vue JSX render function inside a `<script lang="tsx">` block. Parsing it
    // as plain TypeScript would reject the `<button>` as a syntax error; the
    // `tsx` dialect must accept it. (#1498)
    const JSX_BODY: &str = "const label: string = 'hi';\nconst vnode = <button>{label}</button>;\n";

    #[test]
    fn tsx_lang_accepts_embedded_jsx_without_parse_error() {
        let diagnostics = collect_script_parse_diagnostics(JSX_BODY, 0, Some("tsx"));
        assert_eq!(
            diagnostics,
            vec![],
            "tsx script with JSX must parse cleanly, got: {diagnostics:?}"
        );
    }

    #[test]
    fn jsx_lang_accepts_embedded_jsx_without_parse_error() {
        // `jsx` blocks carry no type annotations; drop the `: string`.
        let body = "const label = 'hi';\nconst vnode = <button>{label}</button>;\n";
        let diagnostics = collect_script_parse_diagnostics(body, 0, Some("jsx"));
        assert_eq!(
            diagnostics,
            vec![],
            "jsx script with JSX must parse cleanly, got: {diagnostics:?}"
        );
    }

    #[test]
    fn plain_ts_lang_still_rejects_jsx() {
        // The default dialect stays non-JSX so a stray `<` in a plain TS script
        // is still surfaced rather than silently accepted. A `ts` block, an
        // absent lang, and an unknown lang all keep the prior behavior.
        for lang in [Some("ts"), None, Some("scss")] {
            let diagnostics = collect_script_parse_diagnostics(JSX_BODY, 0, lang);
            assert!(
                !diagnostics.is_empty(),
                "non-JSX dialect (lang={lang:?}) must reject embedded JSX"
            );
        }
    }

    #[test]
    fn source_offset_is_added_to_diagnostic_range() {
        // Offsets are relative to the SFC file: the block offset must shift the
        // reported range. Compare the same broken script at offset 0 vs 100.
        let broken = "const x: = 1;\n";
        let base = collect_script_parse_diagnostics(broken, 0, Some("ts"));
        let shifted = collect_script_parse_diagnostics(broken, 100, Some("ts"));
        assert_eq!(base.len(), shifted.len());
        assert!(!base.is_empty());
        assert_eq!(shifted[0].start, base[0].start + 100);
        assert_eq!(shifted[0].end, base[0].end + 100);
    }

    #[test]
    fn source_type_selects_jsx_dialect_only_for_jsx_langs() {
        assert!(script_source_type(Some("tsx")).is_jsx());
        assert!(script_source_type(Some("jsx")).is_jsx());
        assert!(script_source_type(Some("tsx")).is_typescript());
        assert!(!script_source_type(Some("jsx")).is_typescript());

        // Non-JSX dialects: TypeScript, no JSX.
        assert!(script_source_type(Some("ts")).is_typescript());
        assert!(!script_source_type(Some("ts")).is_jsx());
        assert!(!script_source_type(None).is_jsx());
        assert!(!script_source_type(Some("scss")).is_jsx());
        // Whitespace around the lang is tolerated.
        assert!(script_source_type(Some("  tsx  ")).is_jsx());
    }
}
