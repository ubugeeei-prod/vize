//! vue/no-unused-vars
//!
//! Disallow unused variable definitions in `v-for` and `v-slot` directives.
//!
//! This rule reports variables that are defined in `v-for` or `v-slot` directives
//! but never used in the template. Uses croquis semantic analysis for accurate
//! tracking of variable usage.
//!
//! ## Examples
//!
//! ### Invalid
//! ```vue
//! <!-- 'index' is defined but never used -->
//! <li v-for="(item, index) in items" :key="item.id">{{ item }}</li>
//!
//! <!-- 'foo' is defined but never used -->
//! <template v-slot="{ foo }">
//!   <span>Hello</span>
//! </template>
//! ```
//!
//! ### Valid
//! ```vue
//! <li v-for="(item, index) in items" :key="index">{{ item }}</li>
//! <li v-for="item in items" :key="item.id">{{ item.name }}</li>
//!
//! <template v-slot="{ data }">
//!   <span>{{ data }}</span>
//! </template>
//! ```
//!
//! ## Options
//!
//! An ignore pattern can be configured explicitly; the default matches
//! eslint-plugin-vue and reports underscore-prefixed aliases too.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use oxc_ast::ast::{ArrowFunctionExpression, BindingPattern, IdentifierReference};
use oxc_ast_visit::{Visit, walk::walk_arrow_function_expression};
use vize_croquis::UnusedVarContext;
use vize_l0::String;
use vize_l0::cstr;
use vize_relief::{ElementNode, ExpressionNode, PropNode, RootNode, TemplateChildNode};

mod recovery;

static META: RuleMeta = RuleMeta {
    name: "vue/no-unused-vars",
    description: "Disallow unused variable definitions in v-for and v-slot directives",
    category: RuleCategory::Essential,
    fixable: false,
    default_severity: Severity::Warning,
};

/// Disallow unused v-for and v-slot variables
#[derive(Default)]
pub struct NoUnusedVars {
    /// Optional pattern for variables to ignore.
    ignore_pattern: Option<String>,
}

impl NoUnusedVars {
    /// Check if a variable name should be ignored
    fn should_ignore(&self, name: &str) -> bool {
        // Check custom ignore pattern
        if let Some(pattern) = &self.ignore_pattern {
            if pattern == "^_" {
                return name.starts_with('_');
            }
            if name.starts_with(pattern.as_str()) {
                return true;
            }
        }

        false
    }
}

impl Rule for NoUnusedVars {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn run_on_template<'a>(&self, ctx: &mut LintContext<'a>, root: &RootNode<'a>) {
        // Skip if no analysis available (croquis semantic analysis required)
        if !ctx.has_analysis() {
            recovery::report_unparenthesized_v_for(ctx, self);
            return;
        }

        let Some(analysis) = ctx.analysis() else {
            return;
        };
        let unused_vars = analysis.unused_template_vars();

        for unused in unused_vars {
            // Skip if the variable should be ignored
            if self.should_ignore(unused.name.as_str()) {
                continue;
            }
            if same_element_directive_uses(root, unused.name.as_str(), unused.offset) {
                continue;
            }
            if authored_opening_directive_uses(ctx.source, unused.name.as_str(), unused.offset) {
                continue;
            }

            let (message, help) = match &unused.context {
                UnusedVarContext::VForValue => (
                    cstr!(
                        "Variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "If the variable is intentionally unused, prefix it with underscore: _item",
                ),
                UnusedVarContext::VForKey => (
                    cstr!(
                        "Key variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "Consider removing the key variable or prefix it with underscore: _key",
                ),
                UnusedVarContext::VForIndex => (
                    cstr!(
                        "Index variable '{}' is defined by v-for but never used",
                        unused.name
                    ),
                    "Consider removing the index variable or prefix it with underscore: _index",
                ),
                UnusedVarContext::VSlot { slot_name } => (
                    cstr!(
                        "Slot prop '{}' from slot '{}' is defined but never used",
                        unused.name,
                        slot_name
                    ),
                    "Consider removing unused slot props or prefix with underscore",
                ),
            };

            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    message,
                    unused.offset,
                    unused.offset + unused.name.len() as u32,
                )
                .with_help(help),
            );
        }

        // Some template generators parenthesize a string containing ` in ` as
        // the v-for source. Croquis deliberately rejects that ambiguous
        // recovery shape, but eslint-plugin-vue still recovers the aliases.
        // Keep this fallback narrow (authored empty elements only) so malformed
        // hand-written templates do not acquire speculative scope diagnostics.
        recovery::report_empty_v_for(ctx, self);
    }
}

include!("no_unused_vars/authored_references.rs");
fn same_element_directive_uses(root: &RootNode<'_>, name: &str, offset: u32) -> bool {
    children_contain_same_element_use(&root.children, name, offset)
}

fn children_contain_same_element_use(
    children: &[TemplateChildNode<'_>],
    name: &str,
    offset: u32,
) -> bool {
    for child in children {
        match child {
            TemplateChildNode::Element(element) => {
                if element_owns_template_binding(element, offset)
                    && element.props.iter().any(|prop| {
                        let PropNode::Directive(directive) = prop else {
                            return false;
                        };
                        !matches!(directive.name, "for" | "slot")
                            && directive
                                .exp
                                .as_ref()
                                .is_some_and(|expression| expression_references(expression, name))
                    })
                {
                    return true;
                }
                if children_contain_same_element_use(&element.children, name, offset) {
                    return true;
                }
            }
            TemplateChildNode::If(node) => {
                if node
                    .branches
                    .iter()
                    .any(|branch| children_contain_same_element_use(&branch.children, name, offset))
                {
                    return true;
                }
            }
            TemplateChildNode::IfBranch(branch) => {
                if children_contain_same_element_use(&branch.children, name, offset) {
                    return true;
                }
            }
            TemplateChildNode::For(node)
                if children_contain_same_element_use(&node.children, name, offset) =>
            {
                return true;
            }
            _ => {}
        }
    }
    false
}

fn element_owns_template_binding(element: &ElementNode<'_>, offset: u32) -> bool {
    element.props.iter().any(|prop| match prop {
        PropNode::Directive(directive) => {
            matches!(directive.name, "for" | "slot")
                && directive.loc.span.start <= offset
                && offset < directive.loc.span.end
        }
        PropNode::Attribute(attribute) => {
            matches!(attribute.name, "slot-scope" | "scope")
                && attribute.loc.span.start <= offset
                && offset < attribute.loc.span.end
        }
    })
}

fn expression_references(expression: &ExpressionNode<'_>, name: &str) -> bool {
    let ExpressionNode::Simple(simple) = expression else {
        return false;
    };
    let Some(parsed) = simple.js_ast.and_then(|ast| ast.as_expression()) else {
        return false;
    };
    let mut finder = IdentifierFinder { name, found: false };
    finder.visit_expression(parsed.ast);
    finder.found
}

struct IdentifierFinder<'a> {
    name: &'a str,
    found: bool,
}

impl<'a> Visit<'a> for IdentifierFinder<'_> {
    fn visit_identifier_reference(&mut self, identifier: &IdentifierReference<'a>) {
        self.found |= identifier.name.as_str() == self.name;
    }

    fn visit_arrow_function_expression(&mut self, arrow: &ArrowFunctionExpression<'a>) {
        if arrow
            .params
            .items
            .iter()
            .any(|parameter| binding_binds(&parameter.pattern, self.name))
        {
            return;
        }
        walk_arrow_function_expression(self, arrow);
    }
}

fn binding_binds(pattern: &BindingPattern<'_>, name: &str) -> bool {
    match pattern {
        BindingPattern::BindingIdentifier(identifier) => identifier.name == name,
        BindingPattern::AssignmentPattern(assignment) => binding_binds(&assignment.left, name),
        BindingPattern::ObjectPattern(object) => {
            object
                .properties
                .iter()
                .any(|property| binding_binds(&property.value, name))
                || object
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binding_binds(&rest.argument, name))
        }
        BindingPattern::ArrayPattern(array) => {
            array
                .elements
                .iter()
                .flatten()
                .any(|element| binding_binds(element, name))
                || array
                    .rest
                    .as_ref()
                    .is_some_and(|rest| binding_binds(&rest.argument, name))
        }
    }
}

#[cfg(test)]
mod tests;
