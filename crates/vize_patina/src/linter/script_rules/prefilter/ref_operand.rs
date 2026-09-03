const REF_FACTORY_NAMES: &[&str] = &["ref", "computed", "shallowRef", "toRef", "customRef"];

pub(super) fn source_may_contain_ref_operand(source: &str) -> bool {
    if !REF_FACTORY_NAMES
        .iter()
        .any(|factory| source.contains(factory))
    {
        return false;
    }
    if !source.is_ascii() {
        return true;
    }

    let bindings = collect_direct_ref_bindings(source);
    if bindings.is_empty() {
        return true;
    }

    bindings
        .into_iter()
        .any(|binding| identifier_may_be_ref_operand(source, binding))
}

fn collect_direct_ref_bindings(source: &str) -> Vec<&str> {
    let mut names = Vec::new();
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'=' || is_equality_or_arrow(bytes, index) {
            continue;
        }
        let rest = source[index + 1..].trim_start();
        if !starts_with_ref_factory_call(rest) {
            continue;
        }
        if let Some(name) = identifier_before_assignment(source, index) {
            names.push(name);
        }
    }
    names
}

fn is_equality_or_arrow(bytes: &[u8], index: usize) -> bool {
    matches!(
        bytes.get(index.wrapping_sub(1)),
        Some(b'!' | b'=' | b'<' | b'>')
    ) || matches!(bytes.get(index + 1), Some(b'=' | b'>'))
}

fn starts_with_ref_factory_call(source: &str) -> bool {
    REF_FACTORY_NAMES.iter().any(|factory| {
        source
            .strip_prefix(factory)
            .and_then(|rest| rest.trim_start().as_bytes().first().copied())
            .is_some_and(|byte| byte == b'(' || byte == b'<')
    })
}

fn identifier_before_assignment(source: &str, equals_index: usize) -> Option<&str> {
    let bytes = source.as_bytes();
    let mut end = equals_index;
    while end > 0 && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    let mut start = end;
    while start > 0 && is_ascii_identifier_continue(bytes[start - 1]) {
        start -= 1;
    }
    (start < end).then(|| &source[start..end])
}

fn identifier_may_be_ref_operand(source: &str, name: &str) -> bool {
    let mut search_start = 0;
    while let Some(relative) = source[search_start..].find(name) {
        let start = search_start + relative;
        let end = start + name.len();
        search_start = end;
        if !has_identifier_boundaries(source.as_bytes(), start, end) {
            continue;
        }
        if occurrence_is_ref_initializer(source, start, end)
            || next_non_ws_is_value(source, end)
            || next_non_ws_is_object_key(source, end)
        {
            continue;
        }
        if occurrence_is_obviously_passed_ref(source, start, end) {
            continue;
        }
        return true;
    }
    false
}

fn has_identifier_boundaries(bytes: &[u8], start: usize, end: usize) -> bool {
    !bytes
        .get(start.wrapping_sub(1))
        .is_some_and(|byte| is_ascii_identifier_continue(*byte))
        && !bytes
            .get(end)
            .is_some_and(|byte| is_ascii_identifier_continue(*byte))
}

fn occurrence_is_ref_initializer(source: &str, start: usize, end: usize) -> bool {
    let before = source[..start].trim_end();
    let after = source[end..].trim_start();
    after.starts_with('=')
        && (before.ends_with("const") || before.ends_with("let") || before.ends_with("var"))
}

fn next_non_ws_is_value(source: &str, end: usize) -> bool {
    source[end..].trim_start().starts_with(".value")
}

fn next_non_ws_is_object_key(source: &str, end: usize) -> bool {
    source[end..].trim_start().starts_with(':')
}

fn occurrence_is_obviously_passed_ref(source: &str, start: usize, end: usize) -> bool {
    let before = source[..start].trim_end();
    let after = source[end..].trim_start();
    if before.ends_with('(') && !paren_starts_plain_call(before) {
        return false;
    }
    matches!(after.as_bytes().first(), Some(b')' | b','))
        && !matches!(before.as_bytes().last(), Some(b'!' | b'+' | b'-' | b'~'))
}

fn paren_starts_plain_call(before: &str) -> bool {
    let without_paren = before.trim_end_matches('(').trim_end();
    let Some((start, _)) = without_paren
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_ascii_alphanumeric() && *ch != '_' && *ch != '$')
    else {
        return !is_control_keyword(without_paren);
    };
    let callee = without_paren[start + 1..].trim();
    !callee.is_empty() && !is_control_keyword(callee)
}

fn is_control_keyword(value: &str) -> bool {
    matches!(value, "if" | "while" | "for" | "switch")
}

fn is_ascii_identifier_continue(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_' || byte == b'$'
}

#[cfg(test)]
mod tests {
    use super::source_may_contain_ref_operand;

    #[test]
    fn skips_value_only_refs() {
        let source = "const count = ref(0)\ncount.value++\nwatch(count, () => {})";
        assert!(!source_may_contain_ref_operand(source));
    }

    #[test]
    fn keeps_operand_shapes() {
        for source in [
            "let count = ref(0)\ncount++",
            "let count = ref(0)\n--count",
            "const count = ref(0)\ncount + 1",
            "const count = ref(0)\nconst x = 1 + count",
            "const flag = ref(false)\nif (flag) {}",
            "const flag = ref(false)\nwhile (flag) {}",
            "const flag = ref(false)\nconst x = flag ? 1 : 2",
        ] {
            assert!(source_may_contain_ref_operand(source), "{source}");
        }
    }

    #[test]
    fn keeps_unicode_identifier_shapes_conservative() {
        assert!(source_may_contain_ref_operand("const 値 = ref(0)\n値++"));
    }

    #[test]
    fn keeps_template_literal_interpolation_operands() {
        assert!(source_may_contain_ref_operand(
            "const count = ref(0)\nconst text = `${count}`"
        ));
    }
}
