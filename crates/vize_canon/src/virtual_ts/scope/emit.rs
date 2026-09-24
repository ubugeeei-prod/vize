//! Shared text-emission helpers for v-for loops and v-slot prop types.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use oxc_syntax::identifier::is_identifier_part;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::{Scope, ScopeData};

use crate::virtual_ts::expressions::{
    map_rewritten_template_binding, rewrite_reserved_template_binding,
};
use crate::virtual_ts::types::VizeMapping;

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
                let (binding, annotation) = pattern.split_at_checked(at)?;
                let annotation = annotation.get(1..)?.trim();
                if annotation.is_empty() {
                    return None;
                }
                return Some((binding.trim_end(), annotation));
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
    capture: bool,
) {
    let declaration = if capture {
        cstr!("var {function_name} = function")
    } else {
        cstr!("void function {function_name}")
    };
    if let Some((pattern, annotation)) = split_slot_pattern_annotation(props_pattern) {
        append!(*ts, "{indent}{declaration}({pattern}: {annotation}) {{\n");
        append!(
            *ts,
            "{indent}  const __slot_annotation_check: {annotation} = undefined as unknown as ({props_type});\n{indent}  void __slot_annotation_check;\n"
        );
    } else {
        append!(
            *ts,
            "{indent}{declaration}({props_pattern}: {props_type}) {{\n"
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
/// a source capture followed by `for (const [value, key, index] of source) {`.
///
/// The `__vForList` helper types the destructured tuple from the
/// source kind: arrays/iterables/numbers/strings keep a numeric `key`, while an
/// object source yields `value: T[keyof T]` and `key: keyof T` (matching
/// vue-tsc) instead of the old array-only `(source).forEach` assumption that
/// mis-typed objects and raised spurious TS2339/TS2537. The source expression is
/// rewritten through the template-prop bridge so a source such as `messages`
/// resolves to `__props.messages`; all other authored syntax stays verbatim.
#[expect(clippy::too_many_arguments, reason = "independent emitter inputs")]
pub(super) fn emit_v_for_loop_open(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    template_offset: u32,
    source_offset: Option<u32>,
    indent: &str,
    scope: &Scope,
    template_binding_access: &TemplateBindingAccess,
    capture: bool,
) {
    // The scope is authoritative for both the loop shape and the alias
    // declaration offsets, so the v-for data is read from it directly.
    let ScopeData::VFor(data) = scope.data() else {
        return;
    };
    // Loop bindings are in the TDZ while a for-of RHS is evaluated. Vue
    // evaluates the source in the parent scope, so capture it before entering
    // the loop. A block also isolates repeated projections of this scope.
    append!(*ts, "{indent}{{\n");
    if capture {
        append!(
            *ts,
            "{indent}var __vize_slot_scope_{} = (() => {{\n",
            scope.id.as_u32()
        );
    }
    let source_name = cstr!("__vize_v_for_source_{}", scope.id.as_u32());
    append!(*ts, "{indent}const {source_name} = __vForList(");
    let source_gen_start = ts.len();
    let rewritten_source =
        rewrite_reserved_template_binding(data.source.as_str(), template_binding_access);
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
        map_rewritten_template_binding(
            ts,
            mappings,
            source_gen_start,
            source_start,
            data.source.as_str(),
            template_binding_access,
        );
    }
    ts.push_str(");\n");
    append!(*ts, "{indent}for (const [");
    // The alias pattern is emitted verbatim, so each binding identifier sits at
    // the same relative offset in the generated pattern as in the authored one.
    // Mapping it to the binding's declaration offset makes hover, definition
    // and references answer at the `v-for="it in …"` declaration itself, not
    // only at use sites (#3894).
    let pattern_gen_start = ts.len();
    ts.push_str(data.value_alias.as_str());
    for name in &data.value_bindings {
        if let Some(relative) = pattern_identifier_offset(data.value_alias.as_str(), name) {
            map_alias(
                mappings,
                scope,
                template_offset,
                name,
                pattern_gen_start + relative,
            );
        }
    }
    if let Some(key) = data.key_alias.as_deref() {
        ts.push_str(", ");
        map_alias(mappings, scope, template_offset, key, ts.len());
        ts.push_str(key);
    } else if data.index_alias.is_some() {
        // Keep the index in the third tuple slot even when no key alias is bound.
        ts.push_str(", _key");
    }
    if let Some(index) = data.index_alias.as_deref() {
        ts.push_str(", ");
        map_alias(mappings, scope, template_offset, index, ts.len());
        ts.push_str(index);
    }
    append!(*ts, "] of {source_name}) {{\n");
}

/// The byte offset at which `name` is declared inside the authored alias
/// pattern (`it`, `{ id, name }`, `[first, second]`).
///
/// Mirrors croquis' `find_identifier_token`, which derives the source-side
/// declaration offsets this maps to: both sides must pick the same token, or
/// the generated and authored ranges would name different identifiers.
pub(super) fn pattern_identifier_offset(pattern: &str, name: &str) -> Option<usize> {
    pattern
        .match_indices(name)
        .find(|(at, _)| is_declaration_token(pattern, *at, name.len()))
        .map(|(at, _)| at)
}

/// Whether the `[at, at + len)` slice of `text` is a binding declaration.
///
/// It must not be part of a longer identifier — bounded by ECMAScript's
/// `IdentifierPart`, so `it` matches neither inside `éit` the way a byte-wise
/// ASCII test would nor inside `a\u{301}it` or `it\u{200C}tail`, which a
/// `char::is_alphanumeric` test would miss — and must not be the property of a
/// member access (`{ kind = other.it, it }`), which references another binding
/// instead of declaring this one. A rest element (`[first, ...rest]`) is a
/// declaration and stays eligible.
fn is_declaration_token(text: &str, at: usize, len: usize) -> bool {
    let (Some(leading), Some(trailing)) = (text.get(..at), text.get(at + len..)) else {
        return false;
    };
    let before = leading.chars().next_back();
    let after = trailing.chars().next();
    if before.is_some_and(is_identifier_part) || after.is_some_and(is_identifier_part) {
        return false;
    }
    before != Some('.') || leading.ends_with("..")
}

/// Map one emitted alias identifier back to its authored declaration span.
fn map_alias(
    mappings: &mut Vec<VizeMapping>,
    scope: &Scope,
    template_offset: u32,
    name: &str,
    gen_start: usize,
) {
    let Some(binding) = scope.get_binding(name) else {
        return;
    };
    // Offset zero is the drawer's "never recorded" default; a real alias can
    // never start the template.
    if binding.declaration_offset == 0 {
        return;
    }
    let src_start = (template_offset + binding.declaration_offset) as usize;
    mappings.push(VizeMapping {
        gen_range: gen_start..(gen_start + name.len()),
        src_range: src_start..(src_start + name.len()),
        sub_spans: Vec::new(),
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn pattern_identifier_offsets_respect_word_boundaries() {
        assert_eq!(super::pattern_identifier_offset("it", "it"), Some(0));
        assert_eq!(
            super::pattern_identifier_offset("{ id, name }", "name"),
            Some(6)
        );
        // `item` must not match the `it` prefix.
        assert_eq!(super::pattern_identifier_offset("item", "it"), None);
        // A different binding's default value does not shadow the position.
        assert_eq!(
            super::pattern_identifier_offset("{ kind = fallback, it }", "it"),
            Some(19)
        );
        // A default value referencing a member named like the binding is a
        // reference, not the declaration.
        assert_eq!(
            super::pattern_identifier_offset("{ kind = other.it, it }", "it"),
            Some(19)
        );
        // A non-ASCII identifier that ends with the binding name is a
        // different identifier.
        // `é` is two bytes, so the declared `it` starts at byte 8.
        assert_eq!(
            super::pattern_identifier_offset("{ éit, it }", "it"),
            Some(8)
        );
        // A combining mark and a zero-width joiner continue an identifier too,
        // so neither host identifier yields the binding position.
        assert_eq!(
            super::pattern_identifier_offset("{ a\u{301}it, it }", "it"),
            Some(9)
        );
        assert_eq!(
            super::pattern_identifier_offset("{ it\u{200c}tail, it }", "it"),
            Some(13)
        );
        // A rest element declares its binding.
        assert_eq!(
            super::pattern_identifier_offset("[first, ...it]", "it"),
            Some(11)
        );
    }
}
