use crate::croquis::{TemplateExpression, TemplateExpressionKind};
use crate::drawer::Drawer;
use vize_carton::{CompactString, profile};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

impl Drawer {
    pub(super) fn process_element_directives(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        is_component: bool,
        tag: &str,
    ) {
        profile!("croquis.template.element.second_pass", {
            for prop in &el.props {
                let PropNode::Directive(dir) = prop else {
                    continue;
                };

                if dir.name == "bind" {
                    profile!(
                        "croquis.template.directive.v_bind",
                        self.handle_v_bind_directive(dir, el, scope_vars)
                    );
                } else if dir.name == "if" || dir.name == "else-if" {
                    self.collect_basic_directive_expression(
                        dir.exp.as_ref(),
                        TemplateExpressionKind::VIf,
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
                    let target_component = is_component.then(|| CompactString::new(tag));
                    profile!(
                        "croquis.template.directive.v_on",
                        self.handle_v_on_directive(dir, scope_vars, target_component)
                    );
                }
            }
        });
    }

    pub(super) fn check_element_directive_refs(
        &mut self,
        el: &ElementNode<'_>,
        scope_vars: &[CompactString],
    ) {
        profile!("croquis.template.element.undefined_refs", {
            if !self.options.detect_undefined || !self.script_drawn {
                return;
            }

            for prop in &el.props {
                if let PropNode::Directive(dir) = prop
                    && let Some(ref exp) = dir.exp
                    && dir.name != "for"
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
}

fn expression_content<'a>(exp: &'a ExpressionNode<'_>) -> &'a str {
    match exp {
        ExpressionNode::Simple(s) => s.content.as_str(),
        ExpressionNode::Compound(c) => c.loc.source.as_str(),
    }
}
