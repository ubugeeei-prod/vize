use super::{
    component_name_candidates, is_component_tag, kebab_to_pascal, offset_to_position,
    pascal_to_kebab, position_to_offset, standalone_html_block_at_offset, token_at_offset,
    token_span_at_offset,
};
use crate::virtual_code::BlockType;

#[cfg(unix)]
pub(crate) fn symlink_dir(source: &std::path::Path, target: &std::path::Path) {
    std::fs::create_dir_all(target.parent().expect("symlink parent"))
        .expect("symlink parent directory");
    std::os::unix::fs::symlink(source, target).expect("directory symlink");
}

#[test]
fn test_offset_to_position() {
    let content = "line1\nline2\nline3";

    assert_eq!(offset_to_position(content, 0), (0, 0));
    assert_eq!(offset_to_position(content, 5), (0, 5));
    assert_eq!(offset_to_position(content, 6), (1, 0));
    assert_eq!(offset_to_position(content, 8), (1, 2));
    assert_eq!(offset_to_position(content, 12), (2, 0));
}

#[test]
fn test_offset_to_position_counts_utf16_code_units() {
    let content = "const icon = \"😀\";\nconst message = icon";

    assert_eq!(
        offset_to_position(content, "const icon = \"😀".len()),
        (0, 16)
    );
    assert_eq!(
        offset_to_position(content, content.find("message").unwrap()),
        (1, 6)
    );
}

#[test]
fn test_position_to_offset() {
    let content = "line1\nline2\nline3";

    assert_eq!(position_to_offset(content, 0, 0), Some(0));
    assert_eq!(position_to_offset(content, 0, 5), Some(5));
    assert_eq!(position_to_offset(content, 1, 0), Some(6));
    assert_eq!(position_to_offset(content, 1, 2), Some(8));
    assert_eq!(position_to_offset(content, 2, 0), Some(12));
}

#[test]
fn test_position_to_offset_counts_utf16_code_units() {
    let content = "a😀b\nc";

    assert_eq!(position_to_offset(content, 0, 3), Some("a😀".len()));
    assert_eq!(position_to_offset(content, 0, 4), Some("a😀b".len()));
    assert_eq!(position_to_offset(content, 1, 1), Some(content.len()));
}

#[test]
fn test_position_to_offset_rejects_utf16_surrogate_pair_interior() {
    let content = "a😀b";

    assert_eq!(position_to_offset(content, 0, 2), None);
}

#[test]
fn test_kebab_to_pascal() {
    assert_eq!(kebab_to_pascal("my-component"), "MyComponent");
    assert_eq!(kebab_to_pascal("button"), "Button");
    assert_eq!(kebab_to_pascal("v-for-item"), "VForItem");
    assert_eq!(kebab_to_pascal("a-b-c"), "ABC");
}

#[test]
fn test_pascal_to_kebab() {
    assert_eq!(pascal_to_kebab("MyComponent"), "my-component");
    assert_eq!(pascal_to_kebab("Button"), "button");
    assert_eq!(pascal_to_kebab("VForItem"), "v-for-item");
    assert_eq!(pascal_to_kebab("ABC"), "a-b-c");
}

#[test]
fn test_component_name_candidates_preserve_existing_order() {
    assert_eq!(
        component_name_candidates("description-item"),
        vec!["description-item", "DescriptionItem", "descriptionItem"]
    );
    assert_eq!(
        component_name_candidates("DescriptionItem"),
        vec!["DescriptionItem", "descriptionItem"]
    );
    assert_eq!(
        component_name_candidates("descriptionItem"),
        vec!["descriptionItem"]
    );
}

#[test]
fn test_is_component_tag() {
    assert!(is_component_tag("MyComponent"));
    assert!(is_component_tag("Button"));

    assert!(is_component_tag("my-component"));
    assert!(is_component_tag("v-button"));

    assert!(!is_component_tag("div"));
    assert!(!is_component_tag("span"));
    assert!(!is_component_tag("button"));
    assert!(!is_component_tag("color-profile"));
}

#[test]
fn test_token_span_at_offset_allows_identifier_boundaries() {
    let content = "const message = ref(0)";

    assert_eq!(
        token_span_at_offset(content, 5, |c| c.is_ascii_alphanumeric() || c == b'_'),
        Some((0, 5))
    );
    assert_eq!(
        token_span_at_offset(content, 13, |c| c.is_ascii_alphanumeric() || c == b'_'),
        Some((6, 13))
    );
    assert_eq!(
        token_span_at_offset(content, 15, |c| c.is_ascii_alphanumeric() || c == b'_'),
        None
    );
}

#[test]
fn test_token_at_offset_supports_end_of_file_boundaries() {
    let content = "message";

    assert_eq!(
        token_at_offset(content, content.len(), |c| c.is_ascii_alphanumeric()
            || c == b'_'),
        Some("message".to_string())
    );
}

#[test]
fn standalone_html_block_detection_finds_template_script_and_style_regions() {
    let content = r#"<div>{{ message }}</div>
<script type="module">
const message = "hello"
</script>
<style>
.active { color: red; }
</style>
"#;

    assert_eq!(
        standalone_html_block_at_offset(content, content.find("message").unwrap()),
        BlockType::Template
    );
    assert_eq!(
        standalone_html_block_at_offset(content, content.find("const message").unwrap()),
        BlockType::Script
    );
    assert_eq!(
        standalone_html_block_at_offset(content, content.find(".active").unwrap()),
        BlockType::Style(0)
    );
}

#[test]
fn standalone_html_block_detection_ignores_raw_tags_inside_comments() {
    let content = r#"<!-- <script type="module">const stale = true</script> -->
<!-- <style>.stale { color: red; }</style> -->
<div>{{ message }}</div>
"#;

    assert_eq!(
        standalone_html_block_at_offset(content, content.find("message").unwrap()),
        BlockType::Template
    );
}

#[test]
fn standalone_html_block_detection_resumes_after_comment_closes() {
    let content = r#"<!-- <script>const ignored = true</script> -->
<script>
const active = true
</script>
<div>{{ active }}</div>
"#;

    assert_eq!(
        standalone_html_block_at_offset(content, content.find("const active").unwrap()),
        BlockType::Script
    );
    assert_eq!(
        standalone_html_block_at_offset(content, content.rfind("active").unwrap()),
        BlockType::Template
    );
}
