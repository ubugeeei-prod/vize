use super::mutation_targets::MutationTargetKind;
use super::scope::{PropScope, expression_source, push_for_aliases, push_identifier_tokens};
use super::{META, NoMutatingProps, handlers};
use crate::context::LintContext;
use vize_l0::{String, cstr};
use vize_relief::{
    DirectiveNode, ElementNode, ExpressionNode, ForNode, PropNode, TemplateChildNode,
};

struct TemplateMutation {
    target: String,
    start: u32,
    end: u32,
}

impl NoMutatingProps {
    /// Two-way bindings (`v-model` and Vue 2 `v-bind.sync`) write their target.
    fn check_two_way_mutation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        scope: &PropScope<'_>,
    ) {
        let binding = if directive.name == "model" {
            "v-model"
        } else if directive.name == "bind"
            && directive
                .modifiers
                .iter()
                .any(|modifier| modifier.content == "sync")
        {
            "v-bind.sync"
        } else {
            return;
        };

        let Some(exp) = directive.exp.as_ref() else {
            return;
        };
        let content = expression_source(exp, ctx.source);
        if !self.should_report_template_mutation(scope.mutation_kind(content)) {
            return;
        }
        let span = exp.loc().span;
        if ctx.diagnostics.iter().any(|diagnostic| {
            diagnostic.rule_name == META.name
                && diagnostic.start == span.start
                && diagnostic.end == span.end
        }) {
            return;
        }
        ctx.report(
            crate::diagnostic::LintDiagnostic::error(
                ctx.current_rule,
                cstr!("Unexpected mutation of prop '{}' via {}", content, binding),
                span.start,
                span.end,
            )
            .with_help("Use a local ref or emit an event instead of mutating props directly"),
        );
    }

    /// Check whether a `v-on` inline handler mutates a prop.
    fn check_handler_mutation<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        directive: &DirectiveNode<'a>,
        scope: &PropScope<'_>,
    ) {
        if directive.name != "on" {
            return;
        }
        let Some(exp) = directive.exp.as_ref() else {
            return;
        };
        self.report_expression_mutations(
            ctx,
            exp,
            scope,
            "in an inline handler",
            "Use a local ref or emit an event instead of mutating props directly",
        );
    }

    fn check_template_expression_mutation(
        &self,
        ctx: &mut LintContext<'_>,
        expression: &ExpressionNode<'_>,
        scope: &PropScope<'_>,
    ) {
        self.report_expression_mutations(
            ctx,
            expression,
            scope,
            "in a template expression",
            "Use a non-mutating expression or derive a local value instead",
        );
    }

    fn report_expression_mutations(
        &self,
        ctx: &mut LintContext<'_>,
        expression: &ExpressionNode<'_>,
        scope: &PropScope<'_>,
        context: &str,
        help: &str,
    ) {
        let expression_start = expression.loc().span.start;
        let mut reports: Vec<TemplateMutation> = Vec::new();
        handlers::for_each_mutation_target(expression_source(expression, ctx.source), |mutation| {
            let target = mutation.target.trim();
            let kind = scope.mutation_kind(target).map(|kind| {
                if mutation.deep && kind == MutationTargetKind::Direct {
                    MutationTargetKind::Deep
                } else {
                    kind
                }
            });
            if !self.should_report_template_mutation(kind) {
                return;
            }
            let start = expression_start + mutation.span.start;
            let end = expression_start + mutation.span.end;
            if reports
                .iter()
                .any(|seen| seen.target == target && seen.start == start && seen.end == end)
            {
                return;
            }
            reports.push(TemplateMutation {
                target: String::new(target),
                start,
                end,
            });
        });
        for mutation in reports {
            if ctx.diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_name == META.name
                    && diagnostic.start == mutation.start
                    && diagnostic.end == mutation.end
            }) {
                continue;
            }
            ctx.report(
                crate::diagnostic::LintDiagnostic::error(
                    ctx.current_rule,
                    cstr!(
                        "Unexpected mutation of prop '{}' {}",
                        mutation.target,
                        context
                    ),
                    mutation.start,
                    mutation.end,
                )
                .with_help(help),
            );
        }
    }

    /// Recursively check template children for prop mutations.
    pub(super) fn check_children<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        children: &[TemplateChildNode<'a>],
        scope: &mut PropScope<'_>,
    ) {
        for child in children {
            match child {
                TemplateChildNode::Element(element) => self.check_element(ctx, element, scope),
                TemplateChildNode::If(if_node) => {
                    for branch in &if_node.branches {
                        if let Some(condition) = branch.condition.as_ref() {
                            self.check_template_expression_mutation(ctx, condition, scope);
                        }
                        self.check_children(ctx, &branch.children, scope);
                    }
                }
                TemplateChildNode::For(for_node) => self.check_for(ctx, for_node, scope),
                TemplateChildNode::Interpolation(interpolation) => {
                    self.check_template_expression_mutation(ctx, &interpolation.content, scope);
                }
                _ => {}
            }
        }
    }

    /// A `v-for` binds its aliases for the whole subtree, so they shadow a prop
    /// of the same name inside it.
    fn check_for<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        for_node: &ForNode<'a>,
        scope: &mut PropScope<'_>,
    ) {
        self.check_template_expression_mutation(ctx, &for_node.source, scope);
        let depth = scope.shadowed.len();
        for alias in [
            for_node.value_alias.as_ref(),
            for_node.key_alias.as_ref(),
            for_node.object_index_alias.as_ref(),
        ]
        .into_iter()
        .flatten()
        {
            push_identifier_tokens(expression_source(alias, ctx.source), &mut scope.shadowed);
        }
        self.check_children(ctx, &for_node.children, scope);
        scope.shadowed.truncate(depth);
    }

    /// Check an element for prop mutations.
    fn check_element<'a>(
        &self,
        ctx: &mut LintContext<'a>,
        element: &ElementNode<'a>,
        scope: &mut PropScope<'_>,
    ) {
        let depth = scope.shadowed.len();

        // A `v-for` alias is in scope for the element's own bindings as well
        // as its children. Collect it before checking the other directives.
        for prop in &element.props {
            if let PropNode::Directive(directive) = prop
                && directive.name == "for"
            {
                if let Some(expression) = directive.exp.as_ref() {
                    self.check_template_expression_mutation(ctx, expression, scope);
                }
                push_for_aliases(directive, &mut scope.shadowed, ctx.source);
            }
        }

        for prop in &element.props {
            if let PropNode::Directive(directive) = prop {
                self.check_two_way_mutation(ctx, directive, scope);
                self.check_handler_mutation(ctx, directive, scope);
                if !matches!(directive.name, "for" | "on")
                    && let Some(expression) = directive.exp.as_ref()
                {
                    self.check_template_expression_mutation(ctx, expression, scope);
                }
            }
        }

        // A slot variable scopes only the slot content, not the owner's attrs.
        for prop in &element.props {
            if let PropNode::Directive(directive) = prop
                && directive.name == "slot"
                && let Some(expression) = directive.exp.as_ref()
            {
                push_identifier_tokens(
                    expression_source(expression, ctx.source),
                    &mut scope.shadowed,
                );
            }
        }

        self.check_children(ctx, &element.children, scope);
        scope.shadowed.truncate(depth);
    }

    pub(super) fn should_report_template_mutation(&self, kind: Option<MutationTargetKind>) -> bool {
        match kind {
            Some(MutationTargetKind::Direct) => true,
            Some(MutationTargetKind::Deep) => !self.options.shallow_only,
            None => false,
        }
    }
}
