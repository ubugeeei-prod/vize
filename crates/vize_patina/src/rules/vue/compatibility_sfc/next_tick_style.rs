//! Promise-style `nextTick` usage.

use super::{absolute_range, collect_script_programs};
use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::ir::ByteRange;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{CallExpression, Expression, ImportDeclaration, ImportDeclarationSpecifier};
use oxc_ast_visit::{Visit, walk};
use oxc_span::{GetSpan, Span};
use vize_l0::{CompactString, FxHashSet};

static META: RuleMeta = RuleMeta {
    name: "vue/next-tick-style",
    description: "Enforce Promise-style nextTick usage",
    category: RuleCategory::Recommended,
    fixable: false,
    default_severity: Severity::Warning,
};

/// `vue/next-tick-style`; the upstream default is Promise style.
pub struct NextTickStyle;

impl Rule for NextTickStyle {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_sfc<'a>(&self, ctx: &mut LintContext<'a>) {
        let mut findings = Vec::new();
        collect_script_programs(ctx, |program, offset, source| {
            let mut visitor = NextTickCalls {
                offset,
                source,
                findings: Vec::new(),
                imported_aliases: FxHashSet::default(),
                namespace_aliases: FxHashSet::default(),
            };
            visitor.visit_program(program);
            findings.extend(visitor.findings);
        });
        for finding in findings {
            ctx.warn_at_with_help(
                "Use the Promise returned by `nextTick`",
                finding,
                "Await `nextTick()` or chain `.then(...)` instead of passing a callback",
            );
        }
    }
}

struct NextTickCalls<'s> {
    offset: u32,
    source: &'s str,
    findings: Vec<ByteRange>,
    imported_aliases: FxHashSet<CompactString>,
    namespace_aliases: FxHashSet<CompactString>,
}

impl<'a> Visit<'a> for NextTickCalls<'_> {
    fn visit_import_declaration(&mut self, import: &ImportDeclaration<'a>) {
        if import.source.value == "vue"
            && let Some(specifiers) = &import.specifiers
        {
            for specifier in specifiers {
                match specifier {
                    ImportDeclarationSpecifier::ImportSpecifier(specifier)
                        if specifier.imported.name() == "nextTick" =>
                    {
                        self.imported_aliases
                            .insert(CompactString::from(specifier.local.name.as_str()));
                    }
                    ImportDeclarationSpecifier::ImportNamespaceSpecifier(specifier) => {
                        self.namespace_aliases
                            .insert(CompactString::from(specifier.local.name.as_str()));
                    }
                    _ => {}
                }
            }
        }
        walk::walk_import_declaration(self, import);
    }

    fn visit_call_expression(&mut self, call: &CallExpression<'a>) {
        if is_next_tick_call(call, &self.imported_aliases, &self.namespace_aliases)
            && (!call.arguments.is_empty() || !promise_is_consumed(self.source, call.span))
        {
            self.findings
                .push(absolute_range(self.offset, call.callee.span()));
        }
        walk::walk_call_expression(self, call);
    }
}

fn is_next_tick_call(
    call: &CallExpression<'_>,
    imported_aliases: &FxHashSet<CompactString>,
    namespace_aliases: &FxHashSet<CompactString>,
) -> bool {
    match call.callee.get_inner_expression() {
        Expression::Identifier(identifier) => imported_aliases.contains(identifier.name.as_str()),
        Expression::StaticMemberExpression(member) => {
            member.property.name == "$nextTick"
                || (member.property.name == "nextTick"
                    && matches!(
                        member.object.get_inner_expression(),
                        Expression::Identifier(identifier)
                            if namespace_aliases.contains(identifier.name.as_str())
                    ))
        }
        _ => false,
    }
}

fn promise_is_consumed(source: &str, span: Span) -> bool {
    source
        .get(..span.start as usize)
        .is_some_and(prefix_ends_with_await)
        || source
            .get(span.end as usize..)
            .is_some_and(|suffix| suffix.trim_start().starts_with(".then"))
}

fn prefix_ends_with_await(prefix: &str) -> bool {
    let mut prefix = prefix.trim_end();
    while let Some(stripped) = prefix.strip_suffix('(') {
        prefix = stripped.trim_end();
    }
    prefix.ends_with("await")
}
