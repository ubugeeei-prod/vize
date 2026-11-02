//! Authored `.ts` syntax that TSX would read as JSX.
//!
//! A Content Mapper module is parsed as TSX, where `<T>(value: T) => value`
//! and `<T>value` open JSX elements. A lone arrow type parameter gains an
//! unmapped `,` (`<T,>`); an angle-bracket assertion becomes `((value) as T)`.
//! Rows are split so the moved expression and type keep their authored bytes;
//! the inner `(` starts the assertion node and reports it from the authored `<`.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::{ArrowFunctionExpression, TSTypeAssertion};
use oxc_ast_visit::{Visit, walk};
use oxc_parser::Parser;
use oxc_span::{GetSpan, SourceType, Span};
use vize_atelier_sfc::SfcDescriptor;
use vize_carton::String as CompactString;

use crate::virtual_ts::diagnostic_ownership::DiagnosticOwnershipCollector;
use crate::virtual_ts::{ProjectionMeta, VizeMapping, VizeSemanticLink};

mod rewrite;
use rewrite::Rewrite;

/// Every generated coordinate family that must follow the rewritten text.
pub(super) struct Coordinates<'a> {
    pub(super) mappings: &'a mut Vec<VizeMapping>,
    pub(super) mapping_meta: &'a mut Vec<ProjectionMeta>,
    pub(super) semantic_links: &'a mut [VizeSemanticLink],
    pub(super) prop_default_key_links: &'a mut [VizeSemanticLink],
    pub(super) diagnostic_ownership: Option<&'a DiagnosticOwnershipCollector>,
}

/// TypeScript blocks without JSX of their own. A JSX block keeps its syntax.
pub(super) fn applies(descriptor: &SfcDescriptor<'_>) -> bool {
    let mut typescript = false;
    for block in [descriptor.script.as_ref(), descriptor.script_setup.as_ref()]
        .into_iter()
        .flatten()
    {
        match block.lang.as_deref() {
            Some("tsx" | "jsx") => return false,
            Some(lang) if crate::script_parse::is_typescript_lang(lang) => typescript = true,
            _ => {}
        }
    }
    typescript
}

pub(super) fn rewrite(code: &mut CompactString, coordinates: Coordinates<'_>) {
    if !coordinates
        .mappings
        .iter()
        .any(|row| may_open_expression(code, &row.gen_range))
    {
        return;
    }
    let edits = collect_edits(code);
    if edits.is_empty() {
        return;
    }
    let Some(rewrite) = Rewrite::build(code, &edits) else {
        return;
    };
    rewrite.apply(code, coordinates);
}

/// Cheap gate: a `<` in authored bytes that could start an expression.
/// Over-approximates; the AST decides.
fn may_open_expression(code: &str, row: &Range<usize>) -> bool {
    let bytes = code.as_bytes();
    let Some(slice) = bytes.get(row.clone()) else {
        return false;
    };
    slice.iter().enumerate().any(|(index, &byte)| {
        byte == b'<'
            && opens_expression(bytes, row.start + index)
            && !tsx_safe_parameters(bytes.get(row.start + index + 1..).unwrap_or_default())
    })
}

/// `<T,` and `<T =` already read as type parameters in TSX. `extends` is left
/// to the AST: `<T extends U ? A : B>value` is an assertion.
fn tsx_safe_parameters(after: &[u8]) -> bool {
    let rest = after.trim_ascii_start();
    let rest = rest
        .strip_prefix(b"const")
        .filter(|rest| rest.first().is_some_and(u8::is_ascii_whitespace))
        .map_or(rest, <[u8]>::trim_ascii_start);
    let name = rest
        .iter()
        .take_while(|&&byte| is_identifier_byte(byte))
        .count();
    name > 0
        && rest
            .get(name..)
            .map(<[u8]>::trim_ascii_start)
            .is_some_and(|rest| match rest {
                [b',', ..] => true,
                [b'=', next, ..] => *next != b'>',
                _ => false,
            })
}

fn opens_expression(bytes: &[u8], at: usize) -> bool {
    let mut end = at;
    while end
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
    {
        end -= 1;
    }
    let Some(&previous) = end.checked_sub(1).and_then(|index| bytes.get(index)) else {
        return true;
    };
    if matches!(previous, b'"' | b'\'' | b'`' | b']') {
        return false;
    }
    if !is_identifier_byte(previous) {
        return true;
    }
    let mut start = end;
    while start
        .checked_sub(1)
        .and_then(|index| bytes.get(index))
        .is_some_and(|&byte| is_identifier_byte(byte))
    {
        start -= 1;
    }
    bytes
        .get(start..end)
        .is_some_and(|word| EXPRESSION_KEYWORDS.contains(&word))
}

/// Keywords an expression, and so an assertion, can follow.
const EXPRESSION_KEYWORDS: &[&[u8]] = &[
    b"return",
    b"typeof",
    b"void",
    b"delete",
    b"await",
    b"yield",
    b"case",
    b"throw",
    b"in",
    b"of",
    b"instanceof",
    b"new",
    b"else",
    b"do",
    b"async",
    b"default",
];

const fn is_identifier_byte(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$') || byte >= 0x80
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Edit {
    /// `<T>(` becomes `<T,>(`.
    Comma(u32),
    /// `<T>value` becomes `((value) as T)`.
    Assertion {
        whole: Span,
        ty: Span,
        expression: Span,
    },
}

impl Edit {
    const fn start(self) -> u32 {
        match self {
            Self::Comma(at) => at,
            Self::Assertion { whole, .. } => whole.start,
        }
    }
}

fn collect_edits(code: &str) -> Vec<Edit> {
    let allocator = Allocator::default();
    let parsed = Parser::new(&allocator, code, SourceType::ts()).parse();
    let mut collector = Collector {
        code,
        edits: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    collector.edits.sort_by_key(|edit| edit.start());
    collector.edits
}

struct Collector<'s> {
    code: &'s str,
    edits: Vec<Edit>,
}

impl<'a> Visit<'a> for Collector<'_> {
    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        if let Some(declaration) = it.type_parameters.as_deref()
            && let [parameter] = declaration.params.as_slice()
            && parameter.constraint.is_none()
            && parameter.default.is_none()
            && !comma_follows(self.code, parameter.span.end, declaration.span.end)
        {
            self.edits.push(Edit::Comma(parameter.name.span.end));
        }
        walk::walk_arrow_function_expression(self, it);
    }

    fn visit_ts_type_assertion(&mut self, it: &TSTypeAssertion<'a>) {
        self.edits.push(Edit::Assertion {
            whole: it.span,
            ty: it.type_annotation.span(),
            expression: it.expression.span(),
        });
        walk::walk_ts_type_assertion(self, it);
    }
}

/// Whether `,` is the next token in `code[from..to]`.
fn comma_follows(code: &str, from: u32, to: u32) -> bool {
    let Some(mut rest) = code.get(from as usize..to as usize) else {
        return true;
    };
    loop {
        rest = rest.trim_start();
        if let Some(comment) = rest.strip_prefix("/*") {
            rest = comment.split_once("*/").map_or("", |(_, after)| after);
        } else if let Some(comment) = rest.strip_prefix("//") {
            rest = comment.split_once('\n').map_or("", |(_, after)| after);
        } else {
            return rest.starts_with(',');
        }
    }
}
