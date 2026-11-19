//! Authored `v-slot` parameter decomposition.

use super::{SlotParameter, SlotParameterPart};

fn trimmed_part(source: &str, start: usize, end: usize) -> SlotParameterPart<'_> {
    let part = source.get(start..end).unwrap_or_default();
    let leading = part.len() - part.trim_start().len();
    let text = part.trim();
    SlotParameterPart {
        text,
        start: start + leading,
    }
}

fn is_parameter_default_equals(source: &str, at: usize) -> bool {
    let before = source.get(..at).and_then(|text| text.chars().next_back());
    let after = source.get(at + 1..).and_then(|text| text.chars().next());
    after != Some('>')
        && after != Some('=')
        && !matches!(
            before,
            Some('=' | '!' | '<' | '>' | '+' | '-' | '*' | '/' | '%' | '&' | '|' | '^' | '?')
        )
}

fn can_start_regex(source: &str, at: usize) -> bool {
    let prefix = source.get(..at).unwrap_or_default().trim_end();
    let Some(last) = prefix.chars().next_back() else {
        return true;
    };
    if prefix.ends_with("++") || prefix.ends_with("--") {
        return false;
    }
    if matches!(
        last,
        '(' | '['
            | '{'
            | '='
            | ','
            | ':'
            | ';'
            | '!'
            | '?'
            | '&'
            | '|'
            | '+'
            | '-'
            | '*'
            | '%'
            | '^'
            | '~'
            | '<'
            | '>'
    ) {
        return true;
    }
    let word_start = prefix
        .char_indices()
        .rev()
        .find(|(_, ch)| !ch.is_alphanumeric() && *ch != '_' && *ch != '$')
        .map_or(0, |(at, ch)| at + ch.len_utf8());
    matches!(
        prefix.get(word_start..).unwrap_or_default(),
        "await"
            | "case"
            | "delete"
            | "do"
            | "else"
            | "in"
            | "instanceof"
            | "new"
            | "of"
            | "return"
            | "throw"
            | "typeof"
            | "void"
            | "yield"
    )
}

/// Split a slot function parameter into its authored binding pattern, optional
/// annotation and optional whole-parameter default. The inferred payload type
/// belongs before that default: `({ editor }: Payload = {})`, not the invalid
/// `({ editor } = {}: Payload)`.
pub(in super::super) fn slot_parameter(pattern: &str) -> SlotParameter<'_> {
    let mut depth = 0i32;
    let mut type_angle_depth = 0i32;
    let mut quote: Option<char> = None;
    let mut regex = false;
    let mut regex_class = false;
    let mut escaped = false;
    let mut line_comment = false;
    let mut block_comment = false;
    let mut annotation_at = None;
    let mut default_at = None;
    let mut chars = pattern.char_indices().peekable();

    while let Some((at, ch)) = chars.next() {
        let next = chars.peek().map(|(_, next)| *next);
        if line_comment {
            if matches!(ch, '\n' | '\r') {
                line_comment = false;
            }
            continue;
        }
        if block_comment {
            if ch == '*' && next == Some('/') {
                block_comment = false;
                chars.next();
            }
            continue;
        }
        if regex {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == '[' && !regex_class {
                regex_class = true;
            } else if ch == ']' && regex_class {
                regex_class = false;
            } else if ch == '/' && !regex_class {
                regex = false;
            }
            continue;
        }
        if let Some(open) = quote {
            if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == open {
                quote = None;
            }
            continue;
        }
        if ch == '/' && next == Some('/') {
            line_comment = true;
            chars.next();
            continue;
        }
        if ch == '/' && next == Some('*') {
            block_comment = true;
            chars.next();
            continue;
        }
        if ch == '/' && can_start_regex(pattern, at) {
            regex = true;
            regex_class = false;
            escaped = false;
            continue;
        }
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '{' | '[' | '(' => depth += 1,
            '}' | ']' | ')' if depth > 0 => depth -= 1,
            '<' if annotation_at.is_some() && depth == 0 => type_angle_depth += 1,
            '>' if annotation_at.is_some()
                && depth == 0
                && type_angle_depth > 0
                && pattern.get(..at).and_then(|text| text.chars().next_back()) != Some('=') =>
            {
                type_angle_depth -= 1;
            }
            ':' if depth == 0 && type_angle_depth == 0 && annotation_at.is_none() => {
                annotation_at = Some(at);
            }
            '=' if depth == 0
                && type_angle_depth == 0
                && is_parameter_default_equals(pattern, at) =>
            {
                default_at = Some(at);
                break;
            }
            _ => {}
        }
    }

    let pattern_end = annotation_at.or(default_at).unwrap_or(pattern.len());
    let annotation =
        annotation_at.map(|at| trimmed_part(pattern, at + 1, default_at.unwrap_or(pattern.len())));
    let default = default_at.map(|at| trimmed_part(pattern, at + 1, pattern.len()));
    SlotParameter {
        pattern: trimmed_part(pattern, 0, pattern_end),
        annotation,
        default,
    }
}

/// Split a `v-slot` props expression that carries its own TypeScript
/// annotation (`#item="{ element }: { element: Tag }"`) into the binding
/// pattern and the annotation. Vue allows the annotation, so emitting the
/// inferred slot type after the full expression would produce `pattern: A: B`
/// — a syntax error that aborts Corsa's semantic pass for the whole project.
/// A `:` separates the annotation only at the top nesting level.
pub(in super::super) fn split_slot_pattern_annotation(pattern: &str) -> Option<(&str, &str)> {
    let parameter = slot_parameter(pattern);
    let annotation = parameter.annotation?.text;
    (!annotation.is_empty()).then_some((parameter.pattern.text, annotation))
}

#[cfg(test)]
mod tests {
    use super::slot_parameter;

    #[test]
    fn slot_parameters_separate_annotations_and_whole_parameter_defaults() {
        for (source, pattern, annotation, default) in [
            ("{ editor } = {}", "{ editor }", None, Some("{}")),
            (
                "{ editor }: { editor: string } = {}",
                "{ editor }",
                Some("{ editor: string }"),
                Some("{}"),
            ),
            (
                "slot: () => { editor: string } = fallback",
                "slot",
                Some("() => { editor: string }"),
                Some("fallback"),
            ),
            ("{ editor = fallback }", "{ editor = fallback }", None, None),
            (
                "{ make = () => 1, editor = fallback }",
                "{ make = () => 1, editor = fallback }",
                None,
                None,
            ),
            (
                r"{ matcher = /\}/, editor = fallback }",
                r"{ matcher = /\}/, editor = fallback }",
                None,
                None,
            ),
            (
                r"{ matcher = /[}]/, editor = fallback }",
                r"{ matcher = /[}]/, editor = fallback }",
                None,
                None,
            ),
            (
                "{ ratio = total / count, editor = fallback }",
                "{ ratio = total / count, editor = fallback }",
                None,
                None,
            ),
            (
                "{ ratio = count++ / divisor } = {}",
                "{ ratio = count++ / divisor }",
                None,
                Some("{}"),
            ),
            (
                "{ flag = count > 0, editor = fallback }",
                "{ flag = count > 0, editor = fallback }",
                None,
                None,
            ),
            (
                "{ flag = count < 1 } = {}",
                "{ flag = count < 1 }",
                None,
                Some("{}"),
            ),
            (
                "slot: Factory<T = string> = fallback",
                "slot",
                Some("Factory<T = string>"),
                Some("fallback"),
            ),
        ] {
            let parameter = slot_parameter(source);
            assert_eq!(parameter.pattern.text, pattern, "{source}");
            assert_eq!(
                parameter.annotation.map(|part| part.text),
                annotation,
                "{source}"
            );
            assert_eq!(parameter.default.map(|part| part.text), default, "{source}");
        }
    }
}
