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
            report_recovered_unparenthesized_v_for(ctx, self);
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
        report_recovered_empty_v_for(ctx, self);
    }
}

fn report_recovered_unparenthesized_v_for(ctx: &mut LintContext<'_>, rule: &NoUnusedVars) {
    let source = ctx.source;
    let bytes = source.as_bytes();
    let mut search_from = 0;
    while let Some(relative) = source
        .get(search_from..)
        .and_then(|rest| rest.find("v-for"))
    {
        let directive_start = search_from + relative;
        search_from = directive_start + "v-for".len();
        let mut cursor = search_from;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        let value_start = cursor + 1;
        let Some(value_end_relative) = source
            .get(value_start..)
            .and_then(|value| value.find(quote as char))
        else {
            continue;
        };
        let value_end = value_start + value_end_relative;
        let Some(value) = source.get(value_start..value_end) else {
            continue;
        };
        let Some((aliases, _)) = value.split_once(" in ") else {
            continue;
        };
        if aliases.trim_start().starts_with('(') || !aliases.contains(',') {
            continue;
        }
        let Some(open) = source
            .get(..directive_start)
            .and_then(|prefix| prefix.rfind('<'))
        else {
            continue;
        };
        let opening_before = source.get(open..value_start).unwrap_or_default();
        let element_end = authored_element_end(source, open, value_end);
        let authored_after = source.get(value_end..element_end).unwrap_or_default();
        let mut alias_search = value_start;
        for (index, alias) in aliases.split(',').map(str::trim).enumerate() {
            if alias.is_empty()
                || rule.should_ignore(alias)
                || authored_identifier_reference_present(opening_before, alias)
                || authored_identifier_reference_present(authored_after, alias)
            {
                continue;
            }
            let Some(relative_start) = source
                .get(alias_search..value_end)
                .and_then(|remaining| remaining.find(alias))
            else {
                continue;
            };
            let start = alias_search + relative_start;
            alias_search = start + alias.len();
            if ctx.diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_name == META.name && diagnostic.start == start as u32
            }) {
                continue;
            }
            let (message, help) = if index == 0 {
                (
                    cstr!("Variable '{}' is defined by v-for but never used", alias),
                    "If the variable is intentionally unused, prefix it with underscore: _item",
                )
            } else if index == 1 {
                (
                    cstr!(
                        "Key variable '{}' is defined by v-for but never used",
                        alias
                    ),
                    "Consider removing the key variable or prefix it with underscore: _key",
                )
            } else {
                (
                    cstr!(
                        "Index variable '{}' is defined by v-for but never used",
                        alias
                    ),
                    "Consider removing the index variable or prefix it with underscore: _index",
                )
            };
            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    message,
                    start as u32,
                    (start + alias.len()) as u32,
                )
                .with_help(help),
            );
        }
    }
}

fn authored_element_end(source: &str, open: usize, after: usize) -> usize {
    let Some(tag_start) = source.get(open + 1..) else {
        return after;
    };
    let tag_len = tag_start
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        .count();
    let Some(tag) = tag_start.get(..tag_len) else {
        return after;
    };
    let closing = format!("</{tag}");
    source
        .get(after..)
        .and_then(|rest| rest.find(&closing))
        .and_then(|relative| {
            let closing_start = after + relative;
            source
                .get(closing_start..)
                .and_then(|rest| rest.find('>'))
                .map(|end| closing_start + end + 1)
        })
        .unwrap_or(after)
}

fn report_recovered_empty_v_for(ctx: &mut LintContext<'_>, rule: &NoUnusedVars) {
    let source = ctx.source;
    let bytes = source.as_bytes();
    let mut search_from = 0;

    while let Some(relative) = source
        .get(search_from..)
        .and_then(|rest| rest.find("v-for"))
    {
        let directive_start = search_from + relative;
        search_from = directive_start + "v-for".len();
        if directive_start > 0
            && bytes
                .get(directive_start - 1)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        {
            continue;
        }

        let Some(open) = source
            .get(..directive_start)
            .and_then(|prefix| prefix.rfind('<'))
        else {
            continue;
        };
        let Some(close_relative) = source
            .get(directive_start..)
            .and_then(|rest| rest.find('>'))
        else {
            continue;
        };
        let close = directive_start + close_relative;
        if !authored_element_is_empty(source, open, close) {
            continue;
        }

        let mut cursor = search_from;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes.get(cursor) != Some(&b'=') {
            continue;
        }
        cursor += 1;
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        let Some(quote @ (b'\'' | b'"')) = bytes.get(cursor).copied() else {
            continue;
        };
        let value_start = cursor + 1;
        let Some(value_end_relative) = source
            .get(value_start..close)
            .and_then(|value| value.find(quote as char))
        else {
            continue;
        };
        let value_end = value_start + value_end_relative;
        let Some(value) = source.get(value_start..value_end) else {
            continue;
        };
        let Some((aliases, rhs)) = value.split_once(" in ") else {
            continue;
        };
        if !rhs.contains(" in ") || !(rhs.contains('\'') || rhs.contains('"')) {
            continue;
        }

        let opening_before = source.get(open..directive_start).unwrap_or_default();
        let opening_after = source.get(value_end + 1..=close).unwrap_or_default();
        let aliases = aliases.trim();
        let aliases = aliases
            .strip_prefix('(')
            .and_then(|aliases| aliases.strip_suffix(')'))
            .unwrap_or(aliases);
        let mut alias_search = value_start;
        for (index, alias) in aliases.split(',').map(str::trim).enumerate() {
            if alias.is_empty()
                || !alias
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
                || rule.should_ignore(alias)
                || authored_identifier_reference_present(opening_before, alias)
                || authored_identifier_reference_present(opening_after, alias)
            {
                continue;
            }
            let Some(relative_start) = source
                .get(alias_search..value_end)
                .and_then(|remaining| remaining.find(alias))
            else {
                continue;
            };
            let start = alias_search + relative_start;
            alias_search = start + alias.len();
            if ctx.diagnostics.iter().any(|diagnostic| {
                diagnostic.rule_name == META.name && diagnostic.start == start as u32
            }) {
                continue;
            }
            let (message, help) = if index == 0 {
                (
                    cstr!("Variable '{}' is defined by v-for but never used", alias),
                    "If the variable is intentionally unused, prefix it with underscore: _item",
                )
            } else if index == 1 {
                (
                    cstr!(
                        "Key variable '{}' is defined by v-for but never used",
                        alias
                    ),
                    "Consider removing the key variable or prefix it with underscore: _key",
                )
            } else {
                (
                    cstr!(
                        "Index variable '{}' is defined by v-for but never used",
                        alias
                    ),
                    "Consider removing the index variable or prefix it with underscore: _index",
                )
            };
            ctx.report(
                crate::diagnostic::LintDiagnostic::warn(
                    ctx.current_rule,
                    message,
                    start as u32,
                    (start + alias.len()) as u32,
                )
                .with_help(help),
            );
        }
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
