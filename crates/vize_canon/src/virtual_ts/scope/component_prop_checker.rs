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

/// Authored names that might be declared component props.
///
/// `class` and `style` are always fallthrough attributes in Vize's component
/// surface. Other names stay in the union: the generated type decides whether
/// they are actual props or merely arbitrary attributes.
fn authored_prop_key_union(usage: &ComponentUsage) -> String {
    let mut seen = FxHashSet::default();
    let mut keys = String::default();
    for prop in &usage.props {
        if prop.name_is_dynamic
            || matches!(prop.name.as_str(), "key" | "ref" | "class" | "style")
            || (prop.is_dynamic && prop.value.is_none())
        {
            continue;
        }
        let name = to_camel_case(prop.name.as_str());
        if !seen.insert(name.clone()) {
            continue;
        }
        if !keys.is_empty() {
            keys.push_str(" | ");
        }
        keys.push('"');
        for ch in name.chars() {
            match ch {
                '\\' => keys.push_str("\\\\"),
                '"' => keys.push_str("\\\""),
                '\n' => keys.push_str("\\n"),
                '\r' => keys.push_str("\\r"),
                '\t' => keys.push_str("\\t"),
                _ => keys.push(ch),
            }
        }
        keys.push('"');
    }
    if keys.is_empty() {
        keys.push_str("never");
    }
    keys
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
/// * when the usage binds a declared prop, properties it did not pass are not
///   reported by this path — every property is optional. If it binds only
///   fallthrough attributes, `__VizeWholeProps` keeps the full type so required
///   props are still checked (#3566).
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
/// The extracted `$props` type also contains Vue's `PublicProps` and listener
/// props synthesized from `emits`. Those are accepted at a component boundary,
/// but authoring one does not satisfy the component's own props contract. The
/// helper therefore subtracts both key sets before deciding that an authored
/// name is declared. Vize-generated components expose exact listener keys via
/// `__vizeEmitProps`. External Vue components expose them through their typed
/// `$emit`; the generic-event guard rejects Vue's untyped `(event: string)`
/// fallback before mapping listener props. Listener suffixes try both the raw
/// spelling and its uncapitalized form: Vue maps both `XML` and `xML` to
/// `onXML`, so reversing that key through `Uncapitalize` alone is lossy. Their
/// static typed `emits` option is retained as a second source for component
/// constructors that expose it directly. Vue 2 does not export `PublicProps`,
/// so that import deliberately degrades to `any`, which the tuple-guard
/// converts to an empty key set.
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
        let keys = authored_prop_key_union(usage);
        cstr!(
            "__VizeWholeProps<typeof {component_ref}, __{component_type_name}_Props_{idx}, {keys}>"
        )
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
    if usages.iter().any(|(_, usage)| has_inference_props(usage)) {
        ts.push_str(
            "  type __VizeExactOptionalProps<P> = { [K in keyof P]?: Required<Pick<P, K>>[K] | {} | null };\n",
        );
        ts.push_str(
            "  // @ts-ignore TS2694/TS2307: Vue 2 has no PublicProps; an unresolved alias degrades to any and therefore contributes no keys.\n",
        );
        ts.push_str("  type __VizeVuePublicProps = import('vue').PublicProps;\n");
        ts.push_str(
            "  type __VizeVuePublicPropKeys = [__VizeIsAny<__VizeVuePublicProps>] extends [true] ? never : keyof __VizeVuePublicProps;\n",
        );
        ts.push_str(
            "  type __VizeUsageCamelize<S extends string> = S extends `${infer H}-${infer T}` ? `${H}${Capitalize<__VizeUsageCamelize<T>>}` : S;\n",
        );
        ts.push_str(
            "  type __VizeUsageEventProp<K extends string> = `on${Capitalize<__VizeUsageCamelize<K>>}`;\n",
        );
        ts.push_str(
            "  type __VizeComponentEmitPropKeys<C> = '__vizeEmitProps' extends keyof C ? C extends { __vizeEmitProps?: infer P } ? keyof P : never : C extends { emits?: infer E } ? [__VizeIsAny<E>] extends [true] ? never : __VizeUsageEventProp<E extends readonly (infer N extends string)[] ? N : keyof NonNullable<E> & string> : never;\n",
        );
        ts.push_str(
            "  type __VizeEventNameForProp<K> = K extends `on${infer N}` ? Uncapitalize<N> : never;\n",
        );
        ts.push_str("  type __VizeEventSuffixForProp<K> = K extends `on${infer N}` ? N : never;\n");
        ts.push_str(
            "  type __VizeInstanceEmitPropKeys<C, P> = C extends { new (...args: any[]): { $emit: infer F } } ? [__VizeIsAny<F>] extends [true] ? never : F extends (event: string, ...args: any[]) => any ? never : { [K in keyof P]: K extends `on${string}` ? F extends (event: __VizeEventSuffixForProp<K>, ...args: any[]) => any ? K : F extends (event: __VizeEventNameForProp<K>, ...args: any[]) => any ? K : never : never }[keyof P] : never;\n",
        );
        ts.push_str(
            "  type __VizeDeclaredPropKeys<C, P> = Exclude<keyof P, __VizeVuePublicPropKeys | __VizeComponentEmitPropKeys<C> | __VizeInstanceEmitPropKeys<C, P>>;\n",
        );
        ts.push_str(
            "  type __VizeWholeProps<C, P, K extends PropertyKey> = P extends unknown ? [Extract<__VizeDeclaredPropKeys<C, P>, K>] extends [never] ? P : __VizeExactOptionalProps<P> : never;\n",
        );
    }
    // Emitted only when a usage actually binds an inline callback, because
    // nothing else references these aliases and an unreferenced one is
    // `TS6196`. That reaches
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
    // correct code (#3446). `__VizeCallableProp` remains the safe fallback for
    // components without Vize's resolver. A generic Vize child is invoked once
    // through `__VizePropsResolver`; `__VizeResolvedProp` then selects the
    // instantiated callback type so return errors surface inside the authored
    // body, at the same leaf byte as vue-tsc. `any` is excluded from the
    // fallback so a genuinely `any` prop stays assignable from a non-function
    // value, and a resolved non-generic prop type is returned untouched.
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_callback_prop))
    {
        ts.push_str(
            "  type __VizeCallableProp<T> = __VizeIsAny<T> extends true ? T : unknown extends T ? (...args: any[]) => any : T;\n",
        );
        ts.push_str(
            "  type __VizePropsResolver<C> = C extends { __vizeResolveProps?: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => {}) : (props: any) => {};\n",
        );
        ts.push_str(
            "  type __VizePropsSelector<R> = <A extends Partial<R> & Record<string, unknown>>(props: A) => A;\n",
        );
        ts.push_str("  type __VizeMissingProp = { readonly __vizeMissingProp: unique symbol };\n");
        ts.push_str(
            "  type __VizeResolvedPropEntry<R, K extends PropertyKey> = R extends unknown ? K extends keyof R ? { value: R[K] } : __VizeMissingProp : never;\n",
        );
        ts.push_str(
            "  type __VizeSelectedProps<R, A> = R extends unknown ? A extends Partial<R> ? R : never : never;\n",
        );
        ts.push_str(
            "  type __VizeResolvedProp<R, A, K extends PropertyKey, F, __S = __VizeSelectedProps<R, A>, __E = __VizeResolvedPropEntry<__S, K>, __A = __VizeResolvedPropEntry<R, K>, __P = Extract<__E, { value: unknown }>> = [__S] extends [never] ? F : [__P] extends [never] ? [Extract<__A, { value: unknown }>] extends [never] ? F : never : __P extends { value: infer V } ? V : never;\n",
        );
    }
}

/// The type a per-prop check is annotated with.
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
