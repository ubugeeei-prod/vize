use crate::croquis::{TemplateExpression, TemplateExpressionKind};
use crate::drawer::Drawer;
use crate::drawer::helpers::is_builtin_directive;
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::SourceType;
use vize_carton::{CompactString, profile};
use vize_relief::{DirectiveNode, ElementNode, ExpressionNode, JsExpression, PropNode};

use super::super::slot_names::slot_argument_is_runtime_dynamic;

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
            if self.options.detect_undefined
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
        is_component: bool,
        tag: &str,
    ) {
        let event_target_component = self.event_target_component(el, is_component, tag);
        profile!("croquis.template.element.second_pass", {
            for prop in &el.props {
                let PropNode::Directive(dir) = prop else {
                    continue;
                };

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
            if !self.options.detect_undefined {
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
                kind: TemplateExpressionKind::DynamicDirectiveArgument,
                start: loc.span.start,
                end: loc.span.end,
                scope_id: self.croquis.scopes.current_id(),
                vif_guard: self.current_vif_guard(),
            });
        }

        if self.options.detect_undefined {
            self.check_expression_refs(arg, scope_vars);
        }
    }
}

impl Drawer {
    pub(super) fn event_target_component(
        &self,
        el: &ElementNode<'_>,
        is_component: bool,
        tag: &str,
    ) -> Option<CompactString> {
        if is_component {
            return Some(CompactString::new(tag));
        }

        self.dynamic_component_target(el, tag)
    }

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
            let target = expression_identifier(dir.exp.as_ref()?, &self.template_source)?;
            dynamic_component_target_is_known(self, target.as_str()).then_some(target)
        })
    }
}

fn is_bind_is_directive(dir: &DirectiveNode<'_>) -> bool {
    dir.name == "bind"
        && matches!(
            dir.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.content == "is"
        )
}

fn expression_identifier(exp: &ExpressionNode<'_>, template_source: &str) -> Option<CompactString> {
    let (source, retained) = match exp {
        ExpressionNode::Simple(simple) => (simple.content, simple.js_ast.as_ref()),
        ExpressionNode::Compound(compound) => (compound.loc.span.slice(template_source), None),
    };
    component_reference_expression(source, retained)
}

/// Recognize `<component :is="...">` targets that name a component: a lone
/// identifier or a static member chain.
///
/// Nodes carrying the parse-once retained AST (P1-5) are shape-checked
/// directly and the legacy throwaway parse dies for them (Davinci P1-6);
/// nodes without one (invalid or incomplete text, compound expressions) keep
/// the legacy parse. Under `cfg(any(test, feature = "davinci-differential"))`
/// every retained check is dual-run against the legacy parse and divergence
/// panics — the P1-6 differential lane.
fn component_reference_expression(
    source: &str,
    retained: Option<&JsExpression<'_>>,
) -> Option<CompactString> {
    let result = match retained {
        Some(js) => component_reference_from_ast(js.ast, source),
        None => parse_component_reference_expression(source),
    };
    #[cfg(any(test, feature = "davinci-differential"))]
    if retained.is_some() {
        let legacy = parse_component_reference_expression(source);
        assert_eq!(
            result, legacy,
            "davinci-differential (P1-6): retained-AST component-reference check diverged from the legacy parse for expression {source:?}"
        );
        crate::drawer::differential::record_component_reference_comparison();
    }
    result
}

fn component_reference_from_ast(ast: &Expression<'_>, source: &str) -> Option<CompactString> {
    match ast {
        Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
            Some(CompactString::new(source.trim()))
        }
        _ => None,
    }
}

fn parse_component_reference_expression(source: &str) -> Option<CompactString> {
    let allocator = Allocator::default();
    let source_type = SourceType::from_path("expr.ts").unwrap_or_default();
    let expression = Parser::new(&allocator, source, source_type)
        .parse_expression()
        .ok()?;
    match expression {
        Expression::Identifier(_) | Expression::StaticMemberExpression(_) => {
            Some(CompactString::new(source.trim()))
        }
        _ => None,
    }
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
