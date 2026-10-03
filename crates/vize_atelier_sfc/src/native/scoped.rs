//! Scope only a complete original parser-proven simple-class rule.

use super::NativeSfcCompileError;
use vize_l0::Span;
use vize_l1::{container::vue::StyleView, css::StyleSyntax};
use vize_l4::{module::ScopeId, write::EmitDocument};

pub(super) fn append(
    original: StyleView<'_, '_>,
    syntax: &[StyleSyntax<'_>],
    scope: Option<ScopeId<'_>>,
    output: &mut EmitDocument,
    trim: bool,
) -> Result<(), NativeSfcCompileError> {
    let unavailable = || NativeSfcCompileError::StyleCompilationUnavailable {
        container_index: original.container_index(),
        span: original.block().span(),
    };
    if original
        .attrs()
        .iter()
        .any(|attr| attr.name != "scoped" && (attr.name != "lang" || attr.value != Some("css")))
    {
        return Err(unavailable());
    }
    let observed = syntax
        .iter()
        .find(|syntax| syntax.container_index() == original.container_index())
        .ok_or_else(unavailable)?;
    let receipt =
        observed
            .simple_class()
            .map_err(|issue| NativeSfcCompileError::ScopedStyleUnavailable {
                container_index: original.container_index(),
                issue,
            })?;
    let scope = scope.ok_or_else(unavailable)?;
    let block = receipt.syntax().source();
    let raw = block.source();
    let (text, prefix) = if trim {
        (raw.trim(), raw.len() - raw.trim_start().len())
    } else {
        (raw, 0)
    };
    let start = block.start() + prefix as u32;
    let end = start + text.len() as u32;
    let before_span = Span::new(start, receipt.insertion());
    let after_span = Span::new(receipt.insertion(), end);
    let before = block
        .root_source()
        .get(before_span.start as usize..before_span.end as usize)
        .ok_or_else(unavailable)?;
    let after = block
        .root_source()
        .get(after_span.start as usize..after_span.end as usize)
        .ok_or_else(unavailable)?;
    if !output.is_empty() {
        output.push_str("\n");
    }
    append_original(output, before, before_span);
    output.push_str("[");
    output.push_str(scope.as_str());
    output.push_str("]");
    append_original(output, after, after_span);
    Ok(())
}

fn append_original(output: &mut EmitDocument, text: &str, span: Span) {
    let mut at = span.start;
    for line in text.split_inclusive('\n') {
        let end = at + line.len() as u32;
        output.push_linked(line, Span::new(at, end));
        at = end;
    }
}
