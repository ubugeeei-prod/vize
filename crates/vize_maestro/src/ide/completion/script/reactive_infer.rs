//! Heuristic inference of reactive binding kinds and their inner value types
//! from raw script source, used to enrich completion details without a backing
//! Corsa type session.
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "inferred types are rendered into tower-lsp detail strings, which are std `String`"
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

    if let Some(source) = vize_croquis::facts::reactivity_lookup(&croquis, name) {
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
        let Some((_, initializer)) = script_content.split_once(declaration_start.as_str()) else {
            continue;
        };
        let initializer = initializer.trim_start();
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
        let Some((_, after)) = script_content.split_once(prefix.as_str()) else {
            continue;
        };
        if let Some(generic) = after.strip_prefix('<') {
            if let Some(end) = find_matching_angle(generic)
                && let Some(value_type) = generic.get(..end)
            {
                return Some(value_type.trim().to_string());
            }
        } else if let Some(arguments) = after.strip_prefix('(') {
            return infer_value_type_from_initializer(arguments, wrapper);
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
    for (i, c) in s.char_indices() {
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
    let (_, body) = initializer.split_once("=>")?;
    let body = body.trim_start();

    if let Some(body) = body.strip_prefix('{')
        && let Some((_, returned)) = body.split_once("return")
    {
        let returned = returned.trim_start();
        let returned = returned.split([';', '}']).next().unwrap_or(returned);
        return Some(returned.trim());
    }

    let body = body.split(['\n', ';']).next().unwrap_or(body);
    Some(body.trim().trim_end_matches(')').trim())
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
