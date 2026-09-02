use vize_s0::{FxHashSet, String, ToCompactString};

use super::JsxComponent;
use crate::JsxOutputMode;

pub(super) fn module_code(components: &[JsxComponent], preamble: String, source: &str) -> String {
    if let Some(module) = stateful_module_code(components, preamble.as_str(), source) {
        return module;
    }
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

fn stateful_module_code(
    components: &[JsxComponent],
    preamble: &str,
    source: &str,
) -> Option<String> {
    if components.is_empty() {
        return None;
    }
    if !components.iter().all(|component| {
        component.mode() == JsxOutputMode::Vdom && component.component_setup().is_some()
    }) {
        return None;
    }

    let mut ordered: Vec<&JsxComponent> = components.iter().collect();
    ordered.sort_by_key(|component| component.component_setup().unwrap().declaration_start);

    let mut module = String::default();
    module.push_str(preamble);
    if !module.is_empty() && !module.ends_with('\n') {
        module.push('\n');
    }
    module.push_str("import { defineComponent as _defineComponent } from \"vue\"\n");

    let mut cursor = 0usize;
    for component in ordered {
        let setup = component.component_setup()?;
        let name = component.component_name()?.trim();
        if !is_ascii_js_identifier(name) {
            return None;
        }
        let declaration_start = setup.declaration_start as usize;
        let declaration_end =
            declaration_end_with_semicolon(source, setup.declaration_end as usize);
        if declaration_start < cursor || declaration_end > source.len() {
            return None;
        }
        let render = local_render_code(component.code())?;

        module.push_str(&source[cursor..declaration_start]);
        push_component_wrapper(&mut module, source, name, setup, render.as_str());
        cursor = declaration_end;
    }
    module.push_str(&source[cursor..]);
    Some(module)
}

fn declaration_end_with_semicolon(source: &str, end: usize) -> usize {
    let bytes = source.as_bytes();
    if bytes.get(end).is_some_and(|byte| *byte == b';') {
        end + 1
    } else {
        end
    }
}

fn local_render_code(code: &str) -> Option<String> {
    if code
        .lines()
        .any(|line| line.trim_start().starts_with("import "))
    {
        return None;
    }
    Some(code.replacen("export function render(", "function render(", 1))
        .filter(|local| local.as_str() != code)
        .map(|local| local.to_compact_string())
}

fn push_component_wrapper(
    module: &mut String,
    source: &str,
    name: &str,
    setup: &crate::ComponentSetupSpan,
    render: &str,
) {
    module.push_str("const ");
    module.push_str(name);
    module.push_str(" = _defineComponent({\n  name: \"");
    module.push_str(&escape_js_string(name));
    module.push_str("\",\n");
    push_props_option(module, &setup.destructured_props);
    module.push_str("  ");
    if setup.is_async {
        // The authored body may `await`, so dropping the modifier would emit a
        // module that does not parse.
        module.push_str("async ");
    }
    module.push_str("setup");
    let type_params = setup_type_params(source, setup);
    if !type_params.is_empty() {
        module.push('<');
        module.push_str(type_params);
        module.push('>');
    }
    module.push('(');
    module.push_str(setup_params(source, setup));
    module.push_str(") {\n");

    let setup_source = &source[setup.setup_start as usize..setup.setup_end as usize];
    push_indented_trimmed(module, setup_source, "    ");
    push_indented_trimmed(module, render, "    ");

    module.push_str("    return render\n  }\n});");
}

/// Declare the destructured prop names so Vue routes caller values to `setup`'s
/// first argument instead of to `attrs` (#3861).
///
/// Defaults stay in the destructuring pattern, mirroring how SFC reactive props
/// destructure behaves, so only the names belong here.
fn push_props_option(module: &mut String, props: &[String]) {
    if props.is_empty() {
        return;
    }
    module.push_str("  props: [");
    for (index, prop) in props.iter().enumerate() {
        if index > 0 {
            module.push_str(", ");
        }
        module.push('"');
        module.push_str(&escape_js_string(prop));
        module.push('"');
    }
    module.push_str("],\n");
}

/// The component function's own parameter list, reused verbatim as the `setup()`
/// signature.
///
/// Vue calls `setup(props, ctx)`, which is exactly the JSX component contract, so
/// destructured props and their defaults keep working instead of becoming free
/// variables in the render closure (#3856).
fn setup_params<'a>(source: &'a str, setup: &crate::ComponentSetupSpan) -> &'a str {
    source
        .get(setup.params_start as usize..setup.params_end as usize)
        .map_or("", str::trim)
}

/// The component function's own type parameter list, re-emitted as a generic
/// `setup<T>()` method.
///
/// Without it, a generic component's forwarded parameter annotations would
/// reference type names that the replaced declaration no longer binds.
fn setup_type_params<'a>(source: &'a str, setup: &crate::ComponentSetupSpan) -> &'a str {
    source
        .get(setup.type_params_start as usize..setup.type_params_end as usize)
        .map_or("", str::trim)
}

fn push_indented_trimmed(out: &mut String, block: &str, indent: &str) {
    let block = block.trim();
    if block.is_empty() {
        return;
    }
    for line in block.lines() {
        out.push_str(indent);
        out.push_str(line);
        out.push('\n');
    }
}

fn escape_js_string(value: &str) -> String {
    let mut escaped = String::default();
    for ch in value.chars() {
        match ch {
            '\\' => escaped.push_str("\\\\"),
            '"' => escaped.push_str("\\\""),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            _ => escaped.push(ch),
        }
    }
    escaped
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
