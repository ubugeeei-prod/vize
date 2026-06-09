//! Shared TypeScript stub builders, dedup helpers, and tracked file IO.

use std::{fs, path::Path};

use vize_carton::{FxHashSet, String, ToCompactString, cstr, profile, profiler::global_profiler};

pub(super) fn push_declared_const(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    type_annotation: &str,
) -> bool {
    push_stub(
        stubs,
        seen_names,
        cstr!("declare const {name}: {type_annotation};"),
    )
}

pub(super) fn push_stub(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    stub: String,
) -> bool {
    let Some(name) = declared_name(&stub) else {
        stubs.push(stub);
        return true;
    };
    if seen_names.insert(name.to_compact_string()) {
        stubs.push(stub);
        return true;
    }
    false
}

pub(super) fn push_generic_function_stub(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
) -> bool {
    push_stub(stubs, seen_names, generic_function_stub(name))
}

pub(super) fn generic_function_stub(name: &str) -> String {
    cstr!("declare function {name}<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): any;")
}

pub(super) fn generic_composable_stub(name: &str) -> String {
    cstr!(
        "declare function {name}<T = any, T1 = any, T2 = any, T3 = any>(...args: any[]): ({{ value: T }} & Record<string, any>);"
    )
}

pub(super) fn push_named_overload_stubs(
    stubs: &mut Vec<String>,
    seen_names: &mut FxHashSet<String>,
    name: &str,
    overloads: Vec<String>,
) -> bool {
    if !seen_names.insert(name.to_compact_string()) {
        return false;
    }
    stubs.extend(overloads);
    true
}

pub(super) fn declared_name(stub: &str) -> Option<&str> {
    for prefix in [
        "declare function ",
        "declare const ",
        "declare let ",
        "declare var ",
    ] {
        let Some(rest) = stub.strip_prefix(prefix) else {
            continue;
        };
        let end = rest
            .find(['<', '(', ':', '=', ';', ' '])
            .unwrap_or(rest.len());
        let name = rest[..end].trim();
        if !name.is_empty() {
            return Some(name);
        }
    }
    None
}

pub(super) fn is_template_component_binding(name: &str) -> bool {
    name.chars()
        .next()
        .is_some_and(|first| first == '_' || first.is_ascii_uppercase())
}

#[allow(clippy::disallowed_types)]
pub(super) fn tracked_read_to_string(path: &Path) -> Result<std::string::String, std::io::Error> {
    match profile!("cli.check.nuxt.read", fs::read_to_string(path)) {
        Ok(content) => {
            global_profiler().record_fs_read_to_string(content.len());
            Ok(content)
        }
        Err(error) => {
            global_profiler().record_fs_read_to_string_failure();
            Err(error)
        }
    }
}
