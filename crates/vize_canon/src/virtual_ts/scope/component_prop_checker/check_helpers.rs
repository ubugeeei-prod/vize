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
    ts.push_str("  type $VueIsAny<T> = 0 extends (1 & T) ? true : false;\n");
    ts.push_str(
        "  type $VueComponentNavigationProps<P> = $VueIsAny<P> extends true ? Record<string, unknown> : unknown extends P ? {} : P;\n",
    );
    // Inline parameter shapes keep `TS2345` messages close to `vue-tsc` while
    // preserving optionality and contextual typing. The strict tail is used
    // only when Vize knows the generated child's fallthrough surface.
    ts.push_str(
        "  type $VueEmitListeners<C> = C extends { __vizeEmitProps?: infer __E } ? $VueIsAny<__E> extends true ? {} : NonNullable<__E> : {};\n",
    );
    ts.push_str(
        "  type $VueIsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;\n",
    );
    ts.push_str(
        "  type $VueHasFallthroughProps<C> = $VueIsGeneratedComponent<C> extends true ? C extends { readonly __vizeHasFallthroughProps: true } ? true : false : false;\n",
    );
    ts.push_str(
        "  type $VueFallthroughProps<C> = $VueHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $VueIsAny<__F> extends true ? {} : NonNullable<__F> : {} : {};\n",
    );
    // Only a child generated under `checkRequiredFallthroughAttributes` has
    // non-optional fallthrough props; every other child's surface is
    // `Partial<...>` or an open record, whose required keys are `never`.
    //
    // The no-key case resolves to the literal `{}` rather than an empty mapped
    // type, so the intersection it joins still displays as the child's `Props`
    // in diagnostics, the way `vue-tsc` prints it.
    ts.push_str(
        "  type $VueRequiredFallthroughProps<C, __F = $VueFallthroughProps<C>, __K = $VueRequiredKeys<__F>> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n",
    );
    ts.push_str("  type $VueVueVNodeProps = import('vue').VNodeProps;\n");
    ts.push_str("  type $VueVueAllowedComponentProps = import('vue').AllowedComponentProps;\n");
    ts.push_str("  type $VueVueComponentCustomProps = import('vue').ComponentCustomProps;\n");
    ts.push_str(
        "  interface $VuePublicComponentAttrs extends $VueVueVNodeProps, $VueVueAllowedComponentProps, $VueVueComponentCustomProps {}\n",
    );
    if check_unknown_props {
        if strict_component_attrs {
            // Standard strict attrs accept Vue public attrs and only keys the
            // child's enabled fallthrough producer actually declares.
            ts.push_str(
                "  type $VueAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $VueFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $VuePublicComponentAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            );
            ts.push_str(
                "  type $VueComponentCheckTail<C> = $VueIsGeneratedComponent<C> extends true ? $VuePublicComponentAttrs & $VueAllowedFallthroughAttrs<C> : $VuePublicComponentAttrs;\n"
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
                "  type $VueAllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $VueFallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $VuePublicComponentAttrs | keyof $VueComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            } else {
                "  type $VueAllowedFallthroughAttrs<C> = $VueHasFallthroughProps<C> extends true ? Record<string, unknown> : {};\n"
            });
            ts.push_str(
                "  type $VueComponentCheckTail<C> = $VueIsGeneratedComponent<C> extends true ? $VuePublicComponentAttrs & $VueComponentGlobalHtmlAttrs & $VueAllowedFallthroughAttrs<C> : $VuePublicComponentAttrs & $VueComponentGlobalHtmlAttrs;\n",
            );
        }
    } else {
        ts.push_str(
            "  type $VueComponentCheckTail<C> = $VueIsGeneratedComponent<C> extends true ? $VuePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;\n",
        );
    }
    ts.push_str(
        "  type $VueComponentCheckProps<P, T> = { readonly [K in keyof P]: P[K] } & T;\n",
    );
    // Abstract component constructors carry the same instance props as concrete
    // constructors. Only an authored `any` instance has no prop contract; keep
    // typed instances closed rather than opening every constructor's attrs.
    ts.push_str(
        "  type $VuePublicProps<C> = C extends abstract new (...args: any[]) => infer __I ? $VueIsAny<__I> extends true ? Record<string, unknown> : __I extends { $props: infer __P } ? __P : {} : C extends (props: infer __P, ...args: any[]) => any ? __P : {};\n",
    );
    ts.push_str(
        "  type $VueInstanceRawProps<C> = C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $VueIsAny<__P> extends true ? $VuePublicProps<C> : __P : $VuePublicProps<C>;\n",
    );
    ts.push_str(
        "  type $VueStaticRawProps<C> = C extends { readonly __vizeRawProps?: infer __P } ? $VueIsAny<__P> extends true ? $VuePublicProps<C> : __P : $VuePublicProps<C>;\n",
    );
    ts.push_str(
        "  type $VueHasInstanceRawProps<C> = $VueIsGeneratedComponent<C> extends true ? C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $VueIsAny<__P> extends true ? false : true : false : false;\n",
    );
    ts.push_str(
        "  type $VueHasStaticRawProps<C> = $VueIsGeneratedComponent<C> extends true ? C extends { readonly __vizeRawProps?: infer __P } ? $VueIsAny<__P> extends true ? false : true : false : false;\n",
    );
    // An empty options object matches an optional-only shape and infers
    // `unknown`. Trust raw metadata only when its generated identity is proven,
    // otherwise use the public component contract (including the empty one).
    ts.push_str(
        "  type $VueComponentRawProps<C, __P = $VueHasInstanceRawProps<C> extends true ? $VueInstanceRawProps<C> : $VueHasStaticRawProps<C> extends true ? $VueStaticRawProps<C> : $VuePublicProps<C>> = unknown extends __P ? Record<string, unknown> : __P;\n",
    );
    ts.push_str(
        "  type $VueHasRawProps<C> = $VueHasInstanceRawProps<C> extends true ? true : $VueHasStaticRawProps<C>;\n",
    );
    ts.push_str(
        "  type $VueFallthroughValue<C, K extends PropertyKey> = $VueHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $VueIsAny<__F> extends true ? unknown : $VuePropValue<NonNullable<__F>, K, unknown> : unknown : unknown;\n",
    );
    if usages.iter().any(|(_, usage)| {
        usage
            .events
            .iter()
            .any(|event| !event.name_is_dynamic && !event.name.is_empty())
    }) {
        ts.push_str(
            "  type $VueEventName<K extends string> = K extends `on${infer E}` ? Uncapitalize<E> | $VueKebabCase<Uncapitalize<E>> : never;\n",
        );
        ts.push_str(
            "  type $VueKebabEventAliases<E> = { [K in keyof E & string as $VueKebabCase<K> extends K ? never : $VueKebabCase<K>]: E[K] };\n",
        );
        ts.push_str(
            "  type $VueComponentEvents<C> = C extends { __vizeRawEmits?: infer __R; __vizeEventMap?: infer __E } ? [keyof NonNullable<__R>] extends [never] ? NonNullable<__E> : NonNullable<__R> & $VueKebabEventAliases<NonNullable<__R>> : { [K in keyof (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {}) & string as $VueEventName<K>]: (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {})[K] };\n",
        );
    }
    // A generic SFC compiled by vue-tsc (a library such as reka-ui) is a
    // function returning `VNode & { __ctx?: ... }`. Extracting its props
    // instantiates the generics with their constraints, so it is called
    // directly instead, as vue-tsc does, and TypeScript infers them.
    ts.push_str(
        "  type $VueIsLibraryGenericExport<C> = C extends { __vizeCheck: any } | { readonly __vizeComponentMarker: true } ? false : C extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
    );
    ts.push_str(
        "  type $VuePropChecker<C, P, T = $VueComponentCheckTail<C>> = $VueIsAny<C> extends true ? (props: { readonly [K in keyof P]: P[K] } & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? $VueIsAny<__F> extends true ? (props: $VueComponentCheckProps<P, T>) => void : unknown extends __F ? (props: $VueComponentCheckProps<P, T>) => void : __F extends (...args: any[]) => any ? __F : (props: $VueComponentCheckProps<P, T>) => void : $VueIsLibraryGenericExport<C> extends true ? C : (props: $VueComponentCheckProps<P, T>) => void;\n",
    );
    ts.push_str(
        "  type $VuePropValue<P, K extends PropertyKey, F = unknown, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? F : __V;\n",
    );
    if typed_fallthrough_values
        && usages
            .iter()
            .any(|(_, usage)| !usage.spread_props.is_empty())
    {
        ts.push_str(
            "  type $VueInvalidFallthroughSpreadKeys<P, F, A> = { [K in keyof A]-?: K extends keyof P | keyof $VueKebabProps<P> ? never : K extends keyof F ? $VueIsAny<A[K]> extends true ? never : A[K] extends F[K] ? never : K extends \"id\" ? A[K] extends string | number ? never : K : K : never }[keyof A];\n",
        );
        ts.push_str(
            "  type $VueFallthroughSpreadGuard<C, P, A, __K = $VueInvalidFallthroughSpreadKeys<P, $VueFallthroughProps<C>, A>> = [__K] extends [never] ? unknown : { readonly __vizeInvalidFallthroughSpreadValues: __K };\n",
        );
        ts.push_str(
            "  type $VueFallthroughSpreadChecker<C, P> = <A>(props: A & $VueFallthroughSpreadGuard<C, P, NoInfer<A>>) => A;\n",
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
    // correct code (#3446). `$VueCallableProp` remains the safe fallback for
    // components without Vize's resolver. A generic Vize child is invoked once
    // through `$VuePropsResolver`; `$VueResolvedProp` then selects the
    // instantiated callback type so return errors surface inside the authored
    // body, at the same leaf byte as vue-tsc. `any` is excluded from the
    // fallback so a genuinely `any` prop stays assignable from a non-function
    // value, and a resolved non-generic prop type is returned untouched.
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_callback_prop))
    {
        ts.push_str(
            "  type $VueCallableProp<T> = $VueIsAny<T> extends true ? T : unknown extends T ? (...args: any[]) => any : T;\n",
        );
        ts.push_str(
            "  type $VuePropsResolver<C> = C extends { __vizeResolveProps?: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => {}) : $VueIsLibraryGenericExport<C> extends true ? C : (props: any) => {};\n",
        );
        // A vue-tsc library export returns its instantiated context; its props
        // are what the callback is resolved against.
        ts.push_str(
            "  type $VueResolvedContextProps = <R>(resolved: R) => R extends { __ctx?: infer __X } ? NonNullable<__X> extends { props: infer __P } ? __P : R : R;\n",
        );
        ts.push_str(
            "  type $VuePropsSelector<R> = <A extends Partial<R> & Record<string, unknown>>(props: A) => A;\n",
        );
        ts.push_str("  type $VueMissingProp = { readonly __vizeMissingProp: unique symbol };\n");
        ts.push_str(
            "  type $VueResolvedPropEntry<R, K extends PropertyKey> = R extends unknown ? K extends keyof R ? { value: R[K] } : $VueMissingProp : never;\n",
        );
        ts.push_str(
            "  type $VueSelectedProps<R, A> = R extends unknown ? A extends Partial<R> ? R : never : never;\n",
        );
        ts.push_str(
            "  type $VueResolvedProp<R, A, K extends PropertyKey, F, __S = $VueSelectedProps<R, A>, __E = $VueResolvedPropEntry<__S, K>, __A = $VueResolvedPropEntry<R, K>, __P = Extract<__E, { value: unknown }>> = [__S] extends [never] ? F : [__P] extends [never] ? [Extract<__A, { value: unknown }>] extends [never] ? F : never : __P extends { value: infer V } ? V : never;\n",
        );
    }
    append_ref_callback_helper(ts, usages);
}
