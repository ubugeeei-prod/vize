use vize_l0::String;

pub(super) fn identifier(text: &str) -> bool {
    let mut chars = text.bytes();
    chars
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == b'_')
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-'))
}

pub(super) fn variable_key(name: &str) -> String {
    name.bytes()
        .map(|c| if c == b'_' { '-' } else { char::from(c) })
        .collect()
}

pub(super) fn literal(word: &str) -> bool {
    if let Some(hex) = word.strip_prefix('#') {
        return matches!(hex.len(), 3 | 6) && hex.bytes().all(|c| c.is_ascii_hexdigit());
    }
    let digits = word.bytes().take_while(u8::is_ascii_digit).count();
    if digits != 0 {
        let (n, unit) = word.split_at(digits);
        return n.len() <= 9
            && (n == "0" || !n.starts_with('0'))
            && matches!(
                unit,
                "" | "px" | "em" | "rem" | "%" | "s" | "ms" | "vh" | "vw" | "fr" | "deg"
            );
    }
    identifier(word)
        && !["null", "and", "or", "not", "true", "false"]
            .iter()
            .any(|keyword| word.eq_ignore_ascii_case(keyword))
}

fn compound(selector: &str) -> bool {
    let Some((first, rest)) = selector.split_once('.') else {
        return identifier(selector);
    };
    (first.is_empty() || identifier(first)) && rest.split('.').all(identifier)
}

pub(super) fn selector(text: &str, parent: Option<&str>) -> Option<String> {
    let mut words = text.split_ascii_whitespace();
    let first = words.next()?;
    let mut output = String::default();
    if let Some(suffix) = first.strip_prefix('&') {
        let parent = parent?;
        if !suffix.is_empty()
            && !(suffix.starts_with('.') && compound(suffix))
            && !((suffix.starts_with('_') || suffix.starts_with('-'))
                && suffix
                    .bytes()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, b'_' | b'-')))
        {
            return None;
        }
        output.push_str(parent);
        output.push_str(suffix);
    } else {
        if !compound(first) {
            return None;
        }
        if let Some(parent) = parent {
            output.push_str(parent);
            output.push(' ');
        }
        output.push_str(first);
    }
    for word in words {
        if !compound(word) {
            return None;
        }
        output.push(' ');
        output.push_str(word);
    }
    Some(output)
}
