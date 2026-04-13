//! v-bind directive handling.
//!
//! Processes `:attr="expr"` bindings including:
//! - Expression collection for type checking
//! - `$attrs` usage tracking
//! - Inline callback parameter scope creation

use crate::scope::CallbackScopeData;
use vize_carton::{cstr, profile, CompactString};
use vize_relief::ast::{ElementNode, ExpressionNode};

use crate::analyzer::helpers::extract_inline_callback_params;
use crate::analyzer::Analyzer;

impl Analyzer {
    /// Handle v-bind directive.
    ///
    /// Note: `:key` extraction is done in the first pass before scope creation.
    pub(in crate::analyzer) fn handle_v_bind_directive(
        &mut self,
        dir: &vize_relief::ast::DirectiveNode<'_>,
        _el: &ElementNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        if let Some(ref exp) = dir.exp {
            let content = match exp {
                ExpressionNode::Simple(s) => s.content.as_str(),
                ExpressionNode::Compound(c) => c.loc.source.as_str(),
            };
            let loc = exp.loc();

            // Collect expression
            if self.options.collect_template_expressions {
                let scope_id = self.summary.scopes.current_id();
                self.summary
                    .template_expressions
                    .push(crate::analysis::TemplateExpression {
                        content: CompactString::new(content),
                        kind: crate::analysis::TemplateExpressionKind::VBind,
                        start: loc.start.offset,
                        end: loc.end.offset,
                        scope_id,
                        vif_guard: self.current_vif_guard(),
                    });
            }

            // Track $attrs usage
            if content.contains("$attrs") {
                self.summary.template_info.uses_attrs = true;
                if dir.arg.is_none() && content.trim() == "$attrs" {
                    self.summary.template_info.binds_attrs_explicitly = true;
                }
            }

            // Handle bind callbacks
            if self.options.analyze_template_scopes {
                if let Some(params) = profile!(
                    "croquis.template.callback.extract_params",
                    extract_inline_callback_params(content)
                ) {
                    let context = dir
                        .arg
                        .as_ref()
                        .map(|arg| match arg {
                            ExpressionNode::Simple(s) => {
                                cstr!(":{}callback", s.content)
                            }
                            ExpressionNode::Compound(c) => {
                                cstr!(":{}callback", c.loc.source)
                            }
                        })
                        .unwrap_or_else(|| CompactString::const_new(":bind callback"));

                    self.summary.scopes.enter_template_callback_scope(
                        CallbackScopeData {
                            param_names: params.into_iter().collect(),
                            context,
                        },
                        dir.loc.start.offset,
                        dir.loc.end.offset,
                    );

                    let params_added: Vec<CompactString> = self
                        .summary
                        .scopes
                        .current_scope()
                        .bindings()
                        .map(|(name, _)| CompactString::new(name))
                        .collect();

                    for param in &params_added {
                        scope_vars.push(param.clone());
                    }

                    if self.options.detect_undefined && self.script_analyzed {
                        profile!(
                            "croquis.template.v_bind.refs",
                            self.check_expression_refs(exp, scope_vars, dir.loc.start.offset)
                        );
                    }

                    for _ in &params_added {
                        scope_vars.pop();
                    }

                    self.summary.scopes.exit_scope();
                } else if self.options.detect_undefined && self.script_analyzed {
                    profile!(
                        "croquis.template.v_bind.refs",
                        self.check_expression_refs(exp, scope_vars, dir.loc.start.offset)
                    );
                }
            }
        }
    }
}
