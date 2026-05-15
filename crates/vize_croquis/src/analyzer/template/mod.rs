//! Template AST visiting and analysis.
//!
//! Provides methods for traversing the template AST and collecting:
//! - v-for/v-slot scope variables
//! - Component and directive usage
//! - Undefined reference detection
//! - Template expressions for type checking
//! - Element IDs for cross-file uniqueness checking

mod components;
mod directives;
mod ids;
mod visit_element;

#[cfg(test)]
mod tests;

use super::Analyzer;
use vize_carton::{profile, CompactString};
use vize_relief::ast::{ExpressionNode, RootNode, TemplateChildNode};

impl Analyzer {
    /// Analyze template AST.
    pub fn analyze_template(&mut self, root: &RootNode<'_>) -> &mut Self {
        if !self.options.analyze_template_scopes && !self.options.track_usage {
            return self;
        }

        // Count root-level elements
        let root_element_count = profile!("croquis.template.root_count", {
            let mut root_element_count = 0;
            for child in root.children.iter() {
                if Self::is_element_child(child) {
                    root_element_count += 1;
                }
            }
            root_element_count
        });
        self.summary.template_info.root_element_count = root_element_count;

        // Store template content range
        self.summary.template_info.content_start = root.loc.start.offset;
        self.summary.template_info.content_end = root.loc.end.offset;

        // Single-pass template traversal
        profile!("croquis.template.traverse", {
            for child in root.children.iter() {
                self.visit_template_child(child, &mut Vec::new());
            }
        });

        self
    }

    /// Check if a template child is an actual element.
    pub(super) fn is_element_child(node: &TemplateChildNode<'_>) -> bool {
        match node {
            TemplateChildNode::Element(_) => true,
            TemplateChildNode::If(if_node) => if_node
                .branches
                .first()
                .map(|b| b.children.iter().any(Self::is_element_child))
                .unwrap_or(false),
            TemplateChildNode::For(_) => true,
            _ => false,
        }
    }

    /// Visit template child node.
    pub(super) fn visit_template_child(
        &mut self,
        node: &TemplateChildNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        match node {
            TemplateChildNode::Element(el) => {
                profile!(
                    "croquis.template.visit_element",
                    self.visit_element(el, scope_vars)
                )
            }
            TemplateChildNode::If(if_node) => {
                profile!(
                    "croquis.template.visit_if",
                    self.visit_if(if_node, scope_vars)
                )
            }
            TemplateChildNode::For(for_node) => {
                profile!(
                    "croquis.template.visit_for",
                    self.visit_for(for_node, scope_vars)
                )
            }
            TemplateChildNode::Interpolation(interp) => {
                profile!("croquis.template.interpolation", {
                    let content = match &interp.content {
                        ExpressionNode::Simple(s) => s.content.as_str(),
                        ExpressionNode::Compound(c) => c.loc.source.as_str(),
                    };

                    // Track $attrs usage
                    if content.contains("$attrs") {
                        self.summary.template_info.uses_attrs = true;
                    }

                    if self.options.collect_template_expressions {
                        let loc = interp.content.loc();
                        let scope_id = self.summary.scopes.current_id();
                        self.summary.template_expressions.push(
                            crate::analysis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::analysis::TemplateExpressionKind::Interpolation,
                                start: loc.start.offset,
                                end: loc.end.offset,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }
                    if self.options.detect_undefined && self.script_analyzed {
                        self.check_expression_refs(
                            &interp.content,
                            scope_vars,
                            interp.content.loc().start.offset,
                        );
                    }
                })
            }
            _ => {}
        }
    }
}
