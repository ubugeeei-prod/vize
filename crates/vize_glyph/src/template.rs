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
pub(crate) mod helpers;

/// Native HTML elements whose authored text is whitespace-significant.
///
/// Keep this list shared with the SFC indentation mask: if the two formatter
/// layers disagree, each pass can add another indentation level. Vue treats
/// names containing uppercase ASCII as components, so callers must match
/// these lowercase names exactly.
pub(crate) const WHITESPACE_SIGNIFICANT_NATIVE_ELEMENTS: [&str; 3] = ["pre", "textarea", "listing"];

#[cfg(test)]
mod attribute_priority_tests;

use crate::{error::FormatError, options::FormatOptions};
use vize_s0::String;

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
    use super::{directives, format_template_content, formatter, helpers};
    use crate::options::{AttributeSortOrder, FormatOptions};
    use directives::{custom_attribute_priority, format_v_for_expression, matches_attr_pattern};
    use formatter::format_interpolations;
    use helpers::{is_tag_name_char, is_void_element_str};
    use vize_s0::ToCompactString;

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
    fn test_multiline_self_closing_slash_aligns_with_opening_tag() {
        let source = r#"<slot name="slide" :data="item" :index="index" :is-active="index === selectedIndex" :is-in-view="slidesInView.includes(index)" />"#;
        let mut options = FormatOptions::default();
        options.sort_attributes = false;
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<slot
  name="slide"
  :data="item"
  :index="index"
  :is-active="index === selectedIndex"
  :is-in-view="slidesInView.includes(index)"
/>"#
        );
    }

    #[test]
    fn test_empty_element_stays_inline() {
        let source = "<div>\n</div>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(result.as_str(), "<div></div>");
    }

    #[test]
    fn test_empty_element_with_multiline_attributes_closes_inline() {
        let source = r#"<div class="container" id="main"></div>"#;
        let mut options = FormatOptions::default();
        options.single_attribute_per_line = true;
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<div
  id="main"
  class="container"
></div>"#
        );
    }

    #[test]
    fn test_multiline_interpolation_keeps_expression_indented() {
        let source = r#"<span>
  {{
    preview
      ? [
          preview.document.collectionLabel,
          preview.document.publishedAt
            ? formatDate(preview.document.publishedAt)
            : "Frontmatter required",
          isPreviewPending ? "Previewing" : "Ready",
        ].join(" / ")
      : (activePath ?? "No file selected")
  }}
</span>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<span>
  {{
    preview
      ? [
          preview.document.collectionLabel,
          preview.document.publishedAt
            ? formatDate(preview.document.publishedAt)
            : "Frontmatter required",
          isPreviewPending ? "Previewing" : "Ready",
        ].join(" / ")
      : (activePath ?? "No file selected")
  }}
</span>"#
        );
    }

    #[test]
    fn directive_expression_double_quote_formatting_keeps_valid_attribute_quotes() {
        let source = r#"<MfCheckbox @blur="form.validateField('agreeToPolicy')" />"#;
        let mut options = FormatOptions::default();
        options.single_quote = false;

        let result = format_template_content(source, &options).unwrap();
        assert_eq!(
            result.as_str(),
            r#"<MfCheckbox @blur='form.validateField("agreeToPolicy")' />"#
        );

        let formatted_again = format_template_content(&result, &options).unwrap();
        assert_eq!(formatted_again, result);
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
    fn test_interpolation_less_than_comparison() {
        let source = "<span>{{ i < items.length - 1 ? ',' : '' }}</span>";
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<span>{{ i < items.length - 1 ? "," : "" }}</span>"#
        );
    }

    #[test]
    fn test_v_for_interpolation_less_than_comparison() {
        let source = r#"<span v-for="(item, index) in list" :key="item">{{ item }}{{ index < list.length - 1 ? ',' : '' }}</span>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<span v-for="(item, index) in list" :key="item">{{ item }}{{ index < list.length - 1 ? "," : "" }}</span>"#
        );
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

        assert!(vfor_pos < vif_pos);
        assert!(vif_pos < class_pos);
        assert!(class_pos < click_pos);
    }

    #[test]
    fn test_multiline_attributes() {
        let source = r#"<div class="container" id="main" @click="handler">Content</div>"#;
        let mut options = FormatOptions::default();
        options.single_attribute_per_line = true;
        let result = format_template_content(source, &options).unwrap();

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
        assert!(!is_void_element_str("Link"));
        assert!(!is_void_element_str("Input"));
        assert!(!is_void_element_str("Img"));
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
        let source = r#"<div :class="cls" class="base" :style="s" style="color:red"></div>"#;
        let options = FormatOptions::default();
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<div class="base" style="color:red" :class="cls" :style="s"></div>"#
        );
    }

    #[test]
    fn test_merge_bind_and_non_bind_true() {
        let source = r#"<div :class="cls" class="base" :style="s" style="color:red"></div>"#;
        let mut options = FormatOptions::default();
        options.merge_bind_and_non_bind_attrs = true;
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(result.as_str(), source);
    }

    #[test]
    fn test_max_attributes_per_line() {
        let source = r#"<div class="c" id="main" title="t" aria-label="a" role="button"></div>"#;
        let mut options = FormatOptions::default();
        options.max_attributes_per_line = Some(2);
        let result = format_template_content(source, &options).unwrap();

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

        let lines: Vec<&str> = result.lines().collect();
        assert_eq!(lines.len(), 1, "Should keep the empty element inline");
    }

    #[test]
    fn test_single_attribute_per_line_overrides_max_attributes_per_line() {
        let source = r#"<div class="c" id="main" title="t"></div>"#;
        let mut options = FormatOptions::default();
        options.single_attribute_per_line = true;
        options.max_attributes_per_line = Some(4);
        let result = format_template_content(source, &options).unwrap();

        assert_eq!(
            result.as_str(),
            r#"<div
  id="main"
  class="c"
  title="t"
></div>"#
        );
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

        assert!(id_pos < class_pos);
        assert!(class_pos < click_pos);
        assert!(click_pos < title_pos);
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
