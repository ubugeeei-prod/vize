//! Scope a complete original parser-proven simple-class rule or class list.

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
    let (single, list) = match observed.simple_class() {
        Ok(receipt) => (Some(receipt), None),
        Err(_) => (
            None,
            Some(observed.simple_class_list().map_err(|issue| {
                NativeSfcCompileError::ScopedStyleUnavailable {
                    container_index: original.container_index(),
                    issue,
                }
            })?),
        ),
    };
    let scope = scope.ok_or_else(unavailable)?;
    let block = observed.source();
    let raw = block.source();
    let (text, prefix) = if trim {
        (raw.trim(), raw.len() - raw.trim_start().len())
    } else {
        (raw, 0)
    };
    let start = block.start() + prefix as u32;
    let end = start + text.len() as u32;
    block
        .root_source()
        .get(start as usize..end as usize)
        .ok_or_else(unavailable)?;
    if !output.is_empty() {
        output.push_str("\n");
    }
    let mut cursor = start;
    let insertions = single
        .into_iter()
        .map(|receipt| receipt.insertion())
        .chain(list.into_iter().flat_map(|receipt| receipt.insertions()));
    for insertion in insertions {
        let before_span = Span::new(cursor, insertion);
        let before = block
            .root_source()
            .get(before_span.start as usize..before_span.end as usize)
            .ok_or_else(unavailable)?;
        append_original(output, before, before_span);
        output.push_str("[");
        output.push_str(scope.as_str());
        output.push_str("]");
        cursor = insertion;
    }
    let after_span = Span::new(cursor, end);
    let after = block
        .root_source()
        .get(after_span.start as usize..after_span.end as usize)
        .ok_or_else(unavailable)?;
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
