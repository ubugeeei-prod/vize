use crate::croquis::{TemplateExpression, TemplateExpressionKind};
use crate::drawer::Drawer;
use oxc_allocator::Allocator;
use oxc_ast::ast::Expression;
use oxc_parser::Parser;
use oxc_span::SourceType;
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
        let event_target_component = event_target_component(el, is_component, tag);
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

        let content = expression_content(exp);
        let loc = exp.loc();
        let scope_id = self.croquis.scopes.current_id();
        self.croquis.template_expressions.push(TemplateExpression {
            content: CompactString::new(content),
            kind,
            start: loc.start.offset,
            end: loc.end.offset,
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

        if self.options.collect_template_expressions {
            let loc = arg.loc();
            self.croquis.template_expressions.push(TemplateExpression {
                content: CompactString::new(expression_content(arg)),
                kind: TemplateExpressionKind::DynamicDirectiveArgument,
                start: loc.start.offset,
                end: loc.end.offset,
                scope_id: self.croquis.scopes.current_id(),
                vif_guard: self.current_vif_guard(),
            });
        }

        if self.options.detect_undefined {
            self.check_expression_refs(arg, scope_vars);
        }
    }
}

fn event_target_component(
    el: &ElementNode<'_>,
    is_component: bool,
    tag: &str,
) -> Option<CompactString> {
    if is_component {
        return Some(CompactString::new(tag));
    }

    dynamic_component_target(el, tag)
}

fn dynamic_component_target(el: &ElementNode<'_>, tag: &str) -> Option<CompactString> {
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
        expression_identifier(dir.exp.as_ref()?)
    })
}

fn is_bind_is_directive(dir: &DirectiveNode<'_>) -> bool {
    dir.name == "bind"
        && matches!(
            dir.arg.as_ref(),
            Some(ExpressionNode::Simple(arg)) if arg.content == "is"
        )
}

fn expression_identifier(exp: &ExpressionNode<'_>) -> Option<CompactString> {
    let source = match exp {
        ExpressionNode::Simple(simple) => simple.content.as_str(),
        ExpressionNode::Compound(compound) => compound.loc.source.as_str(),
    };
    parse_component_reference_expression(source)
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

fn expression_content<'a>(exp: &'a ExpressionNode<'_>) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(c) => c.loc.source.as_str(),
    }
}
