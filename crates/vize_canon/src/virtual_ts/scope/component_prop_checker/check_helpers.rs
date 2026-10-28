//! The shared type helpers every per-usage prop check resolves through.

use vize_carton::String;
use vize_croquis::croquis::ComponentUsage;

use super::super::component_ref_props::append_ref_callback_helper;
use super::is_inline_callback_prop;

/// The shared type helpers every per-usage prop check resolves through, emitted
/// once per template scope that has at least one checkable component usage.
/// Keep this set small; the child's own props type carries the contract.
pub(in crate::virtual_ts::scope) fn append_prop_check_helpers(
    ts: &mut String,
    usages: &[(usize, &ComponentUsage)],
    check_unknown_props: bool,
    check_unknown_fallthrough_props: bool,
    strict_component_attrs: bool,
    typed_fallthrough_values: bool,
) {
    ts.push_str("  type $$HogeIsAny<T> = 0 extends (1 & T) ? true : false;\n");
    ts.push_str(
        "  type $$HogeComponentNavigationProps<P> = $$HogeIsAny<P> extends true ? Record<string, unknown> : unknown extends P ? {} : P;\n",
    );
    // Inline parameter shapes keep `TS2345` messages close to `vue-tsc` while
    // preserving optionality and contextual typing. The strict tail is used
    // only when Vize knows the generated child's fallthrough surface.
    ts.push_str(
        "  type $$HogeEmitListeners<C> = C extends { __vizeEmitProps?: infer __E } ? $$HogeIsAny<__E> extends true ? {} : NonNullable<__E> : {};\n",
    );
    ts.push_str(
        "  type $$HogeIsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;\n",
    );
    ts.push_str(
        "  type $$HogeHasFallthroughProps<C> = $$HogeIsGeneratedComponent<C> extends true ? C extends { readonly __vizeHasFallthroughProps: true } ? true : false : false;\n",
    );
    ts.push_str(
        "  type $$HogeFallthroughProps<C> = $$HogeHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $$HogeIsAny<__F> extends true ? {} : NonNullable<__F> : {} : {};\n",
    );
    // Only a child generated under `checkRequiredFallthroughAttributes` has
    // non-optional fallthrough props; every other child's surface is
    // `Partial<...>` or an open record, whose required keys are `never`.
    //
    // The no-key case resolves to the literal `{}` rather than an empty mapped
    // type, so the intersection it joins still displays as the child's `Props`
    // in diagnostics, the way `vue-tsc` prints it.
    ts.push_str(
        "  type $$HogeRequiredFallthroughProps<C, __F = $$HogeFallthroughProps<C>, __K = $$HogeRequiredKeys<__F>> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n",
    );
    ts.push_str("  type $$HogeVueVNodeProps = import('vue').VNodeProps;\n");
    ts.push_str("  type $$HogeVueAllowedComponentProps = import('vue').AllowedComponentProps;\n");
    ts.push_str("  type $$HogeVueComponentCustomProps = import('vue').ComponentCustomProps;\n");
    ts.push_str(
        "  interface $$HogePublicComponentAttrs extends $$HogeVueVNodeProps, $$HogeVueAllowedComponentProps, $$HogeVueComponentCustomProps {}\n",
    );
    if check_unknown_props {
        if strict_component_attrs {
            // Standard strict attrs accept Vue public attrs and only keys the
            // child's enabled fallthrough producer actually declares.
            ts.push_str(
                "  type $$HogeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $$HogeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $$HogePublicComponentAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            );
            ts.push_str(
                "  type $$HogeComponentCheckTail<C> = $$HogeIsGeneratedComponent<C> extends true ? $$HogePublicComponentAttrs & $$HogeAllowedFallthroughAttrs<C> : $$HogePublicComponentAttrs;\n"
            );
        } else {
            // Real HTML attributes are valid Vue fallthrough on any component, so
            // the strict surface accepts every attribute some native element
            // declares — `id`, `type`, `accept`, kebab `aria-*` plus camelized
            // spellings, custom `data-*` — while a name no element knows
            // (`depressed`) stays a strict finding (#4966). Reuse the program-wide
            // native attribute union: rebuilding it in each generated SFC costs
            // tens of thousands of type instantiations per module (#10171). It
            // degrades to `{}` on a `vue` without `NativeElements`, like the native
            // prop checks degrade.
            // Preserve producer-derived known forwarded names without an open index.
            // Already accepted public/HTML keys need no extra mapped intersection;
            // the no-extra-key case remains literal `{}` in diagnostic type display.
            ts.push_str(if check_unknown_fallthrough_props {
                "  type $$HogeAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $$HogeFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $$HogePublicComponentAttrs | keyof $$HogeComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            } else {
                "  type $$HogeAllowedFallthroughAttrs<C> = $$HogeHasFallthroughProps<C> extends true ? Record<string, unknown> : {};\n"
            });
            ts.push_str(
                "  type $$HogeComponentCheckTail<C> = $$HogeIsGeneratedComponent<C> extends true ? $$HogePublicComponentAttrs & $$HogeComponentGlobalHtmlAttrs & $$HogeAllowedFallthroughAttrs<C> : $$HogePublicComponentAttrs & $$HogeComponentGlobalHtmlAttrs;\n",
            );
        }
    } else {
        ts.push_str(
            "  type $$HogeComponentCheckTail<C> = $$HogeIsGeneratedComponent<C> extends true ? $$HogePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;\n",
        );
    }
    ts.push_str(
        "  type $$HogeComponentCheckProps<P, T> = { readonly [K in keyof P]: P[K] } & T;\n",
    );
    // Abstract component constructors carry the same instance props as concrete
    // constructors. Only an authored `any` instance has no prop contract; keep
    // typed instances closed rather than opening every constructor's attrs.
    ts.push_str(
        "  type $$HogePublicProps<C> = C extends abstract new (...args: any[]) => infer __I ? $$HogeIsAny<__I> extends true ? Record<string, unknown> : __I extends { $props: infer __P } ? __P : {} : C extends (props: infer __P, ...args: any[]) => any ? __P : {};\n",
    );
    ts.push_str(
        "  type $$HogeInstanceRawProps<C> = C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $$HogeIsAny<__P> extends true ? $$HogePublicProps<C> : __P : $$HogePublicProps<C>;\n",
    );
    ts.push_str(
        "  type $$HogeStaticRawProps<C> = C extends { readonly __vizeRawProps?: infer __P } ? $$HogeIsAny<__P> extends true ? $$HogePublicProps<C> : __P : $$HogePublicProps<C>;\n",
    );
    ts.push_str(
        "  type $$HogeHasInstanceRawProps<C> = $$HogeIsGeneratedComponent<C> extends true ? C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $$HogeIsAny<__P> extends true ? false : true : false : false;\n",
    );
    ts.push_str(
        "  type $$HogeHasStaticRawProps<C> = $$HogeIsGeneratedComponent<C> extends true ? C extends { readonly __vizeRawProps?: infer __P } ? $$HogeIsAny<__P> extends true ? false : true : false : false;\n",
    );
    // An empty options object matches an optional-only shape and infers
    // `unknown`. Trust raw metadata only when its generated identity is proven,
    // otherwise use the public component contract (including the empty one).
    ts.push_str(
        "  type $$HogeComponentRawProps<C, __P = $$HogeHasInstanceRawProps<C> extends true ? $$HogeInstanceRawProps<C> : $$HogeHasStaticRawProps<C> extends true ? $$HogeStaticRawProps<C> : $$HogePublicProps<C>> = unknown extends __P ? Record<string, unknown> : __P;\n",
    );
    ts.push_str(
        "  type $$HogeHasRawProps<C> = $$HogeHasInstanceRawProps<C> extends true ? true : $$HogeHasStaticRawProps<C>;\n",
    );
    ts.push_str(
        "  type $$HogeFallthroughValue<C, K extends PropertyKey> = $$HogeHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $$HogeIsAny<__F> extends true ? unknown : $$HogePropValue<NonNullable<__F>, K, unknown> : unknown : unknown;\n",
    );
    if usages.iter().any(|(_, usage)| {
        usage
            .events
            .iter()
            .any(|event| !event.name_is_dynamic && !event.name.is_empty())
    }) {
        ts.push_str(
            "  type $$HogeEventName<K extends string> = K extends `on${infer E}` ? Uncapitalize<E> | $$HogeKebabCase<Uncapitalize<E>> : never;\n",
        );
        ts.push_str(
            "  type $$HogeKebabEventAliases<E> = { [K in keyof E & string as $$HogeKebabCase<K> extends K ? never : $$HogeKebabCase<K>]: E[K] };\n",
        );
        ts.push_str(
            "  type $$HogeComponentEvents<C> = C extends { __vizeRawEmits?: infer __R; __vizeEventMap?: infer __E } ? [keyof NonNullable<__R>] extends [never] ? NonNullable<__E> : NonNullable<__R> & $$HogeKebabEventAliases<NonNullable<__R>> : { [K in keyof (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {}) & string as $$HogeEventName<K>]: (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {})[K] };\n",
        );
    }
    // A generic SFC compiled by vue-tsc (a library such as reka-ui) is a
    // function returning `VNode & { __ctx?: ... }`. Extracting its props
    // instantiates the generics with their constraints, so it is called
    // directly instead, as vue-tsc does, and TypeScript infers them.
    ts.push_str(
        "  type $$HogeIsLibraryGenericExport<C> = C extends { __vizeCheck: any } | { readonly __vizeComponentMarker: true } ? false : C extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
    );
    ts.push_str(
        "  type $$HogePropChecker<C, P, T = $$HogeComponentCheckTail<C>> = $$HogeIsAny<C> extends true ? (props: { readonly [K in keyof P]: P[K] } & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? $$HogeIsAny<__F> extends true ? (props: $$HogeComponentCheckProps<P, T>) => void : unknown extends __F ? (props: $$HogeComponentCheckProps<P, T>) => void : __F extends (...args: any[]) => any ? __F : (props: $$HogeComponentCheckProps<P, T>) => void : $$HogeIsLibraryGenericExport<C> extends true ? C : (props: $$HogeComponentCheckProps<P, T>) => void;\n",
    );
    ts.push_str(
        "  type $$HogePropValue<P, K extends PropertyKey, F = unknown, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? F : __V;\n",
    );
    if typed_fallthrough_values
        && usages
            .iter()
            .any(|(_, usage)| !usage.spread_props.is_empty())
    {
        ts.push_str(
            "  type $$HogeInvalidFallthroughSpreadKeys<P, F, A> = { [K in keyof A]-?: K extends keyof P | keyof $$HogeKebabProps<P> ? never : K extends keyof F ? $$HogeIsAny<A[K]> extends true ? never : A[K] extends F[K] ? never : K extends \"id\" ? A[K] extends string | number ? never : K : K : never }[keyof A];\n",
        );
        ts.push_str(
            "  type $$HogeFallthroughSpreadGuard<C, P, A, __K = $$HogeInvalidFallthroughSpreadKeys<P, $$HogeFallthroughProps<C>, A>> = [__K] extends [never] ? unknown : { readonly __vizeInvalidFallthroughSpreadValues: __K };\n",
        );
        ts.push_str(
            "  type $$HogeFallthroughSpreadChecker<C, P> = <A>(props: A & $$HogeFallthroughSpreadGuard<C, P, NoInfer<A>>) => A;\n",
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
    // correct code (#3446). `$$HogeCallableProp` remains the safe fallback for
    // components without Vize's resolver. A generic Vize child is invoked once
    // through `$$HogePropsResolver`; `$$HogeResolvedProp` then selects the
    // instantiated callback type so return errors surface inside the authored
    // body, at the same leaf byte as vue-tsc. `any` is excluded from the
    // fallback so a genuinely `any` prop stays assignable from a non-function
    // value, and a resolved non-generic prop type is returned untouched.
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_callback_prop))
    {
        ts.push_str(
            "  type $$HogeCallableProp<T> = $$HogeIsAny<T> extends true ? T : unknown extends T ? (...args: any[]) => any : T;\n",
        );
        ts.push_str(
            "  type $$HogePropsResolver<C> = C extends { __vizeResolveProps?: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => {}) : $$HogeIsLibraryGenericExport<C> extends true ? C : (props: any) => {};\n",
        );
        // A vue-tsc library export returns its instantiated context; its props
        // are what the callback is resolved against.
        ts.push_str(
            "  type $$HogeResolvedContextProps = <R>(resolved: R) => R extends { __ctx?: infer __X } ? NonNullable<__X> extends { props: infer __P } ? __P : R : R;\n",
        );
        ts.push_str(
            "  type $$HogePropsSelector<R> = <A extends Partial<R> & Record<string, unknown>>(props: A) => A;\n",
        );
        ts.push_str("  type $$HogeMissingProp = { readonly __vizeMissingProp: unique symbol };\n");
        ts.push_str(
            "  type $$HogeResolvedPropEntry<R, K extends PropertyKey> = R extends unknown ? K extends keyof R ? { value: R[K] } : $$HogeMissingProp : never;\n",
        );
        ts.push_str(
            "  type $$HogeSelectedProps<R, A> = R extends unknown ? A extends Partial<R> ? R : never : never;\n",
        );
        ts.push_str(
            "  type $$HogeResolvedProp<R, A, K extends PropertyKey, F, __S = $$HogeSelectedProps<R, A>, __E = $$HogeResolvedPropEntry<__S, K>, __A = $$HogeResolvedPropEntry<R, K>, __P = Extract<__E, { value: unknown }>> = [__S] extends [never] ? F : [__P] extends [never] ? [Extract<__A, { value: unknown }>] extends [never] ? F : never : __P extends { value: infer V } ? V : never;\n",
        );
    }
    append_ref_callback_helper(ts, usages);
}
