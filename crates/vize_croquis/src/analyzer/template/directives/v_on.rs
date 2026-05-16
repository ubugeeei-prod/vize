//! v-on directive handling.
//!
//! Processes `@event="handler"` bindings including:
//! - Inline arrow/function callback scope creation
//! - Implicit `$event` parameter detection
//! - Simple handler reference tracking

use crate::analyzer::helpers::extract_inline_callback_params;
use crate::analyzer::Analyzer;
use crate::scope::EventHandlerScopeData;
use vize_carton::{profile, smallvec, CompactString};
use vize_relief::ast::ExpressionNode;

impl Analyzer {
    /// Handle v-on directive.
    pub(in crate::analyzer) fn handle_v_on_directive(
        &mut self,
        dir: &vize_relief::ast::DirectiveNode<'_>,
        scope_vars: &mut Vec<CompactString>,
        target_component: Option<CompactString>,
    ) {
        if let Some(ref exp) = dir.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(c) => c.loc.source.as_str(),
            };

            // Check for inline arrow/function
            if let Some(params) = profile!(
                "croquis.template.callback.extract_params",
                extract_inline_callback_params(content)
            ) {
                let event_name = dir
                    .arg
                    .as_ref()
                    .map(|arg| match arg {
                        ExpressionNode::Simple(s) => CompactString::new(s.content.as_str()),
                        ExpressionNode::Compound(c) => CompactString::new(c.loc.source.as_str()),
                    })
                    .unwrap_or_else(|| CompactString::const_new("unknown"));

                self.summary.scopes.enter_event_handler_scope(
                    EventHandlerScopeData {
                        event_name,
                        has_implicit_event: false,
                        param_names: params.into_iter().collect(),
                        handler_expression: Some(CompactString::new(content)),
                        target_component: target_component.clone(),
                    },
                    dir.loc.start.offset,
                    dir.loc.end.offset,
                );

                if self.options.collect_template_expressions {
                    let scope_id = self.summary.scopes.current_scope().id;
                    self.summary
                        .template_expressions
                        .push(crate::analysis::TemplateExpression {
                            content: CompactString::new(content),
                            kind: crate::analysis::TemplateExpressionKind::VOn,
                            start: dir.loc.start.offset,
                            end: dir.loc.end.offset,
                            scope_id,
                            vif_guard: self.current_vif_guard(),
                        });
                }

                let params_added: Vec<CompactString> = self
                    .summary
                    .scopes
                    .current_scope()
                    .bindings()
                    .filter(|(name, _)| *name != "$event")
                    .map(|(name, _)| CompactString::new(name))
                    .collect();

                for param in &params_added {
                    scope_vars.push(param.clone());
                }

                if self.options.detect_undefined && self.script_analyzed {
                    profile!(
                        "croquis.template.v_on.refs",
                        self.check_expression_refs(exp, scope_vars, dir.loc.start.offset)
                    );
                }

                for _ in &params_added {
                    scope_vars.pop();
                }

                self.summary.scopes.exit_scope();
            } else {
                // Simple handler reference
                let has_implicit_event = content.contains("$event") || !content.contains('(');

                if has_implicit_event && !content.contains("=>") {
                    self.summary.scopes.enter_event_handler_scope(
                        EventHandlerScopeData {
                            event_name: dir
                                .arg
                                .as_ref()
                                .map(|arg| match arg {
                                    ExpressionNode::Simple(s) => {
                                        CompactString::new(s.content.as_str())
                                    }
                                    ExpressionNode::Compound(c) => {
                                        CompactString::new(c.loc.source.as_str())
                                    }
                                })
                                .unwrap_or_else(|| CompactString::const_new("unknown")),
                            has_implicit_event: true,
                            param_names: smallvec![],
                            handler_expression: Some(CompactString::new(content)),
                            target_component,
                        },
                        dir.loc.start.offset,
                        dir.loc.end.offset,
                    );

                    if self.options.collect_template_expressions {
                        let scope_id = self.summary.scopes.current_scope().id;
                        self.summary.template_expressions.push(
                            crate::analysis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::analysis::TemplateExpressionKind::VOn,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }

                    scope_vars.push(CompactString::const_new("$event"));

                    if self.options.detect_undefined && self.script_analyzed {
                        profile!(
                            "croquis.template.v_on.refs",
                            self.check_expression_refs(exp, scope_vars, dir.loc.start.offset)
                        );
                    }

                    scope_vars.pop();
                    self.summary.scopes.exit_scope();
                } else {
                    if self.options.collect_template_expressions {
                        let scope_id = self.summary.scopes.current_scope().id;
                        self.summary.template_expressions.push(
                            crate::analysis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::analysis::TemplateExpressionKind::VOn,
                                start: dir.loc.start.offset,
                                end: dir.loc.end.offset,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }

                    if self.options.detect_undefined && self.script_analyzed {
                        profile!(
                            "croquis.template.v_on.refs",
                            self.check_expression_refs(exp, scope_vars, dir.loc.start.offset)
                        );
                    }
                }
            }
        }
    }
}
