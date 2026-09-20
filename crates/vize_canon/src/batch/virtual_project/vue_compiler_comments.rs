//! Per-file Vue compiler options belong only to top-level SFC comments.

use crate::virtual_ts::{ResolveStyleClassNames, VirtualTsCheckOptions};
use vize_atelier_core::{ParseMode, ParserOptions, parser::parse_with_options};
use vize_carton::Allocator;
use vize_relief::TemplateChildNode;

pub(super) fn apply(source: &str, mut options: VirtualTsCheckOptions) -> VirtualTsCheckOptions {
    if ![
        "@strictTemplates",
        "@checkUnknownProps",
        "@checkUnknownComponents",
        "@checkUnknownEvents",
        "@strictCssModules",
        "@inferComponentDollarEl",
        "@inferTemplateDollarEl",
        "@inferTemplateDollarSlots",
        "@inferTemplateDollarAttrs",
        "@fallthroughAttributes",
        "@checkRequiredFallthroughAttributes",
        "@resolveStyleClassNames",
        "@resolveStyleImports",
        "@jsxSlots",
    ]
    .iter()
    .any(|option| source.contains(option))
    {
        return options;
    }
    let allocator = Allocator::new();
    let (root, _) = parse_with_options(
        &allocator,
        source,
        ParserOptions {
            mode: ParseMode::Sfc,
            comments: true,
            is_native_tag: Some(|_| true),
            is_pre_tag: |_| true,
            ..Default::default()
        },
    );
    let mut strict = None;
    let mut unknown_props = None;
    let mut unknown_components = None;
    let mut unknown_events = None;
    for child in &root.children {
        let TemplateChildNode::Comment(comment) = child else {
            continue;
        };
        // The upstream parser consumes the first option line per comment;
        // repeated top-level comments override earlier values of that option.
        let Some((key, value)) = comment.content.lines().find_map(|line| {
            line.trim()
                .strip_prefix('@')?
                .split_once(char::is_whitespace)
        }) else {
            continue;
        };
        let Ok(value) = serde_json::from_str::<bool>(value.trim()) else {
            continue;
        };
        match key {
            "strictTemplates" => strict = Some(value),
            "checkUnknownProps" => unknown_props = Some(value),
            "checkUnknownComponents" => unknown_components = Some(value),
            "checkUnknownEvents" => unknown_events = Some(value),
            "strictCssModules" => options.strict_css_modules = value,
            "inferComponentDollarEl" => options.infer_component_dollar_el = value,
            "inferTemplateDollarEl" => options.infer_template_dollar_el = value,
            "inferTemplateDollarSlots" => options.infer_template_dollar_slots = value,
            "inferTemplateDollarAttrs" => options.infer_template_dollar_attrs = value,
            "fallthroughAttributes" => options.fallthrough_attributes = value,
            "checkRequiredFallthroughAttributes" => {
                options.check_required_fallthrough_attributes = value;
            }
            "resolveStyleClassNames" => {
                options.resolve_style_class_names = if value {
                    ResolveStyleClassNames::All
                } else {
                    ResolveStyleClassNames::None
                };
            }
            "resolveStyleImports" => options.resolve_style_imports = value,
            "jsxSlots" => options.jsx_slots = value,
            _ => {}
        }
    }
    if let Some(value) = unknown_props.or(strict) {
        options.check_unknown_props = value;
    }
    if let Some(value) = unknown_components.or(strict) {
        options.check_unknown_components = value;
    }
    if let Some(value) = unknown_events.or(strict) {
        options.check_unknown_events = value;
    }
    options
}

#[cfg(test)]
mod tests {
    use super::apply;
    use crate::virtual_ts::VirtualTsCheckOptions;

    #[test]
    fn check_required_fallthrough_attributes_is_a_per_file_option() {
        let defaults = VirtualTsCheckOptions::default();
        assert!(!defaults.check_required_fallthrough_attributes);
        let options = apply(
            "<!-- @checkRequiredFallthroughAttributes true -->\n<template><div /></template>",
            defaults,
        );
        assert!(options.check_required_fallthrough_attributes);
        assert!(
            !apply(
                "<template><!-- @checkRequiredFallthroughAttributes true --><div /></template>",
                defaults,
            )
            .check_required_fallthrough_attributes
        );
    }

    #[test]
    fn only_top_level_options_override_the_project_setting() {
        let defaults = VirtualTsCheckOptions {
            check_unknown_props: false,
            ..Default::default()
        };
        for source in [
            "<!-- @strictTemplates true --><template><div /></template>",
            "<template><div /></template><!-- @checkUnknownProps true -->",
            "<!-- @strictTemplates false --><!-- @strictTemplates true -->",
        ] {
            assert!(apply(source, defaults).check_unknown_props, "{source}");
        }
        for source in [
            "<template><!-- @strictTemplates true --><div /></template>",
            "<script>const text = '<!-- @strictTemplates true -->';</script>",
            "<style>/* <!-- @strictTemplates true --> */</style>",
            "<docs><!-- @strictTemplates true --></docs>",
            "<!-- prose @strictTemplates true -->",
            "<!-- @strictTemplates \"true\" -->",
            "<!-- @checkUnknownProps false --><!-- @strictTemplates true -->",
            "<!-- @strictTemplates true --><!-- @strictTemplates false -->",
        ] {
            assert!(!apply(source, defaults).check_unknown_props, "{source}");
        }
    }
}
