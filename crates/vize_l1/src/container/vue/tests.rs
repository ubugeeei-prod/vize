use vize_l0::Allocator;

use super::Vue;
use crate::container::{ContainerErrorCode, ContainerFormat};

#[test]
fn splits_blocks_with_ordered_attributes() {
    let source = "<script setup lang=\"ts\">const a = '</script>'</script>\n<template><div/></template>\n<style scoped>.a{}</style>";
    let allocator = Allocator::default();
    let container = Vue.split(&allocator, source);
    assert!(container.errors.is_empty());
    let names: [&str; 3] = [
        container.blocks[0].name,
        container.blocks[1].name,
        container.blocks[2].name,
    ];
    assert_eq!(names, ["script", "template", "style"]);
    let script = &container.blocks[0];
    assert_eq!(script.attrs[0].name, "setup");
    assert_eq!(script.attr("lang").and_then(|attr| attr.value), Some("ts"));
    let content = script.content;
    assert_eq!(
        source.get(content.start as usize..content.end as usize),
        Some("const a = '</script>'")
    );
    assert!(script.close_tag.is_some());
}

#[test]
fn reports_duplicates_and_missing_close_tags() {
    let allocator = Allocator::default();
    let container = Vue.split(
        &allocator,
        "<template></template><template></template><style>",
    );
    let codes: [ContainerErrorCode; 2] = [container.errors[0].code, container.errors[1].code];
    assert_eq!(
        codes,
        [
            ContainerErrorCode::DuplicateBlock,
            ContainerErrorCode::MissingCloseTag
        ]
    );
}
