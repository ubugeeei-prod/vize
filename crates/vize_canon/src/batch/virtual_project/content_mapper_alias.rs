use vize_s0::String as CompactString;

pub(super) fn is_synthetic_content_mapper_identifier(text: &str) -> bool {
    text.starts_with("__vize_prop_check_")
}

pub(super) fn is_alias_projection(generated: &str, original: &str) -> bool {
    let generated_name = unquoted_ts_string(generated);
    let original_name = unquoted_ts_string(original);
    if !is_vue_name_projection(generated_name) || !is_vue_name_projection(original_name) {
        return false;
    }
    if generated_name == original_name {
        return generated != original;
    }
    if original_name == "v-model" && generated_name == "modelValue" {
        return true;
    }
    if original_name.contains('-')
        && !original_name.ends_with('-')
        && generated_name == vue_template_camelize(original_name)
    {
        return true;
    }
    generated_name.strip_prefix("update:") == Some(original_name)
}

fn unquoted_ts_string(text: &str) -> &str {
    let bytes = text.as_bytes();
    if bytes.len() >= 2
        && matches!(bytes[0], b'\'' | b'"' | b'`')
        && bytes.last() == Some(&bytes[0])
    {
        &text[1..text.len() - 1]
    } else {
        text
    }
}

fn is_vue_name_projection(text: &str) -> bool {
    !text.is_empty()
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'$' | b'-' | b':'))
}

fn vue_template_camelize(text: &str) -> CompactString {
    let mut result = CompactString::with_capacity(text.len());
    let mut capitalize_next = false;
    for character in text.chars() {
        if character == '-' {
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
