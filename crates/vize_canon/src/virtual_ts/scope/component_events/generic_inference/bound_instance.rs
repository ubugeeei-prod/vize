//! Bound-props instances of unmarked constructors.
//!
//! An unmarked constructor (a packaged `DefineComponent`, a hand-typed
//! constructor intersection) reaches `$EmitPropsFactory` through inference
//! from its last construct signature, whose type parameters TypeScript erases
//! to their constraints: `new <T>(props: Props<T>)` reads `Props<unknown>`, and
//! so do its `$props` listeners. vue-tsc constructs the component with the
//! bound props instead (`new Comp(props)`), which resolves the overloads and
//! infers `T` from them.
//!
//! The gated call of such a component captures the type of its props literal,
//! and a `new` expression constructs the component with that type. A listener
//! prop of the instance's `$props` then types the payload, ahead of the
//! factory's own inference, as vue-tsc reads props before emits. Every other
//! route leaves no capture, so its instance is `{}` and the payload is the
//! factory's, as before. A component imported from an SFC is always marked and
//! never takes the capture, so its usages emit none of this.

use vize_carton::{String, append, cstr};
use vize_croquis::{Croquis, ScopeData, analysis::ComponentUsage};

/// Whether a usage's emit-props call constructs a bound instance: a static
/// target with valued props and no spread, under the emit gate, whose binding
/// is not an SFC import. Every listener of the usage reads the same answer.
pub(super) fn applies(
    ctx: &super::EmitInferenceContext<'_, '_>,
    usage: &ComponentUsage,
    has_inference_prop: bool,
    dynamic_component: bool,
) -> bool {
    ctx.gated_inference
        && has_inference_prop
        && !dynamic_component
        && usage.spread_props.is_empty()
        && !imports_sfc(ctx.summary, ctx.component_ref)
}

/// `component_ref`'s root binding is a value import from a `.vue` module.
fn imports_sfc(summary: &Croquis, component_ref: &str) -> bool {
    let is_ident = |c: char| c == '_' || c == '$' || c.is_alphanumeric();
    let root = component_ref
        .split(|c: char| !is_ident(c))
        .next()
        .unwrap_or_default();
    summary.scopes.iter().any(|scope| {
        matches!(
            scope.data(),
            ScopeData::ExternalModule(module)
                if !module.is_type_only
                    && module.source.ends_with(".vue")
                    && module.exports.iter().any(|export| export.local_name == root)
        )
    })
}

/// The gate input `input` with the props literal's type captured on the
/// unmarked-constructor routes. Their factory takes the props alone and is not
/// generic, so a type parameter bounded by its input takes the literal's type,
/// and contextual typing still reads that input.
fn capture(library: &str, component_ref: &str, input: &str) -> String {
    cstr!(
        "({input} extends infer __F ? [{library}] extends [true] ? __F : 0 extends (1 & typeof {component_ref}) ? __F : typeof {component_ref} extends {{ readonly __vizeComponentMarker: true }} ? __F : typeof {component_ref} extends abstract new (...args: any) => any ? __F extends (props: infer __P) => infer __R ? <__A extends __P>(props: __A) => __R & {{ readonly __vizeBoundProps: [__A] }} : __F : __F : never)"
    )
}

fn input_type(library: &str, component_ref: &str, usage_idx: usize, hoisted: bool) -> String {
    let input = crate::virtual_ts::helpers::shared_types::EMIT_GATE_INPUT.select(hoisted);
    capture(
        library,
        component_ref,
        &cstr!(
            "{input}<{library}, typeof {component_ref}, typeof __vize_emit_factory_{usage_idx}>"
        ),
    )
}

/// The gated call, with its input captured when `bound`.
pub(super) fn append_gated_call(
    bound: bool,
    ts: &mut crate::virtual_ts::TextBuf,
    (indent, library, component_ref, emit_props): (&str, &str, &str, &str),
    (usage_idx, hoisted): (usize, bool),
) {
    if !bound {
        super::super::emit_props_gate::append_gated_call(
            ts,
            indent,
            library,
            component_ref,
            emit_props,
            (usage_idx, hoisted),
        );
        return;
    }
    let resolver = crate::virtual_ts::helpers::shared_types::EMIT_GATE_RESOLVER.select(hoisted);
    let input = input_type(library, component_ref, usage_idx, hoisted);
    append!(
        *ts,
        "{indent}const __vize_emit_factory_{usage_idx} = (undefined as unknown as {resolver}<{library}, typeof {component_ref}>)({component_ref}); const {emit_props} = (undefined as unknown as {input})(",
    );
}

/// The guarded gated call, with its input captured when `bound`.
pub(super) fn append_guarded_input(
    bound: bool,
    ts: &mut crate::virtual_ts::TextBuf,
    (indent, guard, library, component_ref): (&str, &str, &str, &str),
    (usage_idx, hoisted): (usize, bool),
) {
    if !bound {
        super::super::emit_props_gate::append_guarded_input(
            ts,
            indent,
            guard,
            library,
            component_ref,
            (usage_idx, hoisted),
        );
        return;
    }
    let input = input_type(library, component_ref, usage_idx, hoisted);
    append!(
        *ts,
        "{indent}if ({guard}) return (undefined as unknown as {input})(",
    );
}

/// Constructs the component with the captured props type. A direct call
/// captures nothing, so its instance is `{}`; the instance exists wherever the
/// emit-props call does, for every later listener of the usage.
pub(super) fn append_instance(
    ts: &mut crate::virtual_ts::TextBuf,
    indent: &str,
    component_ref: &str,
    usage_idx: usize,
) {
    append!(
        *ts,
        "{indent}// @ts-ignore Inference-only bound instance; the canonical component-props call owns diagnostics.\n{indent}const __vize_emit_instance_{usage_idx} = new (undefined as unknown as (typeof __vize_emit_props_{usage_idx} extends {{ readonly __vizeBoundProps: [unknown] }} ? typeof {component_ref} : new (props: unknown) => {{}}))(undefined as unknown as (typeof __vize_emit_props_{usage_idx} extends {{ readonly __vizeBoundProps: [infer __A] }} ? __A : never));\n",
    );
}

/// Opens and closes the inferred-arguments alias around the factory's own
/// inference: the bound instance's listener arguments win when they are known.
pub(super) fn args_frame(
    bound: bool,
    usage_idx: usize,
    prop_key: &str,
    raw_key: &str,
) -> (String, &'static str) {
    if !bound {
        return (String::default(), "");
    }
    let props = cstr!(
        "(typeof __vize_emit_instance_{usage_idx} extends {{ $props: infer __P }} ? __P : {{}})"
    );
    let args = super::super::listener_contract::listener_args(&props, prop_key, raw_key);
    (
        cstr!("({args}) extends infer __B extends unknown[] ? unknown[] extends __B ? ("),
        ") : __B : never",
    )
}
