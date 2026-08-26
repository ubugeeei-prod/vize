use vize_canon::virtual_ts::VirtualTsOptions;

pub(super) fn add_art_target_component_bindings(
    options: &mut VirtualTsOptions,
    summary: &vize_croquis::Croquis,
    target: &crate::virtual_code::ArtTargetComponent,
) {
    if target.source.trim().is_empty() {
        return;
    }

    let has_component_binding = summary.bindings.bindings.contains_key(target.name.as_str());
    let mut component_ref = target.name.clone();

    if !has_component_binding {
        let import_alias = vize_s0::cstr!(
            "__VizeArtTarget_{}",
            to_safe_identifier_fragment(target.name.as_str())
        );
        options.auto_import_stubs.push(vize_s0::cstr!(
            "import {import_alias} from {};",
            quote_ts_string(target.source.as_str())
        ));
        component_ref = import_alias.to_string();

        if is_valid_identifier(target.name.as_str()) {
            options
                .auto_import_stubs
                .push(vize_s0::cstr!("const {} = {import_alias};", target.name));
            options
                .external_template_bindings
                .push(target.name.clone().into());
            component_ref = target.name.clone();
        }
    }

    if has_component_binding {
        options
            .external_template_bindings
            .push(target.name.clone().into());
    }

    if let Some(kebab_name) = kebab_case_component_name(target.name.as_str()) {
        let kebab_ref = to_safe_identifier(kebab_name.as_str());
        options
            .auto_import_stubs
            .push(vize_s0::cstr!("const {kebab_ref} = {component_ref};"));
        options.external_template_bindings.push(kebab_name.into());
    }
}

fn quote_ts_string(value: &str) -> String {
    let mut quoted = String::from("\"");
    for ch in value.chars() {
        match ch {
            '\\' => quoted.push_str("\\\\"),
            '"' => quoted.push_str("\\\""),
            '\n' => quoted.push_str("\\n"),
            '\r' => quoted.push_str("\\r"),
            '\t' => quoted.push_str("\\t"),
            _ => quoted.push(ch),
        }
    }
    quoted.push('"');
    quoted
}

fn to_safe_identifier_fragment(value: &str) -> String {
    let mut result = String::with_capacity(value.len().max(1));
    for ch in value.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' || ch == '$' {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    if result.is_empty() {
        result.push('_');
    }
    result
}

fn to_safe_identifier(value: &str) -> String {
    let mut result = to_safe_identifier_fragment(value);
    if !result
        .chars()
        .next()
        .is_some_and(|ch| ch.is_ascii_alphabetic() || ch == '_' || ch == '$')
    {
        result.insert(0, '_');
    }
    if is_reserved_identifier(result.as_str()) {
        result.insert(0, '_');
    }
    result
}

fn is_valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    (first.is_ascii_alphabetic() || first == '_' || first == '$')
        && !is_reserved_identifier(value)
        && chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '$')
}

fn is_reserved_identifier(value: &str) -> bool {
    matches!(
        value,
        "await"
            | "break"
            | "case"
            | "catch"
            | "class"
            | "const"
            | "continue"
            | "debugger"
            | "default"
            | "delete"
            | "do"
            | "else"
            | "enum"
            | "export"
            | "extends"
            | "false"
            | "finally"
            | "for"
            | "function"
            | "if"
            | "import"
            | "in"
            | "instanceof"
            | "new"
            | "null"
            | "return"
            | "super"
            | "switch"
            | "this"
            | "throw"
            | "true"
            | "try"
            | "typeof"
            | "var"
            | "void"
            | "while"
            | "with"
            | "yield"
    )
}

fn kebab_case_component_name(name: &str) -> Option<String> {
    let mut kebab = String::new();
    let mut previous_was_separator = false;

    for (index, ch) in name.char_indices() {
        if ch.is_ascii_uppercase() {
            if index > 0 && !previous_was_separator {
                kebab.push('-');
            }
            kebab.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        } else if ch == '_' || ch == '-' || ch.is_whitespace() {
            if !kebab.ends_with('-') && !kebab.is_empty() {
                kebab.push('-');
            }
            previous_was_separator = true;
        } else {
            kebab.push(ch.to_ascii_lowercase());
            previous_was_separator = false;
        }
    }

    while kebab.ends_with('-') {
        kebab.pop();
    }
    (kebab.contains('-') && kebab != name).then_some(kebab)
}
