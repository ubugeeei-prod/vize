use vize_carton::{String, append, cstr};
use vize_croquis::croquis::ComponentUsage;

pub(super) fn is_inline_function_prop_value(value: &str) -> bool {
    let value = value.trim();
    value.contains("=>") || value.starts_with("function") || value.starts_with("async function")
}

/// Returns whether any checkable prop carries a value to type-check.
///
/// Static attribute values count: `msg="text"` must satisfy the child's
/// prop type just like `:msg="expr"`.
pub(super) fn has_value_props(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        !prop.name_is_dynamic
            && prop.name.as_str() != "key"
            && prop.name.as_str() != "ref"
            && prop.value.is_some()
    })
}

pub(super) fn has_inference_props(usage: &ComponentUsage) -> bool {
    usage.props.iter().any(|prop| {
        !prop.name_is_dynamic && prop.name.as_str() != "key" && prop.name.as_str() != "ref"
    })
}

/// Whether the usage has anything for the whole-props check to look at.
///
/// A `v-bind="obj"` spread contributes no `PassedProp`, so a usage that only
/// spreads has no inference props and used to be skipped entirely — which is
/// why `<Child v-bind="bag" />` was unchecked (#3444).
///
/// Shared with `expressions::component_props`, which gates the call this
/// module's aliases type. Two copies of the rule could disagree and leave the
/// generated TypeScript calling a type that was never declared.
pub(crate) fn has_checkable_props_or_spread(usage: &ComponentUsage) -> bool {
    has_inference_props(usage) || !usage.spread_props.is_empty()
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
/// Consequences, each a case in `component_props_tests.rs`:
///
/// * a prop the template did not pass is not reported — every property is
///   optional, so the missing-required-prop class stays off. That is also what
///   keeps this independent of #3444, whose spread needs the full type.
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
    // A usage that spreads and binds nothing by name is checked against the
    // child's props type *unmodified*. Nothing on that element is covered by the
    // per-prop path, so nothing can be reported twice, and it is the only shape
    // that catches a wrongly typed value *inside* the spread — #3444's oracle,
    // where `<Child v-bind="bag" />` with a string `count` is `TS2345`.
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
pub(super) fn append_prop_check_helpers(ts: &mut String) {
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
}
