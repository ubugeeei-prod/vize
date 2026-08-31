use std::path::Path;

use vize_s0::String as CompactString;

use crate::virtual_ts::to_safe_identifier;

pub(super) fn content_mapper_component_name(path: &Path) -> CompactString {
    let stem = path
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or("VueComponent");
    let pascal = content_mapper_pascal_case(stem);
    let name = to_safe_identifier(pascal.as_str());
    if name.as_str().bytes().all(|byte| byte == b'_') {
        CompactString::from("VueComponent")
    } else {
        name
    }
}

fn content_mapper_pascal_case(value: &str) -> CompactString {
    let mut result = CompactString::with_capacity(value.len());
    let mut capitalize_next = true;

    for character in value.chars() {
        if character == '-' || character == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(character.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(character);
        }
    }

    result
}
