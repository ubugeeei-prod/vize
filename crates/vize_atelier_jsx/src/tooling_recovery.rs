//! Preserve JSX expression spans while an editor is missing a member name.
//!
//! Recovery is only for analysis consumers. Compile entry points retain the
//! original parser verdict. The temporary identifier never enters emitted
//! code: every AST span is mapped back before lowering reads authored text.

use oxc_ast_visit::VisitMut;
use oxc_span::Span;
use vize_s0::Allocator;

use crate::{
    JsxLang, JsxOutputMode, LowerOutput, analyze, finder, lower::Lowerer, parse, span::SpanMapper,
};

/// Lower JSX for type checking and editor queries, retaining parser diagnostics
/// and all authored coordinates when a trailing member access is incomplete.
pub fn lower_source_for_typecheck<'a>(
    bump: &'a Allocator,
    allocator: &oxc_allocator::Allocator,
    source: &'a str,
    lang: JsxLang,
) -> LowerOutput<'a> {
    let mut output = crate::lower_source(bump, allocator, source, lang);
    if !output.roots.is_empty() || output.diagnostics.is_empty() {
        return output;
    }
    let Some(offset) = output
        .diagnostics
        .iter()
        .filter(|diagnostic| diagnostic.is_error())
        .flat_map(|diagnostic| [diagnostic.start as usize, diagnostic.end as usize])
        .find(|&offset| is_missing_member(source, offset))
    else {
        return output;
    };
    let prepared = parse::prepare_source_for_parse(source, lang);
    let mut recovered = prepared.into_owned();
    const MEMBER: &str = "__vize_incomplete_member";
    recovered.insert_str(offset, MEMBER);
    let mut parsed = parse::parse_module(allocator, &recovered, lang);
    if parsed.has_errors() {
        return output;
    }
    AuthoredSpans {
        insertion: offset as u32,
        length: MEMBER.len() as u32,
    }
    .visit_program(&mut parsed.program);
    let mapper = SpanMapper::new(source);
    let mut lowerer = Lowerer::new(bump, &mapper);
    output.roots = finder::lower_program_roots(&parsed.program, &mut lowerer, JsxOutputMode::Vdom);
    output.analysis = analyze::analyze_program(&parsed.program, source);
    output.diagnostics.extend(lowerer.into_diagnostics());
    output
}

fn is_missing_member(source: &str, offset: usize) -> bool {
    if !source.is_char_boundary(offset) {
        return false;
    }
    let prefix = source.get(..offset).unwrap_or_default().trim_end();
    prefix.ends_with('.')
        && !prefix.ends_with("...")
        && source
            .get(offset..)
            .unwrap_or_default()
            .chars()
            .next()
            .is_none_or(|ch| matches!(ch, '}' | ')' | ']' | ';' | ','))
}

struct AuthoredSpans {
    insertion: u32,
    length: u32,
}
impl AuthoredSpans {
    fn offset(&self, position: u32) -> u32 {
        position.min(self.insertion) + position.saturating_sub(self.insertion + self.length)
    }
}
impl<'a> VisitMut<'a> for AuthoredSpans {
    fn visit_span(&mut self, span: &mut Span) {
        span.start = self.offset(span.start);
        span.end = self.offset(span.end);
    }
}

#[cfg(test)]
mod tests;
