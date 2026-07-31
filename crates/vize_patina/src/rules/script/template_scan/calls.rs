//! One template expression, parsed with oxc, yielding its identifier-callee
//! calls.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, CallExpression, Expression};
use oxc_ast_visit::{Visit, walk::walk_call_expression};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};

/// A call the template performs, with a plain identifier callee.
pub(in crate::rules::script) struct TemplateCall<'a> {
    /// The callee identifier, e.g. `total` or `$emit`.
    pub(in crate::rules::script) callee: &'a str,
    /// The first argument, when it is a string literal. `None` for a dynamic
    /// argument (`$emit(name)`) or no arguments at all — a value only the
    /// runtime knows cannot be checked against a declared set or a casing rule.
    pub(in crate::rules::script) first_string_argument: Option<TemplateStringArgument<'a>>,
    /// Byte range of the call expression, relative to the template block.
    pub(in crate::rules::script) start: u32,
    /// Exclusive end of [`TemplateCall::start`].
    pub(in crate::rules::script) end: u32,
}

/// A string-literal argument, with the range of the literal itself so a rule
/// can report there rather than at the whole call.
pub(in crate::rules::script) struct TemplateStringArgument<'a> {
    pub(in crate::rules::script) value: &'a str,
    /// Byte range of the literal *including* its quotes, relative to the
    /// template block — matching how the script halves span a `StringLiteral`.
    pub(in crate::rules::script) start: u32,
    /// Exclusive end of [`TemplateStringArgument::start`].
    pub(in crate::rules::script) end: u32,
}

/// Call `visit` for every identifier-callee call in `source`, a single
/// template expression whose first byte sits at `base` in the template block.
///
/// A template expression may be a statement list rather than a single
/// expression (`@click="a(); b()"` is valid), so it is parsed as a program.
/// Sources oxc rejects are skipped: a call invented from a mis-parse would be a
/// false positive, and a template that does not compile has larger problems.
pub(super) fn for_each_call(source: &str, base: u32, visit: &mut impl FnMut(TemplateCall<'_>)) {
    // Cheap reject for the overwhelmingly common expression (`{{ msg }}`,
    // `:class="cls"`): no call syntax, nothing to parse.
    if !source.contains('(') {
        return;
    }
    let allocator = Allocator::default();
    let parsed = Parser::new(
        &allocator,
        source,
        SourceType::default().with_typescript(true),
    )
    .parse();
    if parsed.panicked || !parsed.errors.is_empty() {
        return;
    }
    let mut collector = CallCollector { calls: Vec::new() };
    collector.visit_program(&parsed.program);
    for call in collector.calls {
        visit(TemplateCall {
            callee: call.callee,
            first_string_argument: call.first_string_argument.map(|(value, span)| {
                TemplateStringArgument {
                    value,
                    start: base + span.start,
                    end: base + span.end,
                }
            }),
            start: base + call.span.start,
            end: base + call.span.end,
        });
    }
}

struct CollectedCall<'a> {
    callee: &'a str,
    first_string_argument: Option<(&'a str, Span)>,
    span: Span,
}

struct CallCollector<'a> {
    calls: Vec<CollectedCall<'a>>,
}

impl<'a> Visit<'a> for CallCollector<'a> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Only a plain identifier callee: a member call (`bus.emit('x')`,
        // `child.$emit('x')`) dispatches on another object, which is a
        // different thing entirely and is deliberately not tracked.
        if let Expression::Identifier(callee) = &it.callee {
            self.calls.push(CollectedCall {
                callee: callee.name.as_str(),
                first_string_argument: match it.arguments.first() {
                    Some(Argument::StringLiteral(literal)) => {
                        Some((literal.value.as_str(), literal.span))
                    }
                    _ => None,
                },
                span: it.span,
            });
        }
        // Keep walking: an argument can hold further calls (`a(b())`).
        walk_call_expression(self, it);
    }
}
