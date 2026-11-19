//! Shared text-emission helpers for v-for loops and v-slot prop types.

use crate::virtual_ts::template_binding_access::TemplateBindingAccess;
use oxc_syntax::identifier::is_identifier_part;
use vize_carton::String;
use vize_carton::append;
use vize_carton::cstr;
use vize_croquis::{Scope, ScopeData};
use vize_l0::ts;

use crate::virtual_ts::expressions::{
    map_rewritten_template_binding, rewrite_reserved_template_binding,
};
use crate::virtual_ts::types::VizeMapping;

#[derive(Clone, Copy)]
pub(super) struct SlotParameterPart<'a> {
    pub(super) text: &'a str,
    pub(super) start: usize,
}

#[derive(Clone, Copy)]
pub(super) struct SlotParameter<'a> {
    pub(super) pattern: SlotParameterPart<'a>,
    pub(super) annotation: Option<SlotParameterPart<'a>>,
    pub(super) default: Option<SlotParameterPart<'a>>,
}

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

/// Split a slot function parameter into its authored binding pattern, optional
/// annotation and optional whole-parameter default. The inferred payload type
/// belongs before that default: `({ editor }: Payload = {})`, not the invalid
/// `({ editor } = {}: Payload)`.
pub(super) fn slot_parameter(pattern: &str) -> SlotParameter<'_> {
    let mut depth = 0i32;
    let mut quote: Option<char> = None;
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
        match ch {
            '\'' | '"' | '`' => quote = Some(ch),
            '{' | '[' | '(' | '<' => depth += 1,
            '}' | ']' | ')' | '>' if depth > 0 => depth -= 1,
            ':' if depth == 0 && annotation_at.is_none() => annotation_at = Some(at),
            '=' if depth == 0 && is_parameter_default_equals(pattern, at) => {
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
pub(super) fn split_slot_pattern_annotation(pattern: &str) -> Option<(&str, &str)> {
    let parameter = slot_parameter(pattern);
    let annotation = parameter.annotation?.text;
    (!annotation.is_empty()).then_some((parameter.pattern.text, annotation))
}

/// Emit the opening of a v-slot scope function. The parameter is annotated
/// with the slot type inferred from the child's `$slots`; an authored
/// annotation types the bindings instead (see [`emit_slot_annotation_check`]).
pub(super) fn emit_slot_function_open(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    function_name: &str,
    props_pattern: &str,
    props_type: &String,
    capture: bool,
    body: impl FnOnce(&mut crate::virtual_ts::TextBuf),
) {
    let declaration = if capture {
        cstr!("var {function_name} = function")
    } else {
        cstr!("void function {function_name}")
    };
    let parameter = slot_parameter(props_pattern);
    let pattern = parameter.pattern.text;
    let annotation = parameter
        .annotation
        .map_or(props_type.as_str(), |annotation| annotation.text);
    let params = parameter.default.map_or_else(
        || cstr!("{pattern}: {annotation}"),
        |default| cstr!("{pattern}: {annotation} = {}", default.text),
    );
    append!(*ts, "{indent}{declaration}({params})");
    // The `{` and `};` are one tight group, so the closer cannot drift.
    ts! { &mut *ts =>
        #(" ") {
            #("\n")
            #{ body(__ts); }
            #indent
        }
        #(";\n")
    }
}

/// vue-tsc's `__VLS_vSlot(slot, (_: Annotation) => [] as any)`: the annotated
/// callback must be assignable to the declared slot member, so a method slot
/// compares its parameter bivariantly and an object-declared slot rejects any
/// annotation. TypeScript reports the callback at its `(`, which maps to the
/// slot expression's first byte, where vue-tsc reports it. The other bytes
/// stay unmapped so the authored pattern keeps its own spans.
pub(super) fn emit_slot_annotation_check(
    ts: &mut crate::virtual_ts::TextBuf,
    diagnostics: &mut Vec<VizeMapping>,
    indent: &str,
    slot_member_type: &str,
    annotation: &str,
    expression_start: usize,
) {
    append!(
        *ts,
        "{indent}(undefined as unknown as <__S, __D extends __S>(slot: __S, decl: __D) => void)(undefined as unknown as ({slot_member_type}), "
    );
    diagnostics.push(VizeMapping {
        gen_range: ts.len()..ts.len() + 1,
        src_range: expression_start..expression_start + 1,
        sub_spans: Vec::new(),
    });
    append!(*ts, "(_: {annotation}) => [] as any);\n");
}

pub(super) fn append_v_for_comment(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    label: &str,
    alias: &str,
    source: &str,
) {
    // The comment is one line. A line break in the alias or the source would
    // otherwise become code and stop the semantic pass (#12932).
    append!(*ts, "\n{indent}// {label}: ");
    push_comment_text(ts, alias);
    append!(*ts, " in ");
    push_comment_text(ts, source);
    ts.push('\n');
}

fn push_comment_text(ts: &mut crate::virtual_ts::TextBuf, text: &str) {
    for c in text.chars() {
        if c == '\n' || c == '\r' {
            ts.push(' ');
        } else {
            ts.push(c);
        }
    }
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
/// Mappings written while the loop header is emitted. Callers that keep a
/// diagnostics copy pass the whole [`super::context::ScopeMappings`].
pub(super) trait AsMappings {
    fn as_mappings(&mut self) -> &mut Vec<VizeMapping>;
}

impl AsMappings for Vec<VizeMapping> {
    fn as_mappings(&mut self) -> &mut Vec<VizeMapping> {
        self
    }
}

impl AsMappings for super::context::ScopeMappings<'_> {
    fn as_mappings(&mut self) -> &mut Vec<VizeMapping> {
        self.navigation
    }
}

#[expect(
    clippy::too_many_arguments,
    clippy::redundant_closure_call,
    reason = "independent emitter inputs and a ts! interpolation writer"
)]
pub(super) fn emit_v_for_loop_open<M>(
    ts: &mut crate::virtual_ts::TextBuf,
    mappings: &mut M,
    template_offset: u32,
    source_offset: Option<u32>,
    // A replay of a loop the template pass already maps leaves its aliases
    // unmapped, so their diagnostics are reported once.
    map_aliases: bool,
    indent: &str,
    scope: &Scope,
    template_binding_access: &TemplateBindingAccess,
    capture: bool,
    body: impl FnOnce(&mut crate::virtual_ts::TextBuf, &mut M),
) where
    M: AsMappings,
{
    // The scope is authoritative for both the loop shape and the alias
    // declaration offsets, so the v-for data is read from it directly.
    let ScopeData::VFor(data) = scope.data() else {
        body(ts, mappings);
        return;
    };
    // Loop bindings are in the TDZ while a for-of RHS is evaluated. Vue
    // evaluates the source in the parent scope, so capture it before entering
    // the loop. A block also isolates repeated projections of this scope.
    // The block braces and the `for` braces are each one `ts!`.
    let scope_id = scope.id.as_u32();
    ts! { &mut *ts =>
        #indent
        {
            #("\n")
            #{
                if capture {
                    append!(*__ts, "{indent}var __vize_slot_scope_{scope_id} = (() => {{\n");
                }
                (move |ts: &mut crate::virtual_ts::TextBuf| {
                    let source_name = cstr!("__vize_v_for_source_{scope_id}");
                    append!(*ts, "{indent}const {source_name} = __vForList(");
                    let source_gen_start = ts.len();
                    // A loop source runs before that loop's own aliases are introduced.
                    let parent_access =
                        template_binding_access.for_scope(scope.parent().unwrap_or(scope.id));
                    let template_binding_access = &parent_access;
                    let rewritten_source = rewrite_reserved_template_binding(
                        data.source.as_str(),
                        template_binding_access,
                    );
                    let source_text = rewritten_source
                        .as_ref()
                        .map_or(data.source.as_str(), |source| source.as_str());
                    let isolated_source =
                        crate::virtual_ts::expressions::isolate_incomplete_expression(source_text);
                    ts.push_str(isolated_source.as_str());
                    let (mapped_start, mapped_len) = isolated_source.mapped_bounds();
                    let source_gen_end = source_gen_start + mapped_start + mapped_len;
                    if let Some(source_offset) = source_offset {
                        let source_start = (template_offset + source_offset) as usize;
                        mappings.as_mappings().push(VizeMapping {
                            gen_range: source_gen_start..source_gen_end,
                            src_range: source_start..(source_start + data.source.len()),
                            sub_spans: Vec::new(),
                        });
                        map_rewritten_template_binding(
                            ts,
                            mappings.as_mappings(),
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
                    for name in data.value_bindings.iter().filter(|_| map_aliases) {
                        if let Some(relative) =
                            pattern_identifier_offset(data.value_alias.as_str(), name)
                        {
                            map_alias(
                                mappings.as_mappings(),
                                scope,
                                template_offset,
                                name,
                                pattern_gen_start + relative,
                            );
                        }
                    }
                    if let Some(key) = data.key_alias.as_deref() {
                        ts.push_str(", ");
                        if map_aliases {
                            map_alias(mappings.as_mappings(), scope, template_offset, key, ts.len());
                        }
                        ts.push_str(key);
                    } else if data.index_alias.is_some() {
                        // Keep the index in the third tuple slot even when no key alias is bound.
                        ts.push_str(", _key");
                    }
                    if let Some(index) = data.index_alias.as_deref() {
                        ts.push_str(", ");
                        if map_aliases {
                            map_alias(mappings.as_mappings(), scope, template_offset, index, ts.len());
                        }
                        ts.push_str(index);
                    }
                    append!(*ts, "] of {source_name})");
                    ts! { &mut *ts =>
                        #(" ") {
                            #("\n")
                            #{ body(__ts, mappings); }
                            #indent
                        }
                        #("\n")
                    }
                })(__ts);
                if capture {
                    append!(*__ts, "{indent}return [];\n{indent}}})();\n");
                }
            }
            #indent
        }
        #("\n")
    }
}

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

    #[test]
    fn multi_line_v_for_alias_stays_inside_the_scope_comment() {
        let alias = "{\n      bar,\n      baz,\n    }";
        let source = "items\r\n";
        let mut ts = crate::virtual_ts::TextBuf::default();
        super::append_v_for_comment(&mut ts, "  ", "v-for scope", alias, source);
        let flat = |text: &str| text.replace(['\n', '\r'], " ");
        let expected = format!("\n  // v-for scope: {} in {}\n", flat(alias), flat(source));
        assert_eq!(ts.as_str(), expected);
        assert_eq!(
            ts.as_str().lines().filter(|line| !line.is_empty()).count(),
            1
        );
    }
}
