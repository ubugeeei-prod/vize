//! Element ID collection and identifier/scope tracking.
//!
//! Handles:
//! - Element ID and ID-reference attribute detection (static and dynamic)
//! - v-for loop scope detection
//! - Expression undefined-reference checking

use crate::analysis::{ElementIdInfo, ElementIdKind, UndefinedRef};
use vize_carton::{profile, CompactString};
use vize_relief::ast::{ElementNode, ExpressionNode, PropNode};

use super::super::helpers::{extract_identifiers_oxc, is_keyword};
use super::super::Analyzer;

/// Attributes that take ID references (not the ID itself).
const ID_REFERENCE_ATTRIBUTES: &[&str] = &[
    "for",              // <label for="...">
    "aria-labelledby",  // ARIA reference
    "aria-describedby", // ARIA reference
    "aria-controls",    // ARIA reference
    "aria-owns",        // ARIA reference
    "aria-activedescendant",
    "aria-flowto",
    "aria-details",
    "aria-errormessage",
    "headers", // <td headers="...">
    "list",    // <input list="...">
    "form",    // <button form="...">
    "popovertarget",
    "anchor",
];

/// Get the `ElementIdKind` for an attribute name.
#[inline]
fn get_id_kind(attr_name: &str) -> Option<ElementIdKind> {
    if attr_name == "id" {
        Some(ElementIdKind::Id)
    } else if attr_name == "for" {
        Some(ElementIdKind::For)
    } else if attr_name.starts_with("aria-") && ID_REFERENCE_ATTRIBUTES.contains(&attr_name) {
        Some(ElementIdKind::AriaReference)
    } else if ID_REFERENCE_ATTRIBUTES.contains(&attr_name) {
        Some(ElementIdKind::OtherReference)
    } else {
        None
    }
}

impl Analyzer {
    /// Collect element IDs from an element node.
    ///
    /// Collects both:
    /// - Static IDs: `id="foo"`, `for="bar"`, etc.
    /// - Dynamic IDs: `:id="expr"`, `:for="expr"`, etc.
    pub(in crate::analyzer) fn collect_element_ids(&mut self, el: &ElementNode<'_>) {
        let scope_id = self.summary.scopes.current_id();
        let in_loop = self.is_in_vfor_scope();

        for prop in &el.props {
            match prop {
                PropNode::Attribute(attr) => {
                    let attr_name = attr.name.as_str();
                    if let Some(kind) = get_id_kind(attr_name) {
                        if let Some(value) = &attr.value {
                            self.summary.element_ids.push(ElementIdInfo {
                                value: value.content.clone(),
                                start: attr.loc.start.offset,
                                end: attr.loc.end.offset,
                                is_static: true,
                                in_loop,
                                scope_id,
                                kind,
                            });
                        }
                    }
                }
                PropNode::Directive(dir) => {
                    if dir.name == "bind" {
                        if let Some(ref arg) = dir.arg {
                            let arg_name = match arg {
                                ExpressionNode::Simple(s) => s.content.as_str(),
                                ExpressionNode::Compound(c) => c.loc.source.as_str(),
                            };

                            if let Some(kind) = get_id_kind(arg_name) {
                                if let Some(ref exp) = dir.exp {
                                    let content = match exp {
                                        ExpressionNode::Simple(s) => s.content.clone(),
                                        ExpressionNode::Compound(c) => {
                                            CompactString::new(c.loc.source.as_str())
                                        }
                                    };

                                    // Check if it's a static string literal
                                    let is_static = Self::is_static_string(&content);

                                    self.summary.element_ids.push(ElementIdInfo {
                                        value: if is_static {
                                            Self::extract_string_value(&content)
                                        } else {
                                            content
                                        },
                                        start: dir.loc.start.offset,
                                        end: dir.loc.end.offset,
                                        is_static,
                                        in_loop,
                                        scope_id,
                                        kind,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    /// Check if the current scope is inside a v-for loop.
    fn is_in_vfor_scope(&self) -> bool {
        use crate::scope::ScopeKind;

        let current_id = self.summary.scopes.current_id();
        let mut to_visit = vec![current_id];
        let mut visited_count = 0;
        const MAX_VISITS: usize = 50;

        while let Some(scope_id) = to_visit.pop() {
            if visited_count >= MAX_VISITS {
                break;
            }
            visited_count += 1;

            if let Some(scope) = self.summary.scopes.get_scope(scope_id) {
                if scope.kind == ScopeKind::VFor {
                    return true;
                }
                // Add parents to visit
                for &parent in &scope.parents {
                    to_visit.push(parent);
                }
            }
        }

        false
    }

    /// Check if an expression is a static string literal.
    fn is_static_string(expr: &str) -> bool {
        let trimmed = expr.trim();
        (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('`') && trimmed.ends_with('`') && !trimmed.contains("${"))
    }

    /// Extract the value from a static string literal.
    fn extract_string_value(expr: &str) -> CompactString {
        let trimmed = expr.trim();
        if (trimmed.starts_with('\'') && trimmed.ends_with('\''))
            || (trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('`') && trimmed.ends_with('`'))
        {
            CompactString::new(&trimmed[1..trimmed.len() - 1])
        } else {
            CompactString::new(trimmed)
        }
    }

    /// Check expression for undefined references.
    pub(super) fn check_expression_refs(
        &mut self,
        expr: &ExpressionNode<'_>,
        scope_vars: &[CompactString],
        base_offset: u32,
    ) {
        let content = match expr {
            ExpressionNode::Simple(s) => s.content.as_str(),
            ExpressionNode::Compound(c) => c.loc.source.as_str(),
        };

        for ident in profile!(
            "croquis.template.expression.extract_identifiers",
            extract_identifiers_oxc(content)
        ) {
            let ident_str = ident.as_str();

            let in_scope_vars = scope_vars.iter().any(|v| v.as_str() == ident_str);
            let in_bindings = self.summary.bindings.contains(ident_str);
            let in_scope_chain = self.summary.scopes.is_defined(ident_str);

            let is_builtin = crate::builtins::is_js_global(ident_str)
                || crate::builtins::is_vue_builtin(ident_str)
                || crate::builtins::is_event_local(ident_str)
                || is_keyword(ident_str);

            let is_defined = in_scope_vars || in_bindings || in_scope_chain || is_builtin;

            if is_defined && !is_builtin {
                self.summary.scopes.mark_used(ident_str);
            } else if !is_defined {
                let ident_offset_in_content = content.find(ident_str).unwrap_or(0) as u32;
                self.summary.undefined_refs.push(UndefinedRef {
                    name: ident,
                    offset: base_offset + ident_offset_in_content,
                    context: CompactString::new("template expression"),
                });
            }
        }
    }
}
