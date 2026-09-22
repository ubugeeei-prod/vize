//! Same-file ecosystem editor helpers.

mod context;
pub(crate) mod i18n;
pub(crate) mod router;
mod router_project;
pub(crate) mod void;

#[cfg(test)]
mod router_extension_tests;
#[cfg(test)]
mod router_project_tests;

use tower_lsp::lsp_types::{
    CompletionItem, Diagnostic, DiagnosticSeverity, NumberOrString, Position, Range, Url,
};

use crate::ide::IdeContext;
use crate::virtual_code::BlockType;

pub(crate) fn completions(ctx: &IdeContext<'_>) -> Vec<CompletionItem> {
    if !matches!(
        ctx.block_type,
        Some(BlockType::Template | BlockType::Script | BlockType::ScriptSetup)
    ) {
        return Vec::new();
    }

    let Some(descriptor) = ctx.descriptor() else {
        return Vec::new();
    };

    let mut items = i18n::completions(ctx, descriptor);
    if items.is_empty() {
        items = router::completions(ctx, descriptor);
    }
    if items.is_empty() {
        items = void::completions(ctx);
    }
    items
}

/// Reuse the same revision that the diagnostics collector already parsed.
pub(crate) fn diagnostics(
    uri: &Url,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Vec<Diagnostic> {
    let content = descriptor.source.as_ref();
    let mut diagnostics = router::route_param_diagnostics(content, uri, descriptor);
    diagnostics.extend(i18n::missing_key_diagnostics(content, descriptor, uri));
    diagnostics
}

pub(crate) fn warning_diagnostic(
    range: Range,
    code: &str,
    message: impl Into<String>,
) -> Diagnostic {
    Diagnostic {
        range,
        severity: Some(DiagnosticSeverity::WARNING),
        code: Some(NumberOrString::String(code.to_string())),
        source: Some(String::from("vize/ecosystem")),
        message: message.into(),
        ..Default::default()
    }
}

pub(crate) fn position_in_range(pos: Position, range: Range) -> bool {
    if pos.line < range.start.line || pos.line > range.end.line {
        return false;
    }
    if pos.line == range.start.line && pos.character < range.start.character {
        return false;
    }
    if pos.line == range.end.line && pos.character > range.end.character {
        return false;
    }
    true
}

#[cfg(test)]
mod resident_tests;
