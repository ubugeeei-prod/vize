//! Component `v-model` helpers shared by definition entrypoints.

pub(super) fn prop_name_from_v_model_attribute(raw_attr_name: &str) -> Option<String> {
    if raw_attr_name == "v-model" || raw_attr_name.starts_with("v-model.") {
        return Some("modelValue".to_string());
    }

    let argument = raw_attr_name.strip_prefix("v-model:")?;
    if argument.starts_with('[') {
        return None;
    }

    let name = argument.split_once('.').map_or(argument, |(name, _)| name);
    (!name.is_empty()).then(|| name.to_string())
}

/// Find the authored declaration anchor for a `defineModel` prop.
///
/// Argument-less `defineModel()` declares `modelValue`, whose prop key has no
/// authored token, so the call identifier is the most useful jump target.
/// Named models jump to the literal argument itself.
pub(super) fn find_prop_in_define_model(
    content: &str,
    property_name: &str,
) -> Option<(usize, usize)> {
    for (define_model_pos, _) in content.match_indices("defineModel") {
        if !is_identifier_boundary(content, define_model_pos, "defineModel".len()) {
            continue;
        }

        let after_name = &content[define_model_pos + "defineModel".len()..];
        let paren_relative = after_name.find('(')?;
        let args_start = define_model_pos + "defineModel".len() + paren_relative + 1;
        let mut arg_start = args_start;
        while arg_start < content.len() && content.as_bytes()[arg_start].is_ascii_whitespace() {
            arg_start += 1;
        }

        match content.as_bytes().get(arg_start).copied() {
            Some(b'\'' | b'"' | b'`') => {
                let quote = content.as_bytes()[arg_start];
                let literal_start = arg_start + 1;
                let literal_end = find_string_literal_end(content, literal_start, quote)?;
                if content.get(literal_start..literal_end) == Some(property_name) {
                    return Some((literal_start, property_name.len()));
                }
            }
            _ if property_name == "modelValue" => {
                return Some((define_model_pos, "defineModel".len()));
            }
            _ => {}
        }
    }

    None
}

fn is_identifier_boundary(content: &str, start: usize, len: usize) -> bool {
    let bytes = content.as_bytes();
    let end = start + len;
    let before = start.checked_sub(1).and_then(|index| bytes.get(index));
    let after = bytes.get(end);
    !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'$')
        && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'$')
}

fn find_string_literal_end(content: &str, start: usize, quote: u8) -> Option<usize> {
    let bytes = content.as_bytes();
    let mut pos = start;
    while pos < bytes.len() {
        match bytes[pos] {
            b'\\' => pos += 2,
            byte if byte == quote => return Some(pos),
            _ => pos += 1,
        }
    }
    None
}
