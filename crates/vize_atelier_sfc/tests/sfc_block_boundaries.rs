use vize_atelier_sfc::parse_sfc;

#[test]
fn top_level_html_void_elements_are_not_custom_blocks() {
    let source = r#"<script src="https://example.com/component.js"></script>
<link rel="stylesheet" href="https://example.com/component.css">"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    assert!(descriptor.script.is_some());
    assert!(descriptor.custom_blocks.is_empty());
}

#[test]
fn return_string_with_comment_opener_does_not_hide_script_close() {
    let source = r#"<script setup>
function accept() {
  return "image/*"
}
</script>
<template><input :accept="accept()" /></template>"#;
    let descriptor = parse_sfc(source, Default::default()).unwrap();

    assert!(descriptor.script_setup.is_some());
    assert!(descriptor.template.is_some());
}
