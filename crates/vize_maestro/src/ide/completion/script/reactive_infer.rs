//! Heuristic inference of reactive binding kinds and their inner value types
//! from raw script source, used to enrich completion details without a backing
//! Corsa type session.
#![allow(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros
)]

use vize_croquis::reactivity::ReactiveKind;
use vize_croquis::{Drawer, DrawerOptions};

pub(super) fn reactive_completion_info(
    script_content: &str,
    name: &str,
    kind: ReactiveKind,
) -> Option<(String, String)> {
    let wrapper = reactive_wrapper_type(kind)?;
    let value_type = infer_reactive_value_type(script_content, name, kind)
        .unwrap_or_else(|| "unknown".to_string());
    let detail = format!("{wrapper}<{value_type}>");
    let access = if kind.needs_value_access() {
        "Access with `.value` in script."
    } else {
        "Direct access in script."
    };
    let doc = format!("```typescript\n{name}: {detail}\n```\n\n{access}");
    Some((detail, doc))
}

fn reactive_wrapper_type(kind: ReactiveKind) -> Option<&'static str> {
    match kind {
        ReactiveKind::Computed => Some("ComputedRef"),
        ReactiveKind::Ref | ReactiveKind::ShallowRef | ReactiveKind::ToRef => Some("Ref"),
        _ => None,
    }
}

pub(super) fn reactive_kind_for_name(script_content: &str, name: &str) -> Option<ReactiveKind> {
    let mut analyzer = Drawer::with_options(DrawerOptions {
        analyze_script: true,
        ..Default::default()
    });
    analyzer.analyze_script_setup(script_content);
    let croquis = analyzer.finish();

    if let Some(source) = croquis.reactivity.lookup(name) {
        return Some(source.kind);
    }

    infer_reactive_kind_from_source(script_content, name)
}

fn infer_reactive_kind_from_source(script_content: &str, name: &str) -> Option<ReactiveKind> {
    let declaration_starts = [
        format!("const {name} = "),
        format!("let {name} = "),
        format!("var {name} = "),
    ];

    for declaration_start in declaration_starts {
        let Some(start) = script_content.find(declaration_start.as_str()) else {
            continue;
        };
        let initializer = script_content[start + declaration_start.len()..].trim_start();
        let callee = initializer
            .split_once('(')
            .map(|(callee, _)| callee.trim())
            .unwrap_or(initializer);

        if let Some(kind) = ReactiveKind::from_name(callee) {
            return Some(kind);
        }
    }

    None
}

/// Inline-source heuristic for the inner type of a reactive binding.
/// Returns `Some("number")` for `const n = ref(0)`, `Some("string")` for
/// `const s = ref<string>()`, etc. Exposed to crate so inlay-hint code can
/// reuse the same inference rather than duplicating it.
pub(crate) fn infer_reactive_value_type(
    script_content: &str,
    name: &str,
    kind: ReactiveKind,
) -> Option<String> {
    let wrapper = match kind {
        ReactiveKind::Computed => "ComputedRef",
        ReactiveKind::Ref | ReactiveKind::ShallowRef | ReactiveKind::ToRef => "Ref",
        _ => return None,
    };

    // The declaration is `const NAME = CALLEE<...>` or `const NAME = CALLEE(...)`
    // (likewise for `let`). Rather than run four separate `format!` + full-string
    // `find` scans (the `<` and `(` variants for each keyword), search the shared
    // `KEYWORD NAME = CALLEE` prefix once per keyword and branch on the byte that
    // immediately follows. A binding is declared once, so at most one keyword
    // matches; behavior is identical to the four-pattern form.
    let callee = reactive_kind_callee(kind);
    for keyword in ["const", "let"] {
        let prefix = format!("{keyword} {name} = {callee}");
        let Some(pos) = script_content.find(prefix.as_str()) else {
            continue;
        };
        let after = &script_content[pos + prefix.len()..];
        match after.as_bytes().first() {
            Some(b'<') => {
                if let Some(end) = find_matching_angle(&after[1..]) {
                    return Some(after[1..end + 1].trim().to_string());
                }
            }
            Some(b'(') => {
                return infer_value_type_from_initializer(&after[1..], wrapper);
            }
            _ => {}
        }
    }

    None
}

fn reactive_kind_callee(kind: ReactiveKind) -> &'static str {
    match kind {
        ReactiveKind::Computed => "computed",
        ReactiveKind::ShallowRef => "shallowRef",
        ReactiveKind::ToRef => "toRef",
        _ => "ref",
    }
}

fn find_matching_angle(s: &str) -> Option<usize> {
    let mut depth = 1;
    for (i, c) in s.chars().enumerate() {
        match c {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
    }
    None
}

fn infer_value_type_from_initializer(initializer: &str, wrapper: &str) -> Option<String> {
    let initializer = initializer.trim_start();
    if wrapper == "ComputedRef"
        && let Some(body) = extract_arrow_body(initializer)
    {
        return infer_expression_type(body);
    }

    infer_expression_type(initializer)
}

fn extract_arrow_body(initializer: &str) -> Option<&str> {
    let arrow = initializer.find("=>")?;
    let body = initializer[arrow + 2..].trim_start();

    if let Some(body) = body.strip_prefix('{')
        && let Some(return_pos) = body.find("return")
    {
        let returned = body[return_pos + "return".len()..].trim_start();
        let end = returned.find([';', '}']).unwrap_or(returned.len());
        return Some(returned[..end].trim());
    }

    let end = body.find(['\n', ';']).unwrap_or(body.len());
    Some(body[..end].trim().trim_end_matches(')').trim())
}

fn infer_expression_type(expression: &str) -> Option<String> {
    let expression = expression.trim();

    if expression.starts_with('"') || expression.starts_with('\'') || expression.starts_with('`') {
        return Some("string".to_string());
    }
    if expression.starts_with("true") || expression.starts_with("false") {
        return Some("boolean".to_string());
    }
    if expression.starts_with(|c: char| c.is_ascii_digit() || c == '-') {
        return Some("number".to_string());
    }
    if expression.contains(".toUpperCase(")
        || expression.contains(".toLowerCase(")
        || expression.contains(".trim(")
    {
        return Some("string".to_string());
    }
    if expression.contains("===")
        || expression.contains("!==")
        || expression.contains(">=")
        || expression.contains("<=")
        || expression.contains(" > ")
        || expression.contains(" < ")
    {
        return Some("boolean".to_string());
    }
    if expression.contains('*') || expression.contains('/') || expression.contains(" - ") {
        return Some("number".to_string());
    }

    None
}
