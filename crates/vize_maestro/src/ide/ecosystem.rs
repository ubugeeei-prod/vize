//! Same-file ecosystem editor helpers.

mod context;
pub(crate) mod i18n;
pub(crate) mod router;
pub(crate) mod void;

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

    let options = vize_atelier_sfc::SfcParseOptions {
        filename: ctx.uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(&ctx.content, options) else {
        return Vec::new();
    };

    let mut items = i18n::completions(ctx, &descriptor);
    if items.is_empty() {
        items = router::completions(ctx, &descriptor);
    }
    if items.is_empty() {
        items = void::completions(ctx);
    }
    items
}

pub(crate) fn diagnostics(content: &str, uri: &Url) -> Vec<Diagnostic> {
    let options = vize_atelier_sfc::SfcParseOptions {
        filename: uri.path().to_string().into(),
        ..Default::default()
    };
    let Ok(descriptor) = vize_atelier_sfc::parse_sfc(content, options) else {
        return Vec::new();
    };

    let mut diagnostics = router::route_param_diagnostics(content, uri);
    diagnostics.extend(i18n::missing_key_diagnostics(content, &descriptor, uri));
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
