//! Pattern scopes are built before visiting arm hosts, without rewriting the AST.

#[cfg(test)]
mod tests;
mod validation;

use vize_armature::patterns::{
    MatchPattern, PatternKind, attribute_source_offset, parse_match_attribute,
};
use vize_carton::{Allocator, Box, CompactString};
use vize_relief::{
    BindingType, ElementNode, ExpressionNode, SimpleExpressionNode, SourceLocation,
    TemplateChildNode,
};

use super::visit_element::bounds::element_subtree_end;
use crate::croquis::{TemplateExpression, TemplateExpressionKind};
use crate::drawer::Drawer;
use crate::patterns::{MatchScopeData, WhenScopeData};
use crate::scope::{ScopeBinding, ScopeData, ScopeKind, Span};
use validation::directive;

impl Drawer {
    pub(super) fn visit_patterned_children(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) -> bool {
        if !self.options.experimental_patterned_template {
            return false;
        }
        let Some(dir) = directive(el, "match") else {
            return false;
        };
        let Some(subject) = self.pattern_expression(dir, "v-match") else {
            return false;
        };
        let start = subject.loc.span.start;
        let end = subject.loc.span.end;
        self.collect_pattern_expression(subject.content, start, end, scope_vars);
        self.enter_pattern_scope(
            ScopeKind::VMatch,
            el,
            ScopeData::VMatch(MatchScopeData {
                subject: CompactString::new(subject.content),
                start,
                end,
            }),
        );
        let mut wildcard = false;
        let mut arms = 0;
        for child in &el.children {
            match child {
                TemplateChildNode::Comment(_) => continue,
                TemplateChildNode::Text(text) if text.content.trim().is_empty() => continue,
                TemplateChildNode::Element(arm) if directive(arm, "when").is_some() => {
                    arms += 1;
                    self.visit_pattern_arm(arm, scope_vars, &mut wildcard);
                }
                _ => {
                    let (start, end) = Self::template_child_range(child);
                    self.pattern_diagnostic(
                        "Every direct child of v-match must declare v-when.",
                        start,
                        end,
                        true,
                    );
                    self.visit_template_child(child, scope_vars);
                }
            }
        }
        if arms == 0 {
            self.pattern_diagnostic("v-match has no v-when arms.", start, end, true);
        }
        self.croquis.scopes.exit_scope();
        true
    }

    fn visit_pattern_arm(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        wildcard: &mut bool,
    ) {
        let Some(dir) = directive(el, "when") else {
            return;
        };
        let Some(exp) = self.pattern_expression(dir, "v-when") else {
            self.visit_element_with_pattern_recovery(el, scope_vars, true);
            return;
        };
        if self.invalid_pattern_arm_host(el, dir) {
            self.visit_element_with_pattern_recovery(el, scope_vars, true);
            return;
        }
        let offset = exp.loc.span.start;
        let Some(raw) = self
            .template_source
            .get(exp.loc.span.start as usize..exp.loc.span.end as usize)
        else {
            self.pattern_diagnostic(
                "Pattern expression is missing authored source metadata.",
                dir.loc.span.start,
                dir.loc.span.end,
                false,
            );
            self.visit_element_with_pattern_recovery(el, scope_vars, true);
            return;
        };
        let arm = match parse_match_attribute(raw) {
            Ok(arm) => arm,
            Err(error) => {
                let at = offset + error.offset;
                let start = if at < exp.loc.span.end { at } else { offset };
                self.pattern_diagnostic(error.message.as_str(), start, exp.loc.span.end, false);
                self.visit_element_with_pattern_recovery(el, scope_vars, true);
                return;
            }
        };
        if *wildcard {
            self.pattern_diagnostic(
                "An unguarded wildcard arm must be last and unique.",
                dir.loc.span.start,
                dir.loc.span.end,
                false,
            );
        }
        *wildcard |= matches!(arm.pattern.kind, PatternKind::Wildcard) && arm.guard.is_none();
        // Values resolve in the enclosing environment, even if this arm
        // declares the same spelling later (including an `as` binding).
        self.collect_pattern_values(&arm.pattern, offset, scope_vars);
        self.enter_pattern_scope(
            ScopeKind::VWhen,
            el,
            ScopeData::VWhen(WhenScopeData {
                arm: arm.clone(),
                offset,
            }),
        );
        let vars_before = scope_vars.len();
        for binding in &arm.bindings {
            let name = CompactString::new(binding.name.as_str());
            self.croquis.scopes.add_binding(
                name.clone(),
                ScopeBinding::new(BindingType::SetupConst, offset + binding.span.start),
            );
            scope_vars.push(name);
        }
        if let Some(guard) = &arm.guard {
            self.collect_pattern_expression(
                guard.text.as_str(),
                offset + guard.span.start,
                offset + guard.span.end,
                scope_vars,
            );
        }
        self.visit_element(el, scope_vars);
        scope_vars.truncate(vars_before);
        self.croquis.scopes.exit_scope();
    }

    fn enter_pattern_scope(&mut self, kind: ScopeKind, el: &ElementNode<'_>, data: ScopeData) {
        self.croquis.scopes.enter_scope_with_vue_global(kind);
        let scope = self.croquis.scopes.current_scope_mut();
        scope.span = Span::new(el.loc.span.start, element_subtree_end(el));
        scope.set_data(data);
    }

    fn collect_pattern_values(
        &mut self,
        pattern: &MatchPattern,
        offset: u32,
        vars: &[CompactString],
    ) {
        match &pattern.kind {
            PatternKind::Value(value) => self.collect_pattern_expression(
                value.text.as_str(),
                offset + value.span.start,
                offset + value.span.end,
                vars,
            ),
            PatternKind::As { pattern, .. } => self.collect_pattern_values(pattern, offset, vars),
            PatternKind::Object { properties, .. } => {
                for property in properties {
                    self.collect_pattern_values(&property.pattern, offset, vars);
                }
            }
            PatternKind::Array { elements, .. } | PatternKind::Or(elements) => {
                for element in elements {
                    self.collect_pattern_values(element, offset, vars);
                }
            }
            _ => {}
        }
    }

    fn collect_pattern_expression(
        &mut self,
        content: &str,
        start: u32,
        end: u32,
        vars: &[CompactString],
    ) {
        if self.options.collect_template_expressions {
            self.croquis.template_expressions.push(TemplateExpression {
                content: CompactString::new(content),
                kind: TemplateExpressionKind::CustomDirective,
                start,
                end,
                scope_id: self.croquis.scopes.current_id(),
                vif_guard: self.current_vif_guard(),
            });
        }
        if self.options.detect_undefined {
            let allocator = Allocator::new();
            let expression = ExpressionNode::Simple(Box::new_in(
                SimpleExpressionNode::new(
                    content,
                    false,
                    SourceLocation {
                        span: vize_carton::Span::new(start, end),
                    },
                ),
                &&allocator,
            ));
            let previous = self.croquis.undefined_refs.len();
            self.check_expression_refs(&expression, vars);
            if let Some(raw) = self.template_source.get(start as usize..end as usize) {
                for reference in &mut self.croquis.undefined_refs[previous..] {
                    reference.offset =
                        start + attribute_source_offset(raw, reference.offset - start);
                }
            }
        }
    }
}
