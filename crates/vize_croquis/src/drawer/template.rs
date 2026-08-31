//! Template AST visiting and drawing.
//!
//! Provides methods for traversing the template AST and collecting:
//! - v-for/v-slot scope variables
//! - Component and directive usage
//! - Undefined reference detection
//! - Template expressions for type checking
//! - Element IDs for cross-file uniqueness checking

mod components;
mod directives;
#[cfg(test)]
mod dynamic_names_tests;
mod ids;
#[cfg(test)]
mod legacy_vue2_tests;
mod slot_names;
#[cfg(test)]
mod slot_scope_tests;
mod visit_element;

#[cfg(test)]
mod tests;

use super::Drawer;
use vize_carton::{CompactString, profile};
use vize_relief::{ExpressionNode, RootNode, TemplateChildNode};

impl Drawer {
    /// Draw template AST facts into the croquis.
    pub fn draw_template(&mut self, root: &RootNode<'_>) -> &mut Self {
        if !self.options.analyze_template_scopes && !self.options.track_usage {
            return self;
        }
        self.template_source = root.source.into();

        // Count root-level elements
        let (root_element_count, root_content_range) = profile!("croquis.template.root_count", {
            let mut root_element_count = 0;
            let mut root_content_range = None;
            for child in root.children.iter() {
                if Self::is_element_child(child) {
                    root_element_count += 1;
                    let (start, end) = Self::template_child_range(child);
                    root_content_range = Some(
                        root_content_range
                            .map(|(first_start, _)| (first_start, end))
                            .unwrap_or((start, end)),
                    );
                }
            }
            (root_element_count, root_content_range)
        });
        self.croquis.template_info.root_element_count = root_element_count;

        // Store the authored template root content range. The root node itself
        // may carry a stub range, so derive this from concrete root children.
        if let Some((start, end)) = root_content_range {
            self.croquis.template_info.content_start = start;
            self.croquis.template_info.content_end = end;
        }

        // Keep profiling around the whole traversal instead of every recursive
        // child visit. The traversal itself is the hot path; per-node spans
        // created enough guard/drop overhead to distort normal template walks.
        profile!("croquis.template.traverse", {
            for child in root.children.iter() {
                self.visit_template_child(child, &mut Vec::new());
            }
        });

        self
    }

    /// Compatibility wrapper for the old Analyzer naming.
    #[inline]
    pub fn analyze_template(&mut self, root: &RootNode<'_>) -> &mut Self {
        self.draw_template(root)
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

    fn template_child_range(node: &TemplateChildNode<'_>) -> (u32, u32) {
        let span = match node {
            TemplateChildNode::Element(node) => node.loc.span,
            TemplateChildNode::Text(node) => node.loc.span,
            TemplateChildNode::Comment(node) => node.loc.span,
            TemplateChildNode::Interpolation(node) => node.loc.span,
            TemplateChildNode::If(node) => node.loc.span,
            TemplateChildNode::IfBranch(node) => node.loc.span,
            TemplateChildNode::For(node) => node.loc.span,
            TemplateChildNode::TextCall(node) => node.loc.span,
            TemplateChildNode::CompoundExpression(node) => node.loc.span,
            TemplateChildNode::Hoisted(_) => return (0, 0),
        };
        (span.start, span.end)
    }

    /// Visit template child node.
    pub(super) fn visit_template_child(
        &mut self,
        node: &TemplateChildNode<'_>,
        scope_vars: &mut Vec<CompactString>,
    ) {
        match node {
            TemplateChildNode::Element(el) => self.visit_element(el, scope_vars),
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
                    let compound_content;
                    let content = match &interp.content {
                        ExpressionNode::Simple(s) => s.content,
                        ExpressionNode::Compound(c) => {
                            compound_content =
                                CompactString::new(c.loc.span.slice(&self.template_source));
                            compound_content.as_str()
                        }
                    };

                    // Track $attrs usage
                    if content.contains("$attrs") {
                        self.croquis.template_info.uses_attrs = true;
                    }

                    if self.options.collect_template_expressions {
                        let loc = interp.content.loc();
                        let scope_id = self.croquis.scopes.current_id();
                        self.croquis.template_expressions.push(
                            crate::croquis::TemplateExpression {
                                content: CompactString::new(content),
                                kind: crate::croquis::TemplateExpressionKind::Interpolation,
                                start: loc.span.start,
                                end: loc.span.end,
                                scope_id,
                                vif_guard: self.current_vif_guard(),
                            },
                        );
                    }
                    if self.options.detect_undefined {
                        self.check_expression_refs(&interp.content, scope_vars);
                    }
                })
            }
            _ => {}
        }
    }
}
