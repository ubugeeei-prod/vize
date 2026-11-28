//! Apply eslint-plugin-vue template rule options by replacing registered rules.
//!
//! Options never enable a rule: a rule the preset did not register stays absent.

use super::super::config::Linter;
use crate::rule::Rule;
use crate::rules::vue::{
    NoDuplicateAttributes, NoLoneTemplate, NoMultipleTemplateRoot, NoStaticInlineStyles,
    NoUnusedComponents, NoVHtml,
};
use vize_l0::config::TemplateRuleOptions;

pub(super) fn apply(mut linter: Linter, options: &TemplateRuleOptions) -> Linter {
    if let Some(value) = options.no_duplicate_attributes {
        replace(
            &mut linter,
            NoDuplicateAttributes {
                allow_coexist_class: value.allow_coexist_class.unwrap_or(true),
                allow_coexist_style: value.allow_coexist_style.unwrap_or(true),
            },
        );
    }
    if let Some(value) = options.no_multiple_template_root {
        replace(
            &mut linter,
            NoMultipleTemplateRoot {
                disallow_comments: value.disallow_comments,
            },
        );
    }
    if let Some(value) = options.no_lone_template {
        replace(
            &mut linter,
            NoLoneTemplate {
                ignore_accessible: value.ignore_accessible,
            },
        );
    }
    if let Some(value) = options.no_static_inline_styles {
        replace(
            &mut linter,
            NoStaticInlineStyles {
                allow_binding: value.allow_binding,
            },
        );
    }
    if let Some(value) = options.no_unused_components {
        replace(
            &mut linter,
            NoUnusedComponents {
                report_with_dynamic_is: !value.ignore_when_binding_present.unwrap_or(true),
                ..NoUnusedComponents::default()
            },
        );
    }
    if let Some(value) = &options.no_v_html {
        let rule = match value.ignore_pattern.as_deref() {
            Some(pattern) if !pattern.is_empty() => NoVHtml::with_ignore_pattern(pattern),
            _ => NoVHtml::default(),
        };
        replace(&mut linter, rule);
    }
    linter
}

fn replace(linter: &mut Linter, rule: impl Rule + 'static) {
    if linter.registry.has_rule(rule.meta().name) {
        linter.registry.replace(Box::new(rule));
    }
}

#[cfg(test)]
mod tests;
