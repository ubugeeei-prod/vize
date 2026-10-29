//! One template expression, parsed with oxc, yielding its identifier-callee
//! calls.

use oxc_allocator::Allocator;
use oxc_ast::ast::{Argument, ArrowFunctionExpression, BindingPattern, CallExpression, Expression};
use oxc_ast_visit::{
    Visit,
    walk::{walk_arrow_function_expression, walk_call_expression},
};
use oxc_parser::Parser;
use oxc_span::{SourceType, Span};
use vize_armature::patterns::attribute_source_span;

/// A call the template performs, with a plain identifier callee.
pub(in crate::rules::script) struct TemplateCall<'a> {
    /// The callee identifier, e.g. `total` or `$emit`.
    pub(in crate::rules::script) callee: &'a str,
    /// The first argument, when it is a string literal or a template literal
    /// without substitutions. `None` for a dynamic
    /// argument (`$emit(name)`) or no arguments at all — a value only the
    /// runtime knows cannot be checked against a declared set or a casing rule.
    pub(in crate::rules::script) first_string_argument: Option<TemplateStringArgument<'a>>,
    /// Byte range of the callee identifier, relative to the template block.
    pub(in crate::rules::script) callee_start: u32,
    /// Exclusive end of [`TemplateCall::callee_start`].
    pub(in crate::rules::script) callee_end: u32,
    /// Byte range of the call expression, relative to the template block.
    pub(in crate::rules::script) start: u32,
    /// Exclusive end of [`TemplateCall::start`].
    pub(in crate::rules::script) end: u32,
    /// Arrow-function parameters whose scope contains this call.
    local_bindings: Vec<&'a str>,
}

impl TemplateCall<'_> {
    pub(super) fn is_locally_shadowed(&self, name: &str) -> bool {
        self.local_bindings.contains(&name)
    }
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
pub(super) fn for_each_call(
    source: &str,
    authored: &str,
    base: u32,
    visit: &mut impl FnMut(TemplateCall<'_>),
) {
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
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return;
    }
    let mut collector = CallCollector {
        calls: Vec::new(),
        local_bindings: Vec::new(),
    };
    collector.visit_program(&parsed.program);
    for call in collector.calls {
        let Some(callee_span) = attribute_source_span(authored, source, call.callee_span) else {
            continue;
        };
        let Some(span) = attribute_source_span(authored, source, call.span) else {
            continue;
        };
        visit(TemplateCall {
            callee: call.callee,
            first_string_argument: call.first_string_argument.and_then(|(value, span)| {
                attribute_source_span(authored, source, span).map(|span| TemplateStringArgument {
                    value,
                    start: base + span.start,
                    end: base + span.end,
                })
            }),
            callee_start: base + callee_span.start,
            callee_end: base + callee_span.end,
            start: base + span.start,
            end: base + span.end,
            local_bindings: call.local_bindings,
        });
    }
}

struct CollectedCall<'a> {
    callee: &'a str,
    callee_span: Span,
    first_string_argument: Option<(&'a str, Span)>,
    span: Span,
    local_bindings: Vec<&'a str>,
}

struct CallCollector<'a> {
    calls: Vec<CollectedCall<'a>>,
    local_bindings: Vec<&'a str>,
}

impl<'a> Visit<'a> for CallCollector<'a> {
    fn visit_call_expression(&mut self, it: &CallExpression<'a>) {
        // Only a plain identifier callee: a member call (`bus.emit('x')`,
        // `child.$emit('x')`) dispatches on another object, which is a
        // different thing entirely and is deliberately not tracked.
        if let Expression::Identifier(callee) = &it.callee {
            self.calls.push(CollectedCall {
                callee: callee.name.as_str(),
                callee_span: callee.span,
                first_string_argument: match it.arguments.first() {
                    Some(Argument::StringLiteral(literal)) => {
                        Some((literal.value.as_str(), literal.span))
                    }
                    Some(Argument::TemplateLiteral(literal)) if literal.expressions.is_empty() => {
                        literal
                            .single_quasi()
                            .map(|quasi| (quasi.as_str(), literal.span))
                    }
                    _ => None,
                },
                span: it.span,
                local_bindings: self.local_bindings.clone(),
            });
        }
        // Keep walking: an argument can hold further calls (`a(b())`).
        walk_call_expression(self, it);
    }

    fn visit_arrow_function_expression(&mut self, it: &ArrowFunctionExpression<'a>) {
        let depth = self.local_bindings.len();
        for parameter in &it.params.items {
            collect_bindings(&parameter.pattern, &mut self.local_bindings);
        }
        if let Some(rest) = &it.params.rest {
            collect_bindings(&rest.rest.argument, &mut self.local_bindings);
        }
        walk_arrow_function_expression(self, it);
        self.local_bindings.truncate(depth);
    }
}

fn collect_bindings<'a>(pattern: &BindingPattern<'a>, out: &mut Vec<&'a str>) {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => out.push(identifier.name.as_str()),
        BindingPattern::ObjectPattern(object) => {
            for property in &object.properties {
                collect_bindings(&property.value, out);
            }
            if let Some(rest) = &object.rest {
                collect_bindings(&rest.argument, out);
            }
        }
        BindingPattern::ArrayPattern(array) => {
            for element in array.elements.iter().flatten() {
                collect_bindings(element, out);
            }
            if let Some(rest) = &array.rest {
                collect_bindings(&rest.argument, out);
            }
        }
        BindingPattern::AssignmentPattern(default) => collect_bindings(&default.left, out),
    }
}
