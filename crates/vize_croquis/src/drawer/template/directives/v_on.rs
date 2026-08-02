//! v-on directive handling.
//!
//! Processes `@event="handler"` bindings including:
//! - Inline arrow/function callback scope creation
//! - Implicit `$event` parameter detection
//! - Simple handler reference tracking

use crate::drawer::Drawer;
use crate::drawer::helpers::extract_inline_callback_params;
use crate::scope::EventHandlerScopeData;
use oxc_ast::ast::Statement;
use oxc_parser::{ParseOptions, Parser};
use oxc_span::SourceType;
use vize_carton::{CompactString, profile, smallvec};
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
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(c) => c.loc.source.as_str(),
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
                            start: loc.start.offset,
                            end: loc.end.offset,
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

                self.croquis.scopes.enter_event_handler_scope(
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
                    let scope_id = self.croquis.scopes.current_scope().id;
                    let exp_loc = exp.loc();
                    self.croquis
                        .template_expressions
                        .push(crate::croquis::TemplateExpression {
                            content: CompactString::new(content),
                            kind: crate::croquis::TemplateExpressionKind::VOn,
                            start: exp_loc.start.offset,
                            end: exp_loc.end.offset,
                            scope_id,
                            vif_guard: self.current_vif_guard(),
                        });
                }

                let params_added: Vec<CompactString> = self
                    .croquis
                    .scopes
                    .current_scope()
                    .bindings()
                    .filter(|(name, _)| *name != "$event")
                    .map(|(name, _)| CompactString::new(name))
                    .collect();

                for param in &params_added {
                    scope_vars.push(param.clone());
                }

                if self.options.detect_undefined {
                    profile!(
                        "croquis.template.v_on.refs",
                        self.check_expression_refs(exp, scope_vars)
                    );
                }

                for _ in &params_added {
                    scope_vars.pop();
                }

                self.croquis.scopes.exit_scope();
            } else {
                // Simple handler reference, or inline statement-list handler.
                let has_implicit_event = content.contains("$event") || !content.contains('(');
                let is_statement_list = profile!(
                    "croquis.template.v_on.statement_list",
                    is_inline_statement_list(content)
                );

                if has_implicit_event || (is_statement_list && !content.contains("=>")) {
                    self.croquis.scopes.enter_event_handler_scope(
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
                            has_implicit_event,
                            param_names: smallvec![],
                            handler_expression: Some(CompactString::new(content)),
                            target_component,
                        },
                        dir.loc.start.offset,
                        dir.loc.end.offset,
                    );

                    if self.options.collect_template_expressions {
                        let scope_id = self.croquis.scopes.current_scope().id;
                        let exp_loc = exp.loc();
                        self.croquis.template_expressions.push(
                            crate::croquis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::croquis::TemplateExpressionKind::VOn,
                                start: exp_loc.start.offset,
                                end: exp_loc.end.offset,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }

                    scope_vars.push(CompactString::const_new("$event"));

                    if self.options.detect_undefined {
                        profile!(
                            "croquis.template.v_on.refs",
                            self.check_expression_refs(exp, scope_vars)
                        );
                    }

                    scope_vars.pop();
                    self.croquis.scopes.exit_scope();
                } else {
                    if self.options.collect_template_expressions {
                        let scope_id = self.croquis.scopes.current_scope().id;
                        let exp_loc = exp.loc();
                        self.croquis.template_expressions.push(
                            crate::croquis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::croquis::TemplateExpressionKind::VOn,
                                start: exp_loc.start.offset,
                                end: exp_loc.end.offset,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }

                    if self.options.detect_undefined {
                        profile!(
                            "croquis.template.v_on.refs",
                            self.check_expression_refs(exp, scope_vars)
                        );
                    }
                }
            }
        }
    }
}

fn is_inline_statement_list(content: &str) -> bool {
    let trimmed = content.trim_end();
    if trimmed.ends_with(';')
        || content
            .split(';')
            .take(2)
            .filter(|part| !part.trim().is_empty())
            .count()
            > 1
    {
        return true;
    }

    if !content.contains(['\n', '\r']) {
        return false;
    }

    let allocator = oxc_allocator::Allocator::default();
    let parsed = Parser::new(&allocator, content, SourceType::ts())
        .with_options(ParseOptions {
            allow_return_outside_function: true,
            ..Default::default()
        })
        .parse();
    if parsed.panicked || !parsed.diagnostics.is_empty() {
        return false;
    }

    let mut statements = parsed
        .program
        .body
        .iter()
        .filter(|statement| !matches!(statement, Statement::EmptyStatement(_)));
    let Some(first) = statements.next() else {
        return false;
    };
    !matches!(first, Statement::ExpressionStatement(_)) || statements.next().is_some()
}

#[cfg(test)]
mod tests {
    use super::is_inline_statement_list;

    #[test]
    fn classifies_asi_and_semicolon_statement_lists() {
        for content in [
            "emit('create')\nemit('close')",
            "emit('create')\r\nemit('close')",
            "emit('create'); emit('close')",
            "emit('create');",
            "\nif (ready) run()\n",
            "\nconst value = getValue()\n",
            "\nreturn run()\n",
        ] {
            assert!(
                is_inline_statement_list(content),
                "expected statement-list handler: {content:?}"
            );
        }
    }

    #[test]
    fn keeps_multiline_single_expressions_out_of_statement_scopes() {
        for content in [
            "handler\n  .call(null)",
            "ready\n  ? onReady()\n  : onPending()",
            "items\n  .map(item => item.id)\n  .join(',')",
            "({\n  key: value\n})",
            "emit('create')\n+",
            "emit('create')",
        ] {
            assert!(
                !is_inline_statement_list(content),
                "expected single-expression handler: {content:?}"
            );
        }
    }
}
