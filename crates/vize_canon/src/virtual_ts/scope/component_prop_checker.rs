use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

use super::component_ref_props::is_inline_ref_callback_prop;
use super::inline_callback_classifier::is_direct_inline_function_prop_value;
use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier_fragment};

mod check_helpers;
pub(super) use check_helpers::append_prop_check_helpers;

/// Whether this prop's authored value is an inline function, which is the only
/// shape that needs the `__VizeCallableProp` fallback.
///
/// The name filters mirror [`append_per_prop_aliases`] exactly, because the two
/// must agree: emitting the helper for a prop the alias loop then skips leaves
/// it unreferenced, which is `TS6196` on a clean SFC.
pub(crate) fn is_inline_callback_prop(prop: &PassedProp) -> bool {
    if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
        return false;
    }
    prop.is_dynamic
        && prop
            .value
            .as_ref()
            .is_some_and(|value| is_direct_inline_function_prop_value(value.as_str()))
}

/// Whether the value contains an inline callback whose standalone generation
/// would lose contextual typing. Legacy Vue 2 globals use this broader shape
/// to avoid reporting `TS7006` for values such as `[(value) => !!value]`.
pub(super) fn contains_inline_function_prop_value(value: &str) -> bool {
    let value = value.trim();
    value.contains("=>") || value.starts_with("function") || value.starts_with("async function")
}

pub(super) fn has_inference_props(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        !prop.name_is_dynamic && prop.name.as_str() != "key" && prop.name.as_str() != "ref"
    })
}

/// The target the usage's whole props object literal is checked against. A
/// literal that omits a required prop is rejected as a whole (`TS2345` on the
/// element); a literal that passes a wrong value is elaborated to one `TS2322`
/// on that prop, matching vue-tsc's one-error behavior for #3569.
///
/// The elaborated `TS2322` lands on the literal's key, which maps back to the
/// authored attribute name — the same position, code and message the per-prop
/// check produces for that prop, so `dedup_diagnostics` collapses the pair. That
/// is why no widening is needed to keep a wrong prop from being reported twice.
///
/// Checking against the raw type is also the cheapest thing that can be
/// generated, which is a correctness property of its own here. An earlier
/// attempt at #3569 inferred the authored object as a generic `A` and picked
/// between a complete and a relaxed target by testing `A` against a projection
/// of the child's declared prop keys. On real projects that machinery both blew
/// past TypeScript's union-complexity limit (`TS2590`) and widened authored
/// string literals through the `A extends Record<string, unknown>` constraint,
/// turning `align="start"` into `string` and reporting correct code as wrong.
/// A checker that only works on toy inputs is worse than the bug it fixes, so
/// the whole-props target carries no conditional types, no mapped types and no
/// inference variable.
///
/// Consequences, covered by component-props and project tests: generated
/// children accept only public attrs unless a recorded fallthrough target opens
/// the attr tail, opaque children keep the permissive tail, Vue public/listener
/// props do not satisfy required own props, inline callback contextual typing
/// stays intact, and the `exactOptionalPropertyTypes` absent-vs-`undefined`
/// distinction survives.
///
/// Vize's public `$props` accepts camel- and kebab-case aliases, which requires
/// camel-case keys to be optional there. The generated props literal already
/// camelizes every authored static name, so the internal `__vizeRawProps`
/// marker preserves the declaration's requiredness for this whole-object
/// check without constructing every camel/kebab key combination. External
/// components have no marker and continue to use their public `$props`.
///
/// Only the **non-generic** branch of `__VizePropChecker` uses this type; a
/// generic child resolves through its own `__vizeCheck` signature and ignores
/// it, so the generic inference path is untouched.
///
/// Note the code divergence. TypeScript 6, which `vue-tsc` pins, reports the
/// exact-optional rejection as `TS2379`; the native TypeScript 7 runtime
/// vize runs reports the identical code against the identical target as `TS2345`
/// with the same explanation nested one level down. Confirmed by running both
/// compilers over the same file across five target shapes, including
/// `vue-tsc`'s own. It is a compiler-version difference, not something the
/// generated code can steer.
pub(super) fn append_prop_checker_alias(
    ts: &mut String,
    usage: &ComponentUsage,
    component_type_name: &str,
    component_ref: &str,
    idx: usize,
    relax_required: bool,
) {
    append!(
        *ts,
        "  type __{component_type_name}_Component_{idx} = typeof {component_ref};\n",
    );
    // The listener props synthesized from the child's `emits` join the check
    // target (#3890): they are part of Vue's public props contract, `vue-tsc`
    // lists them in the displayed parameter type, and their presence is what
    // types an authored `:on-save` binding instead of absorbing it as
    // `unknown`. A component without the marker contributes `{}`. So do the
    // child's required fallthrough props, which exist only for a child
    // generated under `checkRequiredFallthroughAttributes`.
    //
    // A usage that is itself this component's fallthrough root under that
    // option forwards the child's required props to the parent, so it checks
    // the values it binds without demanding the ones it forwards.
    let own_props = if relax_required {
        cstr!("Partial<__{component_type_name}_Props_{idx}>")
    } else {
        cstr!("__{component_type_name}_Props_{idx}")
    };
    append!(
        *ts,
        "  type __{component_type_name}_CheckProps_{idx} = {own_props} & __VizeEmitListeners<__{component_type_name}_Component_{idx}> & __VizeRequiredFallthroughProps<__{component_type_name}_Component_{idx}>;\n",
    );
    append!(
        *ts,
        "  type __{component_type_name}_CheckTail_{idx} = __VizeComponentCheckTail<__{component_type_name}_Component_{idx}>;\n",
    );
    if usage_needs_per_prop_aliases(usage) {
        append!(
            *ts,
            "  type __{component_type_name}_ValueProps_{idx} = __{component_type_name}_Props_{idx} & __VizeEmitListeners<__{component_type_name}_Component_{idx}> & __VizePublicComponentAttrs;\n",
        );
        append!(
            *ts,
            "  type __{component_type_name}_FallthroughValue_{idx}<K extends PropertyKey> = __VizeFallthroughValue<__{component_type_name}_Component_{idx}, K>;\n",
        );
    }
    append!(
        *ts,
        "  type __{component_type_name}_Check_{idx} = __VizePropChecker<__{component_type_name}_Component_{idx}, __{component_type_name}_CheckProps_{idx}, __{component_type_name}_CheckTail_{idx}>;\n",
    );
}

fn usage_needs_per_prop_aliases(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        (!prop.name_is_dynamic && prop.name.as_str() != "key" && prop.value.is_some())
            && (prop.name.as_str() != "ref" || is_inline_ref_callback_prop(prop))
    })
}

///
/// An inline callback prop gets the `__VizeCallableProp` fallback for a child
/// without `__vizeResolveProps`. Vize generic children replace it at the value
/// check with their instantiated resolver result. Every other prop keeps the
/// statically extracted type.
pub(super) fn prop_alias_type(
    prop: &PassedProp,
    component_type_name: &str,
    idx: usize,
    camel_prop_name: &str,
    fallthrough_prop_name: &str,
) -> String {
    if prop.name.as_str() == "ref" && is_inline_ref_callback_prop(prop) {
        return String::from("__VizeComponentRefCallback");
    }
    let resolved = cstr!(
        "__VizePropValue<__{component_type_name}_ValueProps_{idx}, '{camel_prop_name}', __{component_type_name}_FallthroughValue_{idx}<'{fallthrough_prop_name}'>>"
    );
    if is_inline_callback_prop(prop) {
        cstr!("__VizeCallableProp<{resolved}>")
    } else {
        resolved
    }
}

/// One `__X_N_prop_<name>` alias per distinct prop name the usage binds.
///
/// A repeated attribute — a static `class` next to a bound `:class` — reuses the
/// same child prop type, and emitting the alias twice would be a `TS2300` in the
/// generated module, so the name set is deduplicated.
pub(super) fn append_per_prop_aliases(
    ts: &mut String,
    usage: &ComponentUsage,
    component_type_name: &str,
    idx: usize,
) {
    let mut declared_aliases = FxHashSet::default();
    for prop in &usage.props {
        if prop.name_is_dynamic
            || prop.name.as_str() == "key"
            || (prop.name.as_str() == "ref" && !is_inline_ref_callback_prop(prop))
        {
            continue;
        }
        if prop.value.is_none() {
            continue;
        }
        let camel_prop_name = to_camel_case(prop.name.as_str());
        let safe_prop_name = to_safe_identifier_fragment(prop.name.as_str());
        if !declared_aliases.insert(safe_prop_name.clone()) {
            continue;
        }
        append!(
            *ts,
            "  type __{component_type_name}_{idx}_prop_{safe_prop_name} = {};\n",
            prop_alias_type(
                prop,
                component_type_name,
                idx,
                &camel_prop_name,
                prop.name.as_str()
            ),
        );
    }
}
