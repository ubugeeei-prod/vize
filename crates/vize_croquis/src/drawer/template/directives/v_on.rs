//! v-on directive handling.
//!
//! Processes `@event="handler"` bindings including:
//! - Inline arrow/function callback scope creation
//! - Implicit `$event` parameter detection
//! - Simple handler reference tracking

use crate::drawer::Drawer;
use crate::drawer::helpers::extract_inline_callback_params;
use crate::scope::EventHandlerScopeData;
use vize_carton::{CompactString, profile};
use vize_relief::ExpressionNode;

impl Drawer {
    /// Handle v-on directive.
    pub(in crate::drawer) fn handle_v_on_directive(
        &mut self,
        dir: &vize_relief::DirectiveNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        target_component: Option<CompactString>,
    ) {
        if let Some(ref exp) = dir.exp {
            let compound_content;
            let content = match exp {
                ExpressionNode::Simple(s) => s.content,
                ExpressionNode::Compound(c) => {
                    compound_content = CompactString::new(c.loc.span.slice(&self.template_source));
                    compound_content.as_str()
                }
            };

            if dir.arg.is_none() {
                if self.options.collect_template_expressions {
                    let loc = exp.loc();
                    let scope_id = self.croquis.scopes.current_id();
                    self.croquis
                        .template_expressions
                        .push(crate::croquis::TemplateExpression {
                            content: CompactString::new(content),
                            kind: crate::croquis::TemplateExpressionKind::VOn,
                            start: loc.span.start,
                            end: loc.span.end,
                            scope_id,
                            vif_guard: self.current_vif_guard(),
                        });
                }

                if self.options.detect_undefined {
                    profile!(
                        "croquis.template.v_on.refs",
                        self.check_expression_refs(exp, scope_vars)
                    );
                }

                return;
            }

            // Every named listener runs in a handler scope. Whether its body
            // contains parentheses, semicolons or nested callbacks cannot change
            // that ownership; statement bodies must never become `void (body)`.
            let params = profile!(
                "croquis.template.callback.extract_params",
                extract_inline_callback_params(content)
            );
            let has_implicit_event = params.is_none();
            let event_name = match dir.arg.as_ref().expect("named event") {
                ExpressionNode::Simple(argument) => CompactString::new(argument.content),
                ExpressionNode::Compound(argument) => {
                    CompactString::new(argument.loc.span.slice(&self.template_source))
                }
            };
            self.croquis.scopes.enter_event_handler_scope(
                EventHandlerScopeData {
                    event_name,
                    has_implicit_event,
                    param_names: params.unwrap_or_default().into_iter().collect(),
                    handler_expression: Some(CompactString::new(content)),
                    target_component,
                },
                dir.loc.span.start,
                dir.loc.span.end,
            );
            if self.options.collect_template_expressions {
                let location = exp.loc();
                self.croquis
                    .template_expressions
                    .push(crate::croquis::TemplateExpression {
                        content: CompactString::new(content),
                        kind: crate::croquis::TemplateExpressionKind::VOn,
                        start: location.span.start,
                        end: location.span.end,
                        scope_id: self.croquis.scopes.current_id(),
                        vif_guard: self.current_vif_guard(),
                    });
            }
            let previous_count = scope_vars.len();
            scope_vars.extend(
                self.croquis
                    .scopes
                    .current_scope()
                    .bindings()
                    .map(|(name, _)| CompactString::new(name)),
            );
            if self.options.detect_undefined {
                profile!(
                    "croquis.template.v_on.refs",
                    self.check_expression_refs(exp, scope_vars)
                );
            }
            scope_vars.truncate(previous_count);
            self.croquis.scopes.exit_scope();
        }
    }
}

#[cfg(test)]
mod tests;
