use vize_carton::{FxHashSet, String, append, cstr};
use vize_croquis::croquis::{ComponentUsage, PassedProp};

use super::inline_callback_classifier::is_direct_inline_function_prop_value;
use crate::virtual_ts::helpers::{to_camel_case, to_safe_identifier_fragment};

/// Whether this prop's authored value is an inline function, which is the only
/// shape that needs the `__VizeCallableProp` fallback.
///
/// The name filters mirror [`append_per_prop_aliases`] exactly, because the two
/// must agree: emitting the helper for a prop the alias loop then skips leaves
/// it unreferenced, which is `TS6196` on a clean SFC.
pub(super) fn is_inline_callback_prop(prop: &PassedProp) -> bool {
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

/// The target the usage's whole props object literal is checked against.
///
/// `__VizeExactOptionalProps<__X_Props_N>` reports **only** what the per-prop
/// check structurally cannot: the `exactOptionalPropertyTypes` distinction
/// between "the property is absent" and "the property is present and
/// `undefined`" (#3450). That distinction only exists when a whole object
/// literal is assigned at once, which is why the per-prop path cannot express
/// it — it extracts `label?: string` to `string | undefined` through
/// `__VizePropValue` and assigns to that, legal whatever the option says.
///
/// Every property is widened *with* `{} | null`, a union that accepts any value
/// except `undefined`. That is the whole trick: an ordinary type mismatch stays
/// the per-prop check's to report, at its own well-anchored position, and is not
/// reported a second time here. Checking against the child's props type
/// unmodified — or against `Partial<>` of it — reports every wrongly-typed prop
/// twice, one byte apart, with an identical code and message that
/// `dedup_diagnostics` cannot collapse because the positions differ.
///
/// The declared type stays in the union rather than being replaced by `{} | null`
/// because it is the only thing that contextually types an inline callback prop.
/// Against a bare `{} | null` the parameters of `:textConverter="(value) => …"`
/// have no contextual signature to draw from and become implicit `any`, a
/// `TS7006` on correct code, which is what `check_function_props_cli` guards.
/// Since a function is assignable to `{}`, keeping the declared member adds
/// contextual typing without adding a single rejection.
///
/// `null` has to be in the widened type, not just `{}`: a child prop declared
/// `LinkBehavior | null` is legitimately passed `null`, and `{}` alone rejects
/// it. Only `undefined` is this check's business.
///
/// `Required<Pick<P, K>>[K]` rather than `P[K]` because indexed access on an
/// optional property already includes `undefined`, so `P[K]` cannot tell
/// `label?: string` from `label?: string | undefined` and would make the check
/// inert. Stripping the modifier first leaves only an *explicitly* declared
/// `undefined`, which the child opted into and which must stay silent, and it
/// leaves that `undefined` *in the union*, so no conditional branch is needed to
/// allow it.
///
/// Consequences, covered by the component-props and project tests:
///
/// * a usage that binds at least one named prop keeps every other property
///   optional here, so missing-required-prop diagnostics are not duplicated by
///   this widened path. Empty and spread-only usages select the full child type
///   below, which is what catches their missing required props (#3444, #3527).
/// * `class`, `style`, `data-*`, `aria-*` and anything else the child does not
///   declare are absorbed by the `Record<string, unknown>` intersection
///   `__VizePropChecker` applies, which also suppresses object-literal excess
///   property checking.
/// * the alias does not have to agree with the literal's key set. The literal
///   skips props whose value does not generate; a `Pick<>` over the authored
///   names would have had to reproduce that filtering exactly or report phantom
///   missing properties.
/// * with `exactOptionalPropertyTypes` off the check is inert, because an
///   optional property accepts `undefined` implicitly — which is also what
///   `vue-tsc` does.
///
/// Only the **non-generic** branch of `__VizePropChecker` uses this type; a
/// generic child resolves through its own `__vizeCheck` signature and ignores
/// it, so the generic inference path is untouched.
///
/// Note the code divergence. TypeScript 6, which `vue-tsc` pins, reports this as
/// `TS2379`; the `@typescript/native-preview` build vize runs reports the
/// identical code against the identical target as `TS2345` with the same
/// explanation nested one level down. Confirmed by running both compilers over
/// the same file across five target shapes, including `vue-tsc`'s own. It is a
/// compiler-version difference, not something the generated code can steer.
pub(super) fn append_prop_checker_alias(
    ts: &mut String,
    usage: &ComponentUsage,
    component_type_name: &str,
    component_ref: &str,
    idx: usize,
) {
    // A usage that binds nothing by name is checked against the child's props
    // type *unmodified*. Nothing on that element is covered by the per-prop
    // path, so nothing can be reported twice. This covers both spread-only
    // usages (#3444) and an empty `<Child />`, whose `{}` must still fail when
    // the child has required props (#3527).
    //
    // Any usage that also binds by name keeps the widened target: its named
    // props belong to the per-prop check, and the full type there would
    // duplicate every one of them.
    let target = if has_inference_props(usage) {
        cstr!("__VizeExactOptionalProps<__{component_type_name}_Props_{idx}>")
    } else {
        cstr!("__{component_type_name}_Props_{idx}")
    };
    append!(
        *ts,
        "  type __{component_type_name}_CheckProps_{idx} = {target};\n",
    );
    append!(
        *ts,
        "  type __{component_type_name}_Check_{idx} = __VizePropChecker<typeof {component_ref}, __{component_type_name}_CheckProps_{idx}>;\n",
    );
}

/// The shared type helpers every per-usage prop check resolves through, emitted
/// once per template scope that has at least one checkable component usage.
///
/// They live here rather than at the call site because
/// [`append_prop_checker_alias`] is what names them, and the reasoning for
/// `__VizeExactOptionalProps` in particular belongs beside the alias it builds.
pub(super) fn append_prop_check_helpers(ts: &mut String, usages: &[(usize, &ComponentUsage)]) {
    ts.push_str("  type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;\n");
    ts.push_str(
        "  type __VizePropChecker<C, P> = __VizeIsAny<C> extends true ? (props: P & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: P & Record<string, unknown>) => void) : (props: P & Record<string, unknown>) => void;\n",
    );
    ts.push_str(
        "  type __VizePropValue<P, K extends PropertyKey, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? unknown : __V;\n",
    );
    ts.push_str(
        "  type __VizeExactOptionalProps<P> = { [K in keyof P]?: Required<Pick<P, K>>[K] | {} | null };\n",
    );
    // Emitted only when a usage actually binds an inline callback, because
    // nothing else references it and an unreferenced alias is `TS6196:
    // '__VizeCallableProp' is declared but never used`. That reaches
    // check-server clients as an unmapped hint on an otherwise clean SFC, the
    // same way the native element aliases did before #3443. The ambient
    // `declare function` trick those use is not available here: these helpers
    // are emitted inside a template scope's function body, not at module level.
    //
    // A generic child's props come from its `__vizeCheck<T>(props)` call, so
    // `__X_Props_N` is `Record<string, unknown>` and every per-prop alias
    // resolves to `unknown`. An inline callback prop annotated `unknown` has
    // no contextual type, so `strict` reports TS7006 on parameters that are
    // in fact contextually typed by the checker call below — a new error on
    // correct code (#3446). Falling back to a permissive callable gives
    // those parameters a contextual `any` and reports nothing itself. `any`
    // is excluded so a genuinely `any` prop stays assignable from a
    // non-function value, and a resolved prop type is returned untouched so
    // a real mismatch on a non-generic child still surfaces.
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_callback_prop))
    {
        ts.push_str(
            "  type __VizeCallableProp<T> = __VizeIsAny<T> extends true ? T : unknown extends T ? (...args: any[]) => any : T;\n",
        );
    }
}

/// The type a per-prop check is annotated with.
///
/// An inline callback prop gets the `__VizeCallableProp` fallback so a generic
/// child — whose per-prop type resolves to `unknown`, its props coming from the
/// `__vizeCheck<T>(props)` call instead — still contextually types the
/// callback's parameters (#3446). Every other prop keeps the resolved type.
pub(super) fn prop_alias_type(
    prop: &PassedProp,
    component_type_name: &str,
    idx: usize,
    camel_prop_name: &str,
) -> String {
    let resolved =
        cstr!("__VizePropValue<__{component_type_name}_Props_{idx}, '{camel_prop_name}'>");
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
        if prop.name_is_dynamic || prop.name.as_str() == "key" || prop.name.as_str() == "ref" {
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
            prop_alias_type(prop, component_type_name, idx, &camel_prop_name),
        );
    }
}
