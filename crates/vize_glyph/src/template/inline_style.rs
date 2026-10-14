//! Lay out a static `style` attribute the way Prettier does on one line:
//! `name: value` declarations joined by `; `, without a trailing `;`.
//!
//! Only plain declaration lists are touched. A value with a newline, braces,
//! a comment, an entity or an escape stays as written, and so does any
//! declaration that is not `name: value`.

use vize_l0::String;

pub(super) fn normalize(value: &str) -> Option<String> {
    if value.contains(['\n', '\r', '{', '}', '&', '\\']) || value.contains("/*") {
        return None;
    }
    let mut output = String::with_capacity(value.len());
    for declaration in split_declarations(value)? {
        let declaration = declaration.trim();
        if declaration.is_empty() {
            continue;
        }
        let (name, rest) = declaration.split_once(':')?;
        let (name, rest) = (name.trim(), rest.trim());
        let plain_name = !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-' || byte == b'_');
        if !plain_name || rest.is_empty() {
            return None;
        }
        if !output.is_empty() {
            output.push_str("; ");
        }
        if name.starts_with("--") {
            output.push_str(name);
        } else {
            name.chars()
                .for_each(|ch| output.push(ch.to_ascii_lowercase()));
        }
        output.push_str(": ");
        output.push_str(rest);
    }
    (!output.is_empty() && output.as_str() != value).then_some(output)
}

/// Split on `;` outside strings and parentheses; `None` when unbalanced.
fn split_declarations(value: &str) -> Option<Vec<&str>> {
    let mut parts = Vec::new();
    let mut start = 0;
    let mut depth = 0usize;
    let mut quote = None;
    for (index, byte) in value.bytes().enumerate() {
        match (quote, byte) {
            (Some(open), _) if byte == open => quote = None,
            (Some(_), _) => {}
            (None, b'"' | b'\'') => quote = Some(byte),
            (None, b'(') => depth += 1,
            (None, b')') => depth = depth.checked_sub(1)?,
            (None, b';') if depth == 0 => {
                parts.push(value.get(start..index)?);
                start = index + 1;
            }
            _ => {}
        }
    }
    if quote.is_some() || depth != 0 {
        return None;
    }
    parts.push(value.get(start..)?);
    Some(parts)
}

#[cfg(test)]
mod tests {
    use super::normalize;

    #[test]
    fn joins_declarations_without_a_trailing_semicolon() {
        let cases = [
            ("color:red;top:0", "color: red; top: 0"),
            ("color: red;", "color: red"),
            (
                "  color : red ;  ; top:0 !important ;",
                "color: red; top: 0 !important",
            ),
            (
                "background: url('a;b.png'); COLOR: Red",
                "background: url('a;b.png'); color: Red",
            ),
            ("--Main-W: 700px;", "--Main-W: 700px"),
        ];
        for (input, expected) in cases {
            assert_eq!(normalize(input).as_deref(), Some(expected), "{input}");
            assert_eq!(normalize(expected), None, "{expected}");
        }
    }

    #[test]
    fn leaves_anything_but_plain_declarations() {
        for input in [
            "",
            "color: red",
            "--a: { x }; color: red",
            "color: red; /* c */ top: 0",
            "color: {{ x }}",
            "color: red;\n  top: 0",
            "background: url(&quot;a.png&quot;)",
            "red",
            "color: ",
            "content: 'a",
        ] {
            assert_eq!(normalize(input), None, "{input}");
        }
    }
}
