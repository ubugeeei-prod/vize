use crate::drawer::Drawer;
use crate::patterns::PatternDiagnostic;
use crate::scope::ScopeKind;
use vize_carton::{CompactString, cstr};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode, SimpleExpressionNode};

pub(super) fn directive<'a, 's>(
    el: &'a ElementNode<'s>,
    name: &str,
) -> Option<&'a DirectiveNode<'s>> {
    el.props.iter().find_map(|prop| match prop {
        PropNode::Directive(dir) if dir.name == name => Some(dir.as_ref()),
        _ => None,
    })
}

impl Drawer {
    pub(in crate::drawer::template) fn check_orphan_pattern_arm(
        &mut self,
        el: &ElementNode<'_>,
        direct_match_child: bool,
    ) {
        if !self.options.experimental_patterned_template {
            return;
        }
        for name in ["match", "when"] {
            for dir in el
                .props
                .iter()
                .filter_map(|prop| match prop {
                    PropNode::Directive(dir) if dir.name == name => Some(dir),
                    _ => None,
                })
                .skip(1)
            {
                self.pattern_diagnostic(
                    &cstr!("Duplicate v-{name} directive."),
                    dir.loc.span.start,
                    dir.loc.span.end,
                    false,
                );
            }
        }
        let Some(dir) = directive(el, "when") else {
            return;
        };
        let scope = self.croquis.scopes.current_scope();
        if !direct_match_child
            && (scope.kind != ScopeKind::VWhen || scope.span.start != el.loc.span.start)
        {
            self.pattern_diagnostic(
                "v-when must be a direct child of v-match.",
                dir.loc.span.start,
                dir.loc.span.end,
                false,
            );
        }
    }

    pub(super) fn pattern_expression<'a, 's>(
        &mut self,
        dir: &'a DirectiveNode<'s>,
        name: &str,
    ) -> Option<&'a SimpleExpressionNode<'s>> {
        if dir.arg.is_none()
            && dir.modifiers.is_empty()
            && let Some(ExpressionNode::Simple(exp)) = &dir.exp
            && !exp.content.trim().is_empty()
        {
            return Some(exp);
        }
        self.pattern_diagnostic(
            &cstr!("{name} requires an expression and accepts no arguments or modifiers."),
            dir.loc.span.start,
            dir.loc.span.end,
            false,
        );
        None
    }

    pub(super) fn invalid_pattern_arm_host(
        &mut self,
        el: &ElementNode<'_>,
        dir: &DirectiveNode<'_>,
    ) -> bool {
        if el.props.iter().any(|prop| matches!(prop,
            PropNode::Directive(other) if matches!(other.name, "if" | "else-if" | "else" | "for" | "match")
        )) {
            self.pattern_diagnostic(
                "v-when cannot share an element with another structural directive.",
                dir.loc.span.start, dir.loc.span.end, false,
            );
            return true;
        }
        false
    }

    pub(super) fn pattern_diagnostic(
        &mut self,
        message: &str,
        start: u32,
        end: u32,
        warning: bool,
    ) {
        self.croquis.pattern_diagnostics.push(PatternDiagnostic {
            message: CompactString::new(message),
            start,
            end,
            warning,
        });
    }
}
