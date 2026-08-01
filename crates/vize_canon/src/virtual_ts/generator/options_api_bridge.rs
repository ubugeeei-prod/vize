//! Options API typed-instance bridge for the virtual TypeScript generator.
//!
//! The bridge re-emits every `computed`/`methods` function body inside setup
//! scope with an explicit `this: __VizeThis` receiver, so `this.foo` in an
//! Options API component is type-checked instead of silently `any`.
//!
//! Only the Vue 2 dialect emits it. A Vue 3 options object is generated (or
//! wrapped) as `defineComponent({ ... })`, so Vue's own options machinery
//! already types `this` inside the authored copy; re-checking those bodies
//! against the approximate `__VizeThis` shape adds no coverage, and now that
//! the copies anchor on authored bytes it would publish findings vue-tsc never
//! reports: implicit `any` callback parameters off `any`-typed members, plus
//! every member the shape cannot see (`$emit`, plugin globals, and mixins that
//! are not component constructors — see [`inherited`]). The Vue 2
//! `defineComponent` shim types no receiver at all, which is where the bridge
//! remains the only checker.
//!
//! Because those bodies are verbatim copies of authored bytes, every copy is
//! published as a [`VizeMapping`]. Without them a diagnostic raised inside a
//! bridged body has no authored origin, and `CompositeSourceMap` reports the
//! *generated* offset as if it were an authored one — anchoring real findings
//! hundreds of lines away, inside whatever block happens to sit at that offset
//! (#3582 moved them by exactly the number of bytes it added to the preamble).

mod collect;
mod inherited;

use std::ops::Range;

use vize_croquis::{BindingType, Croquis};

use self::collect::collect_options_api_bridge;
use self::inherited::{INHERITED_MEMBER_HELPERS, InheritedComponent};
use super::options_api_support::{extend_options_api_descriptor_names, is_safe_value_identifier};
use crate::virtual_ts::types::VizeMapping;
use vize_carton::{CompactString, String, append, profile};

pub(super) fn generate_options_api_bridge(
    ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    script: &str,
    script_offset: usize,
) {
    profile!(
        "canon.virtual_ts.generate_options_api_bridge",
        emit_options_api_bridge(ts, mappings, summary, script, script_offset)
    );
}

fn emit_options_api_bridge(
    mut ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    summary: &Croquis,
    script: &str,
    script_offset: usize,
) {
    // Matches the gate in `generate_options_api_variables`: the typed-instance
    // bridge is only meaningful for non-`<script setup>` components.
    if summary.bindings.is_script_setup {
        return;
    }

    let Some(bridge) = collect_options_api_bridge(script) else {
        return;
    };
    if bridge.computed.is_empty() && bridge.methods.is_empty() {
        return;
    }

    let mut names: Vec<&str> = summary
        .bindings
        .bindings
        .iter()
        .filter_map(|(name, binding_type)| {
            let name = name.as_str();
            match binding_type {
                BindingType::Data | BindingType::Options | BindingType::Props => {
                    is_safe_value_identifier(name).then_some(name)
                }
                _ => None,
            }
        })
        .collect();
    extend_options_api_descriptor_names(&mut names, summary);
    names.sort_unstable();
    names.dedup();

    ts.push_str("  // Options API typed instance bridge\n");
    for (index, mapped_type) in bridge.mapped_types.iter().enumerate() {
        // `ReturnType<typeof store>` only instantiates for a store *factory*;
        // a plain store object raises TS2344 here. Anchor the alias on the
        // authored spread so that finding cannot escape as an unmapped offset.
        let gen_start = ts.len();
        append!(
            ts,
            "  type __VizeOptionsMap{index} = {{ {} }};\n",
            mapped_type.text
        );
        mappings.push(VizeMapping {
            gen_range: gen_start..ts.len(),
            src_range: shift(script_offset, &mapped_type.src),
            sub_spans: Vec::new(),
        });
    }
    emit_inherited_aliases(ts, mappings, &bridge.inherited, script_offset);
    ts.push_str("  type __VizeThis = {\n");
    for name in names {
        append!(ts, "    {name}: any;\n");
    }
    ts.push_str("  }");
    for index in 0..bridge.mapped_types.len() {
        append!(ts, " & __VizeOptionsMap{index}");
    }
    for index in 0..bridge.inherited.len() {
        append!(ts, " & __VizeInherited{index}");
    }
    ts.push_str(";\n");

    for function in bridge.computed.iter().chain(bridge.methods.iter()) {
        emit_bridge_function(ts, mappings, function, script_offset);
    }

    if !bridge.computed.is_empty() || !bridge.methods.is_empty() {
        ts.push_str("  ");
        let mut first = true;
        for function in bridge.computed.iter().chain(bridge.methods.iter()) {
            if !first {
                ts.push(' ');
            }
            append!(
                ts,
                "void __vize_{}_{};",
                function.kind_prefix(),
                function.safe_name
            );
            first = false;
        }
        ts.push('\n');
    }
    ts.push('\n');
}

/// Emit one `__VizeInherited{index}` alias per `mixins:` / `extends:` entry.
///
/// Each alias is anchored on the authored reference it was derived from: a
/// finding raised by the `typeof` query (an unresolved import, say) must land
/// on the author's `mixins: [base]` instead of escaping as an unmapped
/// generated offset, which `CompositeSourceMap` would then report as if it
/// were an authored one.
fn emit_inherited_aliases(
    mut ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    inherited: &[InheritedComponent],
    script_offset: usize,
) {
    if inherited.is_empty() {
        return;
    }
    ts.push_str(INHERITED_MEMBER_HELPERS);
    for (index, component) in inherited.iter().enumerate() {
        let gen_start = ts.len();
        append!(
            ts,
            "  type __VizeInherited{index} = __VizeInheritedMembers<typeof {}>;\n",
            component.reference
        );
        mappings.push(VizeMapping {
            gen_range: gen_start..ts.len(),
            src_range: shift(script_offset, &component.src),
            sub_spans: Vec::new(),
        });
    }
}

fn emit_bridge_function(
    mut ts: &mut String,
    mappings: &mut Vec<VizeMapping>,
    function: &OptionsFunction,
    script_offset: usize,
) {
    let params = if function.params.is_empty() {
        String::from("this: __VizeThis")
    } else {
        let mut params = String::from("this: __VizeThis, ");
        params.push_str(&function.params);
        params
    };
    // The authored modifiers have to survive the copy: dropping `async` makes
    // TypeScript report TS1308 on the author's `await`, and dropping `*` makes
    // it report TS1163 on their `yield`.
    let modifier = if function.is_async { "async " } else { "" };
    let star = if function.is_generator { "*" } else { "" };
    let kind = function.kind_prefix();
    let gen_start = ts.len();
    append!(
        ts,
        "  {modifier}function{star} __vize_{kind}_{}({params}) ",
        function.safe_name
    );
    let body_gen_start = ts.len();
    ts.push_str(&function.body);
    let gen_end = ts.len();
    ts.push('\n');

    let spans = &function.spans;
    // Coarse mapping: the synthetic receiver and the rewritten signature carry
    // no authored bytes of their own, so they clamp into the authored function
    // instead of escaping as an unmapped generated offset.
    mappings.push(VizeMapping {
        gen_range: gen_start..gen_end,
        src_range: shift(script_offset, &spans.function),
        sub_spans: Vec::new(),
    });
    // Precise mapping: the body is copied byte for byte, so offsets inside it
    // translate 1:1 and land on the authored token that raised the diagnostic.
    if let Some((body_gen_offset, src)) = spans.body.as_ref() {
        let start = body_gen_start.saturating_add(*body_gen_offset);
        mappings.push(VizeMapping {
            gen_range: start..start.saturating_add(src.end.saturating_sub(src.start)),
            src_range: shift(script_offset, src),
            sub_spans: Vec::new(),
        });
    }
}

fn shift(script_offset: usize, span: &Range<usize>) -> Range<usize> {
    script_offset.saturating_add(span.start)..script_offset.saturating_add(span.end)
}

#[derive(Debug, Default)]
struct OptionsApiBridge {
    computed: Vec<OptionsFunction>,
    methods: Vec<OptionsFunction>,
    mapped_types: Vec<MappedType>,
    /// `mixins:` / `extends:` references whose members `__VizeThis` inherits.
    inherited: Vec<InheritedComponent>,
}

/// One `...mapState(store, [...])` spread lowered to a mapped type, paired with
/// the script-relative range of the authored call it was derived from.
#[derive(Debug)]
struct MappedType {
    text: String,
    src: Range<usize>,
}

#[derive(Debug)]
struct OptionsFunction {
    kind: OptionsFunctionKind,
    safe_name: CompactString,
    params: String,
    body: String,
    is_async: bool,
    is_generator: bool,
    /// Authored spans this function reproduces.
    spans: BridgeSpans,
}

/// Script-relative authored ranges behind one generated bridge function.
#[derive(Debug)]
struct BridgeSpans {
    /// The whole authored function, from its parameter list to its body close.
    function: Range<usize>,
    /// The authored range reproduced verbatim inside the generated body,
    /// paired with that copy's byte offset within `OptionsFunction::body`.
    body: Option<(usize, Range<usize>)>,
}

impl OptionsFunction {
    fn kind_prefix(&self) -> &'static str {
        match self.kind {
            OptionsFunctionKind::Computed => "computed",
            OptionsFunctionKind::Method => "method",
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum OptionsFunctionKind {
    Computed,
    Method,
}
