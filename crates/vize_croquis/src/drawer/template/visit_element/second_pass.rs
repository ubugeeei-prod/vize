use super::super::slot_names::slot_argument_is_runtime_dynamic;
use super::component_reference::expression_identifier;
use super::dynamic_component_alias::dynamic_component_alias;
use super::v_for_scope::v_for_scope_bindings;
use crate::croquis::{TemplateExpression, TemplateExpressionKind};
use crate::drawer::Drawer;
use crate::drawer::helpers::{
    extract_identifier_refs_oxc, is_builtin_directive, parse_v_for_scope_expression,
};
use vize_carton::{CompactString, profile};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, PropNode};
impl Drawer {
    pub(super) fn process_element_conditional_directive(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &[CompactString],
    ) {
        for prop in &el.props {
            let PropNode::Directive(dir) = prop else {
                continue;
            };
            if dir.name != "if" && dir.name != "else-if" {
                continue;
            }
            self.collect_basic_directive_expression(dir.exp.as_ref(), TemplateExpressionKind::VIf);
            if self.checks_binding_reads()
                && let Some(exp) = dir.exp.as_ref()
            {
                self.check_expression_refs(exp, scope_vars);
            }
        }
    }
    pub(super) fn process_element_directives(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        event_target_component: Option<CompactString>,
    ) {
        profile!("croquis.template.element.second_pass", {
            for prop in &el.props {
                let PropNode::Directive(dir) = prop else {
                    continue;
                };
                // Pattern directives use their own grammar. When disabled,
                // Canon reports that policy error; never emit their pattern
                // text as a JavaScript custom-directive expression.
                if matches!(dir.name, "match" | "when") {
                    continue;
                }
                if dir.name != "slot" {
                    self.collect_dynamic_directive_argument(dir, scope_vars);
                }

                if dir.name == "bind" {
                    profile!(
                        "croquis.template.directive.v_bind",
                        self.handle_v_bind_directive(dir, el, scope_vars)
                    );
                } else if dir.name == "show" {
                    self.collect_basic_directive_expression(
                        dir.exp.as_ref(),
                        TemplateExpressionKind::VShow,
                    );
                } else if dir.name == "html" || dir.name == "text" {
                    // Their values are ordinary template expressions; without
                    // collecting them the checker never sees them (#9110).
                    self.collect_basic_directive_expression(
                        dir.exp.as_ref(),
                        if dir.name == "html" {
                            TemplateExpressionKind::VHtml
                        } else {
                            TemplateExpressionKind::VText
                        },
                    );
                } else if dir.name == "model" {
                    self.collect_basic_directive_expression(
                        dir.exp.as_ref(),
                        TemplateExpressionKind::VModel,
                    );
                } else if dir.name == "on" && self.options.analyze_template_scopes {
                    profile!(
                        "croquis.template.directive.v_on",
                        self.handle_v_on_directive(dir, scope_vars, event_target_component.clone())
                    );
                } else if !is_builtin_directive(dir.name) {
                    // A custom directive's value is an ordinary template
                    // expression; collecting it here is what lets it reach the
                    // type checker, and gives it the enclosing scope id and
                    // v-if guard for free, exactly like `v-show`.
                    self.collect_basic_directive_expression(
                        dir.exp.as_ref(),
                        TemplateExpressionKind::CustomDirective,
                    );
                }
            }
        });
    }

    pub(super) fn process_dynamic_slot_argument(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &[CompactString],
    ) {
        for prop in &el.props {
            if let PropNode::Directive(dir) = prop
                && dir.name == "slot"
            {
                self.collect_dynamic_directive_argument(dir, scope_vars);
            }
        }
    }

    pub(super) fn check_element_directive_refs(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &[CompactString],
    ) {
        profile!("croquis.template.element.undefined_refs", {
            if !self.options.detect_undefined && !self.track_unused_bindings {
                return;
            }

            for prop in &el.props {
                if let PropNode::Directive(dir) = prop
                    && let Some(ref exp) = dir.exp
                    && dir.name != "for"
                    && dir.name != "if"
                    && dir.name != "else-if"
                    && dir.name != "slot"
                    && dir.name != "on"
                    && dir.name != "bind"
                    && !matches!(dir.name, "match" | "when")
                {
                    self.check_expression_refs(exp, scope_vars);
                }
            }
        });
    }

    fn collect_basic_directive_expression(
        &mut self,
        exp: Option<&ExpressionNode<'_>>,
        kind: TemplateExpressionKind,
    ) {
        if !self.options.collect_template_expressions {
            return;
        }

        let Some(exp) = exp else {
            return;
        };

        let content = expression_content(exp, &self.template_source);
        let loc = exp.loc();
        let scope_id = self.croquis.scopes.current_id();
        self.croquis.template_expressions.push(TemplateExpression {
            content: CompactString::new(content),
            kind,
            start: loc.span.start,
            end: loc.span.end,
            scope_id,
            vif_guard: self.current_vif_guard(),
        });
    }

    fn collect_dynamic_directive_argument(
        &mut self,
        dir: &DirectiveNode<'_>,
        scope_vars: &[CompactString],
    ) {
        let Some(arg) = dir.arg.as_ref().filter(|arg| match arg {
            ExpressionNode::Simple(simple) => !simple.is_static,
            ExpressionNode::Compound(_) => true,
        }) else {
            return;
        };
        if dir.name == "slot" && !slot_argument_is_runtime_dynamic(arg, &self.template_source) {
            return;
        }

        if self.options.collect_template_expressions {
            let loc = arg.loc();
            self.croquis.template_expressions.push(TemplateExpression {
                content: CompactString::new(expression_content(arg, &self.template_source)),
                kind: if dir.name == "on" {
                    TemplateExpressionKind::DynamicEventArgument
                } else {
                    TemplateExpressionKind::DynamicDirectiveArgument
                },
                start: loc.span.start,
                end: loc.span.end,
                scope_id: self.croquis.scopes.current_id(),
                vif_guard: self.current_vif_guard(),
            });
        }

        if self.checks_binding_reads() {
            self.check_expression_refs(arg, scope_vars);
        }
    }
}

impl Drawer {
    pub(super) fn dynamic_component_target(
        &self,
        el: &ElementNode<'_>,
        tag: &str,
    ) -> Option<CompactString> {
        if tag != "component" {
            return None;
        }

        el.props.iter().find_map(|prop| {
            let PropNode::Directive(dir) = prop else {
                return None;
            };
            if !is_bind_is_directive(dir) {
                return None;
            }
            let exp = dir.exp.as_ref()?;
            if let Some(target) = expression_identifier(exp, &self.template_source) {
                // Capture the member once, inside its recorded narrowing
                // guard when present. The authored :is expression owns errors;
                // repeated root-level typeof queries must not duplicate them.
                let owns_scope = !el.props.iter().any(|prop| matches!(prop, PropNode::Directive(dir) if matches!(dir.name, "scope" | "for")));
                if target.contains('.') && owns_scope
                    && self.in_template_root_scope()
                    && self.dynamic_target_has_script_root(target.as_str())
                    && self.new_dynamic_alias_is_unbound(el.loc.span.start)
                    && !dynamic_is_captures_same_element_for_alias(el, target.as_str(), &self.template_source)
                {
                    return Some(CompactString::new(dynamic_component_alias(el.loc.span.start)));
                }
                return dynamic_component_target_is_known(self, target.as_str()).then_some(target);
            }
            // Root-owned script values keep their identity under a slot/loop
            // which does not capture them; so must the v-if guard the root
            // alias repeats. Actual lexical bindings, unknown globals and
            // same-element v-for captures still cannot be hoisted.
            ((self.in_template_root_scope()
                || (!el.props.iter().any(|prop| matches!(prop, PropNode::Directive(dir) if dir.name == "scope"))
                    && self.new_dynamic_alias_is_unbound(el.loc.span.start)
                    && self.dynamic_is_reads_only_script_values(exp)
                    && self.vif_guard_reads_only_script_values()))
                && !dynamic_is_captures_same_element_for_alias(
                    el,
                    expression_content(exp, &self.template_source),
                    &self.template_source,
                ))
            .then(|| CompactString::new(dynamic_component_alias(el.loc.span.start)))
        })
    }
}

fn dynamic_is_captures_same_element_for_alias(
    el: &ElementNode<'_>,
    is_expression: &str,
    template_source: &str,
) -> bool {
    let references = extract_identifier_refs_oxc(is_expression);
    if references.is_empty() {
        return false;
    }
    el.props.iter().any(|prop| {
        let PropNode::Directive(dir) = prop else {
            return false;
        };
        if dir.name != "for" {
            return false;
        }
        let Some(exp) = dir.exp.as_ref() else {
            return false;
        };
        parse_v_for_scope_expression(expression_content(exp, template_source)).is_some_and(
            |aliases| {
                v_for_scope_bindings(&aliases)
                    .iter()
                    .any(|alias| references.iter().any(|reference| reference.name == *alias))
            },
        )
    })
}

fn is_bind_is_directive(dir: &DirectiveNode<'_>) -> bool {
    dir.name == "bind"
        && matches!(
            dir.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.content == "is"
        )
}

fn dynamic_component_target_is_known(drawer: &Drawer, target: &str) -> bool {
    let root = target
        .split('.')
        .next()
        .map(str::trim)
        .unwrap_or(target)
        .trim();
    drawer.croquis.bindings.contains(root) || starts_like_component_identifier(root)
}

fn starts_like_component_identifier(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|character| character.is_ascii_uppercase())
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>, source: &'a str) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content,
        ExpressionNode::Compound(c) => c.loc.span.slice(source),
    }
}
