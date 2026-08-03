//! Shared text-emission helpers for v-for loops and v-slot prop types.

use vize_carton::append;
use vize_carton::cstr;
use vize_carton::{FxHashSet, String};
use vize_carton::{camelize, capitalize};
use vize_croquis::{Croquis, VForScopeData};

use crate::virtual_ts::types::{VirtualTsOptions, VizeMapping};
use crate::virtual_ts::{expressions::rewrite_reserved_template_prop, helpers::to_safe_identifier};

/// Type annotation for a `v-slot` scope's props. When the slot is on a child
/// component (`component` is `Some`), the props are inferred from that child's
/// `$slots[name]` parameter (its `defineSlots`), so misuse raises a real
/// diagnostic (#764). Dynamic slot names are matched against the union of all
/// declared slot function props, matching Vue's runtime lookup without
/// treating the expression text as a static slot key. Otherwise — and whenever
/// the child has no typed slot — it falls back to `any` so untyped or built-in
/// slot hosts never produce a false positive.
pub(super) fn slot_props_type(
    summary: &Croquis,
    options: &VirtualTsOptions,
    component: Option<&str>,
    slot_name: &str,
    slot_name_is_static: bool,
) -> String {
    match component {
        Some(component) => {
            let component_ref = component_binding_reference(summary, options, component);
            if slot_name_is_static {
                cstr!(
                    "typeof {component_ref} extends {{ new (): {{ $slots: infer __S }} }} ? (\"{slot_name}\" extends keyof __S ? (NonNullable<__S[\"{slot_name}\"]> extends (props: infer __P, ...args: any[]) => any ? __P : any) : any) : any"
                )
            } else {
                cstr!(
                    "typeof {component_ref} extends {{ new (): {{ $slots: infer __S }} }} ? ({{ [__K in keyof __S]: NonNullable<__S[__K]> extends (props: infer __P, ...args: any[]) => any ? __P : never }}[keyof __S] extends infer __P ? ([__P] extends [never] ? any : __P) : any) : any"
                )
            }
        }
        None => "any".into(),
    }
}

fn component_binding_reference(
    summary: &Croquis,
    options: &VirtualTsOptions,
    template_name: &str,
) -> String {
    let camel_name = camelize(template_name);
    let pascal_name = capitalize(camel_name.as_str());
    for candidate in [template_name, camel_name.as_str(), pascal_name.as_str()] {
        if summary.bindings.bindings.contains_key(candidate)
            || options
                .external_template_bindings
                .iter()
                .any(|name| name.as_str() == candidate)
        {
            return to_safe_identifier(candidate);
        }
    }
    to_safe_identifier(template_name)
}

/// Split a `v-slot` props expression that carries its own TypeScript
/// annotation (`#item="{ element }: { element: Tag }"`) into the binding
/// pattern and the annotation. Vue allows the annotation, so emitting the
/// inferred slot type after the full expression would produce `pattern: A: B`
/// — a syntax error that aborts Corsa's semantic pass for the whole project.
/// A `:` separates the annotation only at the top nesting level.
pub(super) fn split_slot_pattern_annotation(pattern: &str) -> Option<(&str, &str)> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
    let mut escaped = false;

    for (at, ch) in pattern.char_indices() {
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
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '{' | '[' | '(' | '<' => depth += 1,
            '}' | ']' | ')' | '>' => depth -= 1,
            ':' if depth == 0 => {
                let annotation = pattern[at + 1..].trim();
                if annotation.is_empty() {
                    return None;
                }
                return Some((pattern[..at].trim_end(), annotation));
            }
            _ => {}
        }
    }
    None
}

/// Emit the opening of a v-slot scope function. The parameter is annotated
/// with the slot type inferred from the child's `$slots`; when the user wrote
/// their own annotation, that annotation is kept for the bindings and a
/// separate assignment asserts the child's actual slot props are assignable
/// to it, mirroring how vue-tsc validates annotated slot props.
pub(super) fn emit_slot_function_open(
    ts: &mut String,
    indent: &str,
    function_name: &str,
    props_pattern: &str,
    props_type: &String,
) {
    if let Some((pattern, annotation)) = split_slot_pattern_annotation(props_pattern) {
        append!(
            *ts,
            "{indent}void function {function_name}({pattern}: {annotation}) {{\n"
        );
        append!(
            *ts,
            "{indent}  const __slot_annotation_check: {annotation} = undefined as unknown as ({props_type});\n{indent}  void __slot_annotation_check;\n"
        );
    } else {
        append!(
            *ts,
            "{indent}void function {function_name}({props_pattern}: {props_type}) {{\n"
        );
    }
}

pub(super) fn append_v_for_comment(
    ts: &mut String,
    indent: &str,
    label: &str,
    alias: &str,
    source: &str,
) {
    append!(*ts, "\n{indent}// {label}: {alias} in ");
    for c in source.chars() {
        if c == '\n' || c == '\r' {
            ts.push(' ');
        } else {
            ts.push(c);
        }
    }
    ts.push('\n');
}

/// Emit the opening of a v-for scope as
/// `__vForList(source).forEach(([value, key, index]) => {`.
///
/// The overloaded `__vForList` helper types the destructured tuple from the
/// source kind: arrays/iterables/numbers/strings keep a numeric `key`, while an
/// object source yields `value: T[keyof T]` and `key: keyof T` (matching
/// vue-tsc) instead of the old array-only `(source).forEach` assumption that
/// mis-typed objects and raised spurious TS2339/TS2537. The source expression is
/// rewritten through the template-prop bridge so a source such as `messages`
/// resolves to `__props.messages`; all other authored syntax stays verbatim.
pub(super) fn emit_v_for_loop_open(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    template_offset: u32,
    source_offset: Option<u32>,
    indent: &str,
    data: &VForScopeData,
    template_prop_names: &FxHashSet<String>,
) {
    append!(*ts, "{indent}__vForList(");
    let source_gen_start = ts.len();
    let rewritten_source =
        rewrite_reserved_template_prop(data.source.as_str(), template_prop_names);
    ts.push_str(
        rewritten_source
            .as_ref()
            .map_or(data.source.as_str(), |source| source.as_str()),
    );
    let source_gen_end = ts.len();
    if let Some(source_offset) = source_offset {
        let source_start = (template_offset + source_offset) as usize;
        mappings.push(VizeMapping {
            gen_range: source_gen_start..source_gen_end,
            src_range: source_start..(source_start + data.source.len()),
            sub_spans: Vec::new(),
        });
    }
    append!(*ts, ").forEach(([{}", data.value_alias);
    if let Some(key) = data.key_alias.as_deref() {
        append!(*ts, ", {key}");
    } else if data.index_alias.is_some() {
        // Keep the index in the third tuple slot even when no key alias is bound.
        ts.push_str(", _key");
    }
    if let Some(index) = data.index_alias.as_deref() {
        append!(*ts, ", {index}");
    }
    ts.push_str("]) => {\n");
}
