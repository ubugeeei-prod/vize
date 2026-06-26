use vize_carton::{FxHashSet, String, ToCompactString};

use super::JsxComponent;

pub(super) fn module_code(components: &[JsxComponent], preamble: String) -> String {
    let mut module = preamble;
    let multi_component = components.len() > 1;
    let mut render_export_names: FxHashSet<String> = FxHashSet::default();
    for (index, component) in components.iter().enumerate() {
        let code = component.code();
        if code.is_empty() {
            continue;
        }
        if !module.is_empty() && !module.ends_with('\n') {
            module.push('\n');
        }
        if multi_component {
            let export_name = unique_render_export_name(
                component.component_name(),
                index,
                &mut render_export_names,
            );
            module.push_str(&rename_render_export(code, &export_name));
        } else {
            module.push_str(code);
        }
    }
    module
}

pub(super) fn unique_render_export_name(
    component_name: Option<&str>,
    index: usize,
    used_names: &mut FxHashSet<String>,
) -> String {
    let mut candidate = component_name
        .filter(|name| is_ascii_js_identifier(name))
        .map(|name| name.to_compact_string())
        .unwrap_or_else(|| format_args!("render_{}", index + 1).to_compact_string());

    if used_names.insert(candidate.clone()) {
        return candidate;
    }

    let base = candidate;
    let mut suffix = 2;
    loop {
        candidate = format_args!("{}_{}", base, suffix).to_compact_string();
        if used_names.insert(candidate.clone()) {
            return candidate;
        }
        suffix += 1;
    }
}

pub(super) fn rename_render_export(code: &str, export_name: &str) -> String {
    let replacements = [
        ("export function render(", "export function "),
        ("function ssrRender(", "export function "),
    ];
    for (needle, replacement_prefix) in replacements {
        if let Some(start) = code.find(needle) {
            let mut renamed = String::default();
            renamed.push_str(&code[..start]);
            renamed.push_str(replacement_prefix);
            renamed.push_str(export_name);
            renamed.push('(');
            renamed.push_str(&code[start + needle.len()..]);
            return renamed;
        }
    }
    String::from(code)
}

fn is_ascii_js_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !is_ascii_js_identifier_start(first) {
        return false;
    }
    chars.all(is_ascii_js_identifier_continue)
}

fn is_ascii_js_identifier_start(c: char) -> bool {
    c == '_' || c == '$' || c.is_ascii_alphabetic()
}

fn is_ascii_js_identifier_continue(c: char) -> bool {
    is_ascii_js_identifier_start(c) || c.is_ascii_digit()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_export_names_prefer_component_names_and_fallbacks() {
        let mut used = FxHashSet::default();
        assert_eq!(
            unique_render_export_name(Some("InspectTable"), 0, &mut used),
            "InspectTable"
        );
        assert_eq!(
            unique_render_export_name(Some("InspectTable"), 1, &mut used),
            "InspectTable_2"
        );
        assert_eq!(
            unique_render_export_name(Some("値"), 2, &mut used),
            "render_3"
        );
        assert_eq!(unique_render_export_name(None, 3, &mut used), "render_4");
    }

    #[test]
    fn renames_vdom_and_ssr_render_exports() {
        assert_eq!(
            rename_render_export("export function render(_ctx, _cache) {\n}", "App"),
            "export function App(_ctx, _cache) {\n}"
        );
        assert_eq!(
            rename_render_export("function ssrRender(_ctx, _push) {\n}", "App"),
            "export function App(_ctx, _push) {\n}"
        );
    }
}
