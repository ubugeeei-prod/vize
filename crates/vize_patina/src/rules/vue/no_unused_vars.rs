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
    }
}

fn authored_opening_directive_uses(source: &str, name: &str, offset: u32) -> bool {
    let offset = offset as usize;
    let Some(open) = source.get(..offset).and_then(|prefix| prefix.rfind('<')) else {
        return false;
    };
    let bytes = source.as_bytes();
    let mut cursor = open + 1;
    while cursor < bytes.len() {
        while bytes.get(cursor).is_some_and(u8::is_ascii_whitespace) {
            cursor += 1;
        }
        if bytes
            .get(cursor)
            .is_none_or(|byte| matches!(byte, b'>' | b'/'))
        {
            break;
        }
        let attr_start = cursor;
        while bytes
            .get(cursor)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b'=' | b'>' | b'/'))
        {
            cursor += 1;
        }
        let attr = source.get(attr_start..cursor).unwrap_or_default();
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
        cursor += 1;
        let value_start = cursor;
        while bytes.get(cursor).is_some_and(|byte| *byte != quote) {
            cursor += 1;
        }
        let value_end = cursor;
        cursor += usize::from(bytes.get(cursor) == Some(&quote));
        if matches!(attr, "v-for" | "v-slot")
            || attr.starts_with("v-slot:")
            || attr.starts_with('#')
            || !matches!(attr.as_bytes().first(), Some(b':' | b'@' | b'#'))
                && !attr.starts_with("v-")
        {
            continue;
        }
        if source
            .get(value_start..value_end)
            .is_some_and(|value| identifier_token_present(value, name))
        {
            return true;
        }
    }
    false
}

fn identifier_token_present(source: &str, name: &str) -> bool {
    if let Some(arrow) = source.find("=>")
        && arrow_parameter_binds(source.get(..arrow).unwrap_or_default(), name)
    {
        return false;
    }
    source.match_indices(name).any(|(start, matched)| {
        let end = start + matched.len();
        !source
            .as_bytes()
            .get(start.wrapping_sub(1))
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
            && !source
                .as_bytes()
                .get(end)
                .is_some_and(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$'))
    })
}

fn arrow_parameter_binds(prefix: &str, name: &str) -> bool {
    let prefix = prefix.trim_end();
    if prefix == name {
        return true;
    }
    let Some(parameters) = prefix
        .strip_suffix(')')
        .and_then(|text| text.rsplit_once('('))
    else {
        return false;
    };
    parameters
        .1
        .split(',')
        .map(str::trim)
        .any(|parameter| parameter == name)
}

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
mod tests {
    use super::NoUnusedVars;
    use crate::linter::Linter;
    use crate::rule::{Rule, RuleCategory};

    #[test]
    fn test_meta() {
        let rule = NoUnusedVars::default();
        assert_eq!(rule.meta().name, "vue/no-unused-vars");
        assert_eq!(rule.meta().category, RuleCategory::Essential);
    }

    #[test]
    fn test_default_does_not_ignore_underscore_prefix() {
        let rule = NoUnusedVars::default();
        assert!(!rule.should_ignore("_item"));
        assert!(!rule.should_ignore("_"));
        assert!(!rule.should_ignore("item"));
        assert!(!rule.should_ignore("index"));
    }

    #[test]
    fn same_element_v_if_counts_as_upstream_usage() {
        let source = r#"<template>
  <img v-for="(url, index) in urls" v-if="index === selected" :src="url" />
</template>"#;
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
            .lint_sfc(source, "Probe.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| diagnostic.rule_name != "vue/no-unused-vars"),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn same_element_slot_prop_directive_counts_as_usage() {
        let source = r#"<template>
  <FormKit v-slot="{ state: { loading }, unused }" :label="loading ? 'Wait' : 'Go'" />
</template>"#;
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
            .lint_sfc(source, "Probe.vue");
        let messages: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(messages.len(), 1, "{:?}", result.diagnostics);
        assert!(messages[0].contains("unused"), "{messages:?}");
    }

    #[test]
    fn authored_directive_use_survives_a_repaired_subtree() {
        let source = r#"<template>
  <a><a v-for="(file, index) in files" @click="remove(index)">{{ file }}</a></a>
</template>"#;
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
            .lint_sfc(source, "Probe.vue");
        assert!(
            result
                .diagnostics
                .iter()
                .all(|diagnostic| !diagnostic.message.contains("index")),
            "{:?}",
            result.diagnostics
        );
    }

    #[test]
    fn vue2_slot_scope_and_unparenthesized_v_for_report_unused_aliases() {
        let source = r#"<template>
  <Child><template slot-scope="{ used, unused }">{{ used }}</template></Child>
  <div v-for="item,index in items">{{ item }}</div>
</template>"#;
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["vue/no-unused-vars".into()]))
            .lint_sfc(source, "Probe.vue");
        let messages: Vec<_> = result
            .diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.rule_name == "vue/no-unused-vars")
            .map(|diagnostic| diagnostic.message.as_str())
            .collect();
        assert_eq!(messages.len(), 2, "{:?}", result.diagnostics);
        assert!(messages.iter().any(|message| message.contains("unused")));
        assert!(messages.iter().any(|message| message.contains("index")));
    }
}
