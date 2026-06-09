//! Shared text-emission helpers for v-for loops and v-slot prop types.

use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;

use crate::virtual_ts::helpers::to_safe_identifier;

/// Type annotation for a `v-slot` scope's props. When the slot is on a child
/// component (`component` is `Some`), the props are inferred from that child's
/// `$slots[name]` parameter (its `defineSlots`), so misuse raises a real
/// diagnostic (#764). Dynamic slot names are matched against the union of all
/// declared slot function props, matching Vue's runtime lookup without
/// treating the expression text as a static slot key. Otherwise — and whenever
/// the child has no typed slot — it falls back to `any` so untyped or built-in
/// slot hosts never produce a false positive.
pub(super) fn slot_props_type(
    component: Option<&str>,
    slot_name: &str,
    slot_name_is_static: bool,
) -> String {
    match component {
        Some(component) => {
            let component_ref = to_safe_identifier(component);
            if slot_name_is_static {
                cstr!(
                    "typeof {component_ref} extends {{ new (): {{ $slots: infer __S }} }} ? (__S extends {{ \"{slot_name}\"?: (props: infer __P, ...args: any[]) => any }} ? __P : any) : any"
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
/// passed through verbatim so any `as Type` assertion flows into the helper.
pub(super) fn emit_v_for_loop_open(
    ts: &mut String,
    indent: &str,
    value_alias: &str,
    key_alias: Option<&str>,
    index_alias: Option<&str>,
    source: &str,
) {
    append!(*ts, "{indent}__vForList({source}).forEach(([{value_alias}");
    if let Some(key) = key_alias {
        append!(*ts, ", {key}");
    } else if index_alias.is_some() {
        // Keep the index in the third tuple slot even when no key alias is bound.
        ts.push_str(", _key");
    }
    if let Some(index) = index_alias {
        append!(*ts, ", {index}");
    }
    ts.push_str("]) => {\n");
}
