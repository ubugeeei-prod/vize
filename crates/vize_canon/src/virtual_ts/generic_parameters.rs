//! One parser-owned description of an SFC's authored type parameters.
//!
//! Byte ranges are relative to the attribute value. The AST is arena-local;
//! only owned spellings and relative ranges escape preparation.

use std::ops::Range;

use oxc_allocator::Allocator;
use oxc_ast::ast::Statement;
use oxc_parser::Parser;
use oxc_semantic::SemanticBuilder;
use oxc_span::{GetSpan, SourceType, Span};
use vize_carton::{String, ToCompactString, cstr, profile};

use crate::script_parse::ScriptParseDiagnostic;

pub(crate) type GenericRow = (Range<usize>, Range<usize>);

#[cfg(test)]
fn preparation_count(increment: bool) -> usize {
    thread_local! {
        static COUNT: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
    }
    COUNT.with(|count| {
        if increment {
            count.set(count.get() + 1);
        }
        count.get()
    })
}

const PREFIX: &str = "function __vize_generic<";

#[derive(Debug)]
pub(crate) struct GenericParameter {
    pub(crate) span: Range<usize>,
    pub(crate) core: Range<usize>,
    pub(crate) default: Option<Range<usize>>,
    pub(crate) constraint: Option<Range<usize>>,
    pub(crate) name: String,
    pub(crate) is_const: bool,
    dependent_constraint: bool,
}

/// Prepared once, then borrowed by every projection consumer. Invalid input
/// retains its authored spelling so native TypeScript still diagnoses it.
#[derive(Debug)]
pub(crate) struct GenericParameters {
    raw: String,
    trailing_line_comment: bool,
    pub(crate) parameters: Vec<GenericParameter>,
    pub(crate) diagnostics: Vec<ScriptParseDiagnostic>,
    pub(crate) escaped_wrapper: bool,
    pub(crate) recoverable_at_closing_delimiter: bool,
    pub(crate) names: String,
    pub(crate) declaration: String,
    pub(crate) alias_declaration: String,
    pub(crate) alias_authored: String,
    pub(crate) fallback: String,
    authored: String,
    raw_rows: Vec<GenericRow>,
    declaration_rows: Vec<GenericRow>,
    alias_declaration_rows: Vec<GenericRow>,
    alias_authored_rows: Vec<GenericRow>,
}

impl GenericParameters {
    pub(crate) fn parse(raw: &str) -> Self {
        #[cfg(test)]
        preparation_count(true);
        profile!(
            "canon.virtual_ts.parse_generic_parameters",
            Self::prepare(raw)
        )
    }

    fn prepare(raw: &str) -> Self {
        let source = cstr!("{PREFIX}{raw}\n>() {{}}");
        let allocator = Allocator::default();
        let parsed = Parser::new(&allocator, &source, SourceType::ts()).parse();
        let closing = PREFIX.len() + raw.len() + 1;
        let recoverable_at_closing_delimiter = !parsed.diagnostics.is_empty()
            && parsed.diagnostics.iter().all(|error| {
                error
                    .labels
                    .iter()
                    .find(|label| label.primary())
                    .or_else(|| error.labels.first())
                    .is_some_and(|label| label.offset() as usize == closing && label.len() == 1)
            });
        let mut diagnostics: Vec<_> = parsed
            .diagnostics
            .iter()
            .map(|error| {
                let label = error
                    .labels
                    .iter()
                    .find(|label| label.primary())
                    .or_else(|| error.labels.first());
                let start = label
                    .map_or(0, |label| {
                        (label.offset() as usize).saturating_sub(PREFIX.len())
                    })
                    .min(raw.len());
                let end = label
                    .map_or(raw.len(), |label| {
                        (label.offset() as usize)
                            .saturating_add(label.len().max(1) as usize)
                            .saturating_sub(PREFIX.len())
                    })
                    .min(raw.len());
                ScriptParseDiagnostic {
                    message: error.to_compact_string(),
                    start: start as u32,
                    end: end.max(start) as u32,
                }
            })
            .collect();
        if parsed.panicked && diagnostics.is_empty() {
            diagnostics.push(ScriptParseDiagnostic {
                message: "Parser could not recover generic parameters".into(),
                start: 0,
                end: raw.len() as u32,
            });
        }
        let semantic = SemanticBuilder::new()
            .with_build_nodes(true)
            .build(&parsed.program)
            .semantic;
        let declaration = match parsed.program.body.first() {
            Some(Statement::FunctionDeclaration(function)) => function.type_parameters.as_deref(),
            _ => None,
        };
        let escaped_wrapper = diagnostics.is_empty()
            && match (parsed.program.body.first(), declaration) {
                (Some(Statement::FunctionDeclaration(function)), Some(parameters)) => {
                    parsed.program.body.len() != 1
                        || function.span.end as usize != source.len()
                        || parameters.span.start as usize != PREFIX.len() - 1
                        || parameters.span.end as usize != PREFIX.len() + raw.len() + 2
                }
                _ => diagnostics.is_empty(),
            };
        if escaped_wrapper && diagnostics.is_empty() {
            diagnostics.push(ScriptParseDiagnostic {
                message: "Generic attribute must contain one complete type-parameter list".into(),
                start: 0,
                end: raw.len() as u32,
            });
        }
        let relative = |span: Span| {
            (span.start as usize)
                .saturating_sub(PREFIX.len())
                .min(raw.len())
                ..(span.end as usize)
                    .saturating_sub(PREFIX.len())
                    .min(raw.len())
        };
        let symbols: Vec<_> = declaration
            .into_iter()
            .flat_map(|declaration| &declaration.params)
            .filter_map(|parameter| parameter.name.symbol_id.get())
            .collect();
        let parameters: Vec<_> = declaration
            .into_iter()
            .flat_map(|declaration| &declaration.params)
            .map(|parameter| {
                let constraint = parameter.constraint.as_ref().map(GetSpan::span);
                let core_end = constraint.map_or(parameter.name.span.end, |span| span.end);
                let dependent_constraint = constraint.is_some_and(|constraint| {
                    symbols.iter().any(|symbol| {
                        semantic
                            .scoping()
                            .get_resolved_references(*symbol)
                            .any(|reference| {
                                let span = semantic.nodes().get_node(reference.node_id()).span();
                                constraint.start <= span.start && span.end <= constraint.end
                            })
                    })
                });
                GenericParameter {
                    span: relative(parameter.span),
                    core: relative(Span::new(parameter.name.span.start, core_end)),
                    default: parameter
                        .default
                        .as_ref()
                        .map(|value| relative(value.span())),
                    constraint: constraint.map(relative),
                    name: parameter.name.name.as_str().into(),
                    is_const: parameter.r#const,
                    dependent_constraint,
                }
            })
            .collect();
        let mut plan = Self {
            raw: raw.into(),
            trailing_line_comment: parsed.program.comments.iter().any(|comment| {
                matches!(comment.kind, oxc_ast::ast::CommentKind::Line)
                    && comment.span.end as usize == PREFIX.len() + raw.len()
            }),
            parameters,
            diagnostics,
            escaped_wrapper,
            recoverable_at_closing_delimiter,
            names: String::default(),
            declaration: String::default(),
            alias_declaration: String::default(),
            alias_authored: String::default(),
            fallback: String::default(),
            authored: raw.into(),
            raw_rows: Vec::new(),
            declaration_rows: Vec::new(),
            alias_declaration_rows: Vec::new(),
            alias_authored_rows: Vec::new(),
        };
        if plan.parameters.is_empty() {
            // An invalid signature must never become an absent/clean generic.
            plan.names = raw.into();
            plan.fallback = raw.into();
        } else {
            for parameter in &plan.parameters {
                if !plan.names.is_empty() {
                    plan.names.push_str(", ");
                    plan.fallback.push_str(", ");
                }
                plan.names.push_str(&parameter.name);
                let fallback = if parameter.dependent_constraint {
                    "any"
                } else {
                    parameter
                        .constraint
                        .as_ref()
                        .and_then(|span| raw.get(span.clone()))
                        .unwrap_or("unknown")
                };
                plan.fallback.push_str(fallback);
            }
        }
        if plan.trailing_line_comment {
            plan.authored.push('\n');
        }
        plan.raw_rows = if plan.diagnostics.is_empty() {
            plan.parameters
                .iter()
                .flat_map(|parameter| {
                    std::iter::once(parameter.core.clone()).chain(parameter.default.iter().cloned())
                })
                .map(|range| (range.clone(), range))
                .collect()
        } else {
            vec![(0..raw.len(), 0..raw.len())]
        };
        (plan.declaration, plan.declaration_rows) = plan.render(false, true);
        (plan.alias_declaration, plan.alias_declaration_rows) = plan.render(true, true);
        (plan.alias_authored, plan.alias_authored_rows) = plan.render(true, false);
        plan
    }

    #[cfg(any(feature = "native", test))]
    pub(crate) fn raw(&self) -> &str {
        &self.raw
    }

    pub(crate) fn authored(&self) -> &str {
        &self.authored
    }

    pub(crate) fn declaration_variants(&self) -> [(&str, &[GenericRow]); 4] {
        [
            (&self.authored, &self.raw_rows),
            (&self.declaration, &self.declaration_rows),
            (&self.alias_declaration, &self.alias_declaration_rows),
            (&self.alias_authored, &self.alias_authored_rows),
        ]
    }

    pub(crate) fn has_repeated_fallback_constraint(&self) -> bool {
        self.parameters
            .iter()
            .any(|parameter| parameter.constraint.is_some() && !parameter.dependent_constraint)
    }

    pub(crate) fn parameter_names(&self) -> impl Iterator<Item = &str> {
        self.parameters
            .iter()
            .map(|parameter| parameter.name.as_str())
    }

    fn render(&self, drop_const: bool, add_defaults: bool) -> (String, Vec<GenericRow>) {
        if !self.diagnostics.is_empty() || self.parameters.is_empty() {
            return (self.authored.clone(), self.raw_rows.clone());
        }
        let mut output = String::default();
        let mut rows = Vec::new();
        let mut previous = 0;
        for (index, parameter) in self.parameters.iter().enumerate() {
            let gap = self
                .raw
                .get(previous..parameter.span.start)
                .unwrap_or_default();
            if index != 0 && gap.chars().all(|ch| ch.is_whitespace() || ch == ',') {
                output.push_str(", ");
            } else if !gap.trim().is_empty() {
                output.push_str(gap);
            }
            let start = if drop_const && parameter.is_const {
                let after_const = parameter.span.start + "const".len();
                let rest = self
                    .raw
                    .get(after_const..parameter.span.end)
                    .unwrap_or_default();
                after_const + rest.len() - rest.trim_start().len()
            } else {
                parameter.span.start
            };
            let generated = output.len();
            output.push_str(self.raw.get(start..parameter.span.end).unwrap_or_default());
            for original in std::iter::once(&parameter.core).chain(parameter.default.iter()) {
                let from = generated + original.start.saturating_sub(start);
                rows.push((from..from + original.len(), original.clone()));
            }
            if add_defaults && parameter.default.is_none() {
                output.push_str(" = any");
            }
            previous = parameter.span.end;
        }
        let tail = self.raw.get(previous..).unwrap_or_default();
        if !tail.trim().is_empty() {
            output.push_str(tail);
        }
        // A final authored line comment must not consume the generated `>`.
        if self.trailing_line_comment {
            output.push('\n');
        }
        (output, rows)
    }
}

/// The quoted value of `name="..."` inside an opening tag. Attribute values
/// are skipped whole so a different attribute's text cannot supply `name`.
pub(crate) fn attribute_value(source: &str, tag: Range<usize>, name: &str) -> Option<Range<usize>> {
    let text = source.get(tag.clone())?;
    let bytes = text.as_bytes();
    let mut cursor = 0;
    while bytes
        .get(cursor)
        .is_some_and(|byte| !byte.is_ascii_whitespace())
    {
        cursor += 1;
    }
    while cursor < bytes.len() {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes
            .get(cursor)
            .is_none_or(|byte| matches!(byte, b'/' | b'>'))
        {
            return None;
        }
        let start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'/' | b'>'))
        {
            cursor += 1;
        }
        let attribute = text.get(start..cursor)?;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let quote = *bytes.get(cursor)?;
        if !matches!(quote, b'"' | b'\'') {
            while bytes
                .get(cursor)
                .is_some_and(|byte| !byte.is_ascii_whitespace() && *byte != b'>')
            {
                cursor += 1;
            }
            continue;
        }
        cursor += 1;
        let value_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| *byte != quote) {
            cursor += 1;
        }
        bytes.get(cursor)?;
        if attribute == name {
            return Some(tag.start + value_start..tag.start + cursor);
        }
        cursor += 1;
    }
    None
}

/// The exact authored attribute range used for generic diagnostics.
pub(crate) fn authored_generic_range(
    source: &str,
    descriptor: &vize_atelier_sfc::SfcDescriptor<'_>,
) -> Option<Range<usize>> {
    let block = descriptor.script_setup.as_ref()?;
    attribute_value(source, block.loc.tag_start..block.loc.start, "generic")
}

#[cfg(test)]
mod tests;

#[cfg(all(test, feature = "native"))]
mod recovery_tests;
