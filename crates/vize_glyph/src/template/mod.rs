//! High-performance template formatting for Vue SFC.
//!
//! Features:
//! - Proper indentation and nesting
//! - Directive shorthand normalization (`v-bind:` -> `:`, `v-on:` -> `@`, `v-slot:` -> `#`)
//! - Interpolation spacing normalization (`{{expr}}` -> `{{ expr }}`)
//! - JS expression formatting in directive values via oxc_formatter
//! - Attribute sorting following Vue style guide order
//! - `single_attribute_per_line` support with `bracket_same_line`

mod attributes;
mod directives;
mod formatter;
mod helpers;

use crate::{error::FormatError, options::FormatOptions};
use vize_carton::String;

use formatter::TemplateFormatter;
use helpers::is_whitespace;

/// Format Vue template content.
#[inline]
pub fn format_template_content(
    source: &str,
    options: &FormatOptions,
) -> Result<String, FormatError> {
    let bytes = source.as_bytes();

    // Fast path: all whitespace
    if bytes.iter().all(|&b| is_whitespace(b)) {
        return Ok(String::default());
    }

    let formatter = TemplateFormatter::new(options);
    formatter.format(bytes)
}

#[cfg(test)]
mod tests {
    use super::{attributes, directives, format_template_content, formatter, helpers};
    use crate::options::{AttributeSortOrder, FormatOptions};
    use attributes::attribute_priority;
    use directives::{custom_attribute_priority, format_v_for_expression, matches_attr_pattern};
    use formatter::format_interpolations;
    use helpers::{is_tag_name_char, is_void_element_str};
    use vize_carton::ToCompactString;

    #[test]
    fn test_format_simple_template() {
        let source = "<div>Hello</div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_nested_template() {
        let source = "<div><span>Hello</span></div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_with_attributes() {
        let source = r#"<div class="container" id="main">Content</div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_format_self_closing() {
        let source = "<input type=\"text\" />";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_directive_shorthand_v_bind() {
        let source = r#"<div v-bind:class="active"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_directive_shorthand_v_on() {
        let source = r#"<div v-on:click="handler"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_directive_shorthand_v_slot() {
        let source = r#"<template v-slot:default="props"></template>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_interpolation_spacing_normalized() {
        let options = FormatOptions::default();
        let result = format_interpolations("{{count}}", &options);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_interpolation_already_spaced() {
        let options = FormatOptions::default();
        let result = format_interpolations("{{ count }}", &options);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_interpolation_in_text() {
        let options = FormatOptions::default();
        let result = format_interpolations("Hello {{name}} world", &options);
        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_v_for_normalization() {
        let result = format_v_for_expression("(item,index) in items");
        assert_eq!(result, "(item, index) in items");
    }

    #[test]
    fn test_v_for_simple() {
        let result = format_v_for_expression("item in items");
        assert_eq!(result, "item in items");
    }

    #[test]
    fn test_attribute_sorting() {
        let source =
            r#"<div :class="cls" v-if="show" v-for="item in items" @click="handle"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        let vfor_pos = result.find("v-for").unwrap();
        let vif_pos = result.find("v-if").unwrap();
        let class_pos = result.find(":class").unwrap();
        let click_pos = result.find("@click").unwrap();

        assert!(vfor_pos < vif_pos, "v-for should come before v-if");
        assert!(vif_pos < class_pos, "v-if should come before :class");
        assert!(class_pos < click_pos, ":class should come before @click");
    }

    #[test]
    fn test_multiline_attributes() {
        let source = r#"<div class="container" id="main" @click="handler">Content</div>"#;
        let mut options = FormatOptions::default();
        options.single_attribute_per_line = true;
        let result = format_template_content(source, &options).unwrap();

        // Each attribute should be on its own line
        let lines: Vec<&str> = result.lines().collect();
        assert!(lines.len() > 2, "Should have multiple lines for attributes");
    }

    #[test]
    fn test_void_elements() {
        assert!(is_void_element_str("br"));
        assert!(is_void_element_str("img"));
        assert!(is_void_element_str("input"));
        assert!(!is_void_element_str("div"));
        assert!(!is_void_element_str("span"));
    }

    #[test]
    fn test_is_tag_name_char() {
        assert!(is_tag_name_char(b'a'));
        assert!(is_tag_name_char(b'Z'));
        assert!(is_tag_name_char(b'0'));
        assert!(is_tag_name_char(b'-'));
        assert!(is_tag_name_char(b'_'));
        assert!(!is_tag_name_char(b' '));
        assert!(!is_tag_name_char(b'>'));
    }

    #[test]
    fn test_v_else_boolean_attribute() {
        let source = r#"<div v-if="show">A</div><div v-else>B</div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_html_comment() {
        let source = "<!-- This is a comment -->\n<div>Content</div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_attribute_priority_order() {
        assert!(attribute_priority("is") < attribute_priority("v-for"));
        assert!(attribute_priority("v-for") < attribute_priority("v-if"));
        assert!(attribute_priority("v-if") < attribute_priority("v-show"));
        assert!(attribute_priority("v-show") < attribute_priority("id"));
        assert!(attribute_priority("id") < attribute_priority("ref"));
        assert!(attribute_priority("ref") < attribute_priority(":key"));
        assert!(attribute_priority(":key") < attribute_priority("v-model"));
        assert!(attribute_priority("v-model") < attribute_priority(":class"));
        // :class and class share the same priority so they stay adjacent
        assert_eq!(attribute_priority(":class"), attribute_priority("class"));
        assert_eq!(attribute_priority(":style"), attribute_priority("style"));
        assert!(attribute_priority("class") < attribute_priority("@click"));
        assert!(attribute_priority("@click") < attribute_priority("#default"));
        assert!(attribute_priority("#default") < attribute_priority("v-html"));
    }

    // ---------------------------------------------------------------
    // Tests for new configuration options
    // ---------------------------------------------------------------

    #[test]
    fn test_sort_attributes_disabled() {
        // When sort_attributes is false, keep original order
        let source =
            r#"<div @click="handle" :class="cls" v-if="show" v-for="item in items"></div>"#;
        let mut options = FormatOptions::default();
        options.sort_attributes = false;
        let result = format_template_content(source, &options).unwrap();

        let click_pos = result.find("@click").unwrap();
        let class_pos = result.find(":class").unwrap();
        let vif_pos = result.find("v-if").unwrap();
        let vfor_pos = result.find("v-for").unwrap();

        // Original order preserved: @click, :class, v-if, v-for
        assert!(click_pos < class_pos);
        assert!(class_pos < vif_pos);
        assert!(vif_pos < vfor_pos);
    }

    #[test]
    fn test_alphabetical_sort_within_group() {
        // Props (same priority group 8) should be sorted alphabetically
        let source = r#"<div title="t" class="c" aria-label="a"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        let aria_pos = result.find("aria-label").unwrap();
        let class_pos = result.find("class").unwrap();
        let title_pos = result.find("title").unwrap();

        assert!(
            aria_pos < class_pos,
            "aria-label should come before class alphabetically"
        );
        assert!(
            class_pos < title_pos,
            "class should come before title alphabetically"
        );
    }

    #[test]
    fn test_as_written_sort_within_group() {
        // When attribute_sort_order is AsWritten, keep original order within group
        let source = r#"<div title="t" class="c" aria-label="a"></div>"#;
        let mut options = FormatOptions::default();
        options.attribute_sort_order = AttributeSortOrder::AsWritten;
        let result = format_template_content(source, &options).unwrap();

        let title_pos = result.find("title").unwrap();
        let class_pos = result.find("class").unwrap();
        let aria_pos = result.find("aria-label").unwrap();

        // Original order within group preserved: title, class, aria-label
        assert!(title_pos < class_pos);
        assert!(class_pos < aria_pos);
    }

    #[test]
    fn test_merge_bind_and_non_bind_false() {
        // Default: non-bind attrs first, then bind attrs, each sorted alphabetically
        let source = r#"<div :class="cls" class="base" :style="s" style="color:red"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        let class_pos = result.find("class=").unwrap();
        let style_pos = result.find("style=").unwrap();
        let bind_class_pos = result.find(":class=").unwrap();
        let bind_style_pos = result.find(":style=").unwrap();

        // Non-bind first: class, style, :class, :style
        assert!(
            class_pos < bind_class_pos,
            "class should come before :class"
        );
        assert!(
            style_pos < bind_style_pos,
            "style should come before :style"
        );
    }

    #[test]
    fn test_merge_bind_and_non_bind_true() {
        // When merge_bind_and_non_bind_attrs is true, bind and non-bind are merged
        let source = r#"<div :class="cls" class="base" :style="s" style="color:red"></div>"#;
        let mut options = FormatOptions::default();
        options.merge_bind_and_non_bind_attrs = true;
        let result = format_template_content(source, &options).unwrap();

        // With merging: class and :class sort together by base name "class",
        // style and :style sort together by base name "style"
        // Expected order: class, :class, style, :style (or :class, class, :style, style)
        // Since both have the same base, stable sort by original_index applies
        let class_pos = result.find("class=").unwrap();
        let bind_class_pos = result.find(":class=").unwrap();
        let style_pos = result.find("style=").unwrap();
        let bind_style_pos = result.find(":style=").unwrap();

        // "class" and ":class" should be adjacent, "style" and ":style" should be adjacent
        // All "class*" come before "style*" alphabetically
        assert!(
            class_pos.min(bind_class_pos) < style_pos.min(bind_style_pos),
            "class group should come before style group"
        );
    }

    #[test]
    fn test_max_attributes_per_line() {
        let source = r#"<div class="c" id="main" title="t" aria-label="a" role="button"></div>"#;
        let mut options = FormatOptions::default();
        options.max_attributes_per_line = Some(2);
        let result = format_template_content(source, &options).unwrap();

        // Should wrap: multiple lines with at most 2 attrs per line
        let lines: Vec<&str> = result.lines().collect();
        assert!(
            lines.len() >= 3,
            "Should have at least 3 lines with max 2 attrs per line for 5 attrs"
        );
    }

    #[test]
    fn test_max_attributes_per_line_no_wrap_if_within() {
        let source = r#"<div class="c" id="main"></div>"#;
        let mut options = FormatOptions::default();
        options.max_attributes_per_line = Some(3);
        let result = format_template_content(source, &options).unwrap();

        // 2 attrs <= 3 limit, no wrapping
        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 2, "Should not wrap (div + closing)");
    }

    #[test]
    fn test_custom_attribute_groups() {
        // Custom groups: [["id"], ["class", ":class"], ["@*"], ["*"]]
        let source = r#"<div @click="h" class="c" id="main" title="t"></div>"#;
        let mut options = FormatOptions::default();
        options.attribute_groups = Some(vec![
            vec!["id".to_compact_string()],
            vec!["class".to_compact_string(), ":class".to_compact_string()],
            vec!["@*".to_compact_string()],
            vec!["*".to_compact_string()],
        ]);
        let result = format_template_content(source, &options).unwrap();

        let id_pos = result.find("id=").unwrap();
        let class_pos = result.find("class=").unwrap();
        let click_pos = result.find("@click=").unwrap();
        let title_pos = result.find("title=").unwrap();

        assert!(id_pos < class_pos, "id should come first (group 0)");
        assert!(
            class_pos < click_pos,
            "class (group 1) before @click (group 2)"
        );
        assert!(
            click_pos < title_pos,
            "@click (group 2) before title (group 3)"
        );
    }

    #[test]
    fn test_normalize_directive_shorthands_disabled() {
        let source = r#"<div v-bind:class="active" v-on:click="handler"></div>"#;
        let mut options = FormatOptions::default();
        options.normalize_directive_shorthands = false;
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_dynamic_directive_shorthand_snapshot() {
        let source = r#"<button v-bind:[name]="value" v-on:[event]="handler">{{label}}</button>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        insta::assert_snapshot!(result.as_str());
    }

    #[test]
    fn test_custom_attribute_priority() {
        let groups = vec![
            vec!["v-for".to_compact_string()],
            vec!["v-if".to_compact_string(), "v-else".to_compact_string()],
            vec![":*".to_compact_string()],
            vec!["@*".to_compact_string()],
        ];

        assert_eq!(custom_attribute_priority("v-for", &groups), 0);
        assert_eq!(custom_attribute_priority("v-if", &groups), 1);
        assert_eq!(custom_attribute_priority("v-else", &groups), 1);
        assert_eq!(custom_attribute_priority(":class", &groups), 2);
        assert_eq!(custom_attribute_priority(":style", &groups), 2);
        assert_eq!(custom_attribute_priority("@click", &groups), 3);
        // Unmatched gets groups.len()
        assert_eq!(custom_attribute_priority("id", &groups), 4);
    }

    #[test]
    fn test_matches_attr_pattern() {
        assert!(matches_attr_pattern("class", "class"));
        assert!(!matches_attr_pattern("class", "id"));
        assert!(matches_attr_pattern(":class", ":*"));
        assert!(matches_attr_pattern(":style", ":*"));
        assert!(matches_attr_pattern("@click", "@*"));
        assert!(matches_attr_pattern("v-for", "v-*"));
        assert!(matches_attr_pattern("anything", "*"));
    }

    #[test]
    fn test_print_width_triggers_multiline() {
        // Very narrow print_width should trigger multiline
        let source = r#"<div class="container" id="main" title="tooltip"></div>"#;
        let mut options = FormatOptions::default();
        options.print_width = 30;
        let result = format_template_content(source, &options).unwrap();

        let lines: Vec<&str> = result.lines().collect();
        assert!(
            lines.len() > 2,
            "Narrow print_width should trigger multiline attributes"
        );
    }
}
