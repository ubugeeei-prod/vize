//! a11y/role-img-has-name
//!
//! WAI-ARIA 1.2 marks the `img` role as requiring an accessible name.
//! An empty `alt` removes the implicit image role, so `role="img"` puts it back
//! without a name. `a11y/no-redundant-roles` keeps that attribute on purpose.

use crate::context::LintContext;
use crate::diagnostic::Severity;
use crate::rule::{Rule, RuleCategory, RuleMeta};
use vize_relief::{ElementNode, ExpressionNode, PropNode};

use super::helpers::string_literal_value;

static META: RuleMeta = RuleMeta {
    name: "a11y/role-img-has-name",
    description: "Require an accessible name on elements with role=\"img\"",
    category: RuleCategory::Accessibility,
    fixable: false,
    default_severity: Severity::Warning,
};

const MESSAGE: &str = "Elements with role=\"img\" require an accessible name";
const HELP: &str =
    "Add aria-label, aria-labelledby, or title. An img can also use a non-empty alt.";

/// Report `role="img"` when the element has no accessible name.
#[derive(Default)]
pub struct RoleImgHasName;

enum Attr<'a> {
    Missing,
    Dynamic,
    Static(&'a str),
}

fn attribute<'a>(element: &'a ElementNode<'a>, name: &str) -> Attr<'a> {
    for prop in &element.props {
        match prop {
            PropNode::Attribute(attr) if attr.name == name => {
                return Attr::Static(attr.value.as_ref().map(|value| value.content).unwrap_or(""));
            }
            PropNode::Directive(dir) if dir.name == "bind" => {
                let Some(ExpressionNode::Simple(arg)) = &dir.arg else {
                    continue;
                };
                if !arg.is_static || arg.content != name {
                    continue;
                }
                let Some(ExpressionNode::Simple(expression)) = &dir.exp else {
                    return Attr::Dynamic;
                };
                if let Some(value) = string_literal_value(expression.content) {
                    return Attr::Static(value);
                }
                return Attr::Dynamic;
            }
            _ => {}
        }
    }
    Attr::Missing
}

fn role_is_img(value: &str) -> bool {
    value.split_whitespace().any(|token| token == "img")
}

fn supplies_name(attr: Attr<'_>) -> bool {
    match attr {
        Attr::Dynamic => true,
        Attr::Static(value) => !value.trim().is_empty(),
        Attr::Missing => false,
    }
}

fn has_accessible_name(element: &ElementNode<'_>) -> bool {
    supplies_name(attribute(element, "aria-label"))
        || supplies_name(attribute(element, "aria-labelledby"))
        || supplies_name(attribute(element, "title"))
        || (element.tag == "img" && supplies_name(attribute(element, "alt")))
}

impl Rule for RoleImgHasName {
    fn meta(&self) -> &'static RuleMeta {
        &META
    }

    fn enter_element<'a>(&self, ctx: &mut LintContext<'a>, element: &ElementNode<'a>) {
        let Attr::Static(role) = attribute(element, "role") else {
            return;
        };
        if role_is_img(role) && !has_accessible_name(element) {
            ctx.warn_with_help(MESSAGE, &element.loc, HELP);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::linter::Linter;

    const ISSUE: &str = r#"<script setup lang="ts">
const source = "/a.png";
</script>

<template>
  <div>
    <img :src="source" alt="" role="img" />
    <img :src="source" role="img" />
    <img :src="source" alt="" />
  </div>
</template>
"#;

    fn lines(source: &str) -> Vec<u32> {
        let result = Linter::new()
            .with_enabled_rules(Some(vec!["a11y/role-img-has-name".into()]))
            .lint_sfc(source, "RoleImage.vue");
        result
            .diagnostics
            .into_iter()
            .filter(|diagnostic| diagnostic.rule_name == "a11y/role-img-has-name")
            .map(|diagnostic| {
                source[..diagnostic.start as usize]
                    .bytes()
                    .filter(|byte| *byte == b'\n')
                    .count() as u32
                    + 1
            })
            .collect()
    }

    #[test]
    fn empty_alt_with_role_img_and_a_bare_role_img_are_unnamed() {
        assert_eq!(lines(ISSUE), vec![7, 8]);
    }

    #[test]
    fn a_non_empty_alt_names_the_image_role() {
        let source = "<template>\n  <img src=\"/a.png\" alt=\"Network diagram\" role=\"img\" />\n</template>\n";
        assert!(lines(source).is_empty());
    }

    #[test]
    fn aria_label_or_title_names_a_non_img_element() {
        let labelled =
            "<template>\n  <span role=\"img\" aria-label=\"Status\"></span>\n</template>\n";
        let titled = "<template>\n  <span role=\"img\" title=\"Status\"></span>\n</template>\n";
        assert!(lines(labelled).is_empty());
        assert!(lines(titled).is_empty());
    }

    #[test]
    fn a_span_with_role_img_and_no_name_is_reported() {
        let source = "<template>\n  <span role=\"img\"></span>\n</template>\n";
        assert_eq!(lines(source), vec![2]);
    }
}
