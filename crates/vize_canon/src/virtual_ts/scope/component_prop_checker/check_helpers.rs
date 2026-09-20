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
) {
    ts.push_str("  type __VizeIsAny<T> = 0 extends (1 & T) ? true : false;\n");
    // Inline parameter shapes keep `TS2345` messages close to `vue-tsc` while
    // preserving optionality and contextual typing. The strict tail is used
    // only when Vize knows the generated child's fallthrough surface.
    ts.push_str(
        "  type __VizeEmitListeners<C> = C extends { __vizeEmitProps?: infer __E } ? __VizeIsAny<__E> extends true ? {} : NonNullable<__E> : {};\n",
    );
    ts.push_str(
        "  type __VizeIsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;\n",
    );
    ts.push_str(
        "  type __VizeHasFallthroughProps<C> = __VizeIsGeneratedComponent<C> extends true ? C extends { readonly __vizeHasFallthroughProps: true } ? true : false : false;\n",
    );
    ts.push_str(
        "  type __VizeFallthroughProps<C> = __VizeHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? __VizeIsAny<__F> extends true ? {} : NonNullable<__F> : {} : {};\n",
    );
    // Only a child generated under `checkRequiredFallthroughAttributes` has
    // non-optional fallthrough props; every other child's surface is
    // `Partial<...>` or an open record, whose required keys are `never`.
    //
    // The no-key case resolves to the literal `{}` rather than an empty mapped
    // type, so the intersection it joins still displays as the child's `Props`
    // in diagnostics, the way `vue-tsc` prints it.
    ts.push_str(
        "  type __VizeRequiredFallthroughProps<C, __F = __VizeFallthroughProps<C>, __K = __VizeRequiredKeys<__F>> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n",
    );
    ts.push_str("  type __VizeVueVNodeProps = import('vue').VNodeProps;\n");
    ts.push_str("  type __VizeVueAllowedComponentProps = import('vue').AllowedComponentProps;\n");
    ts.push_str("  type __VizeVueComponentCustomProps = import('vue').ComponentCustomProps;\n");
    ts.push_str(
        "  interface __VizePublicComponentAttrs extends __VizeVueVNodeProps, __VizeVueAllowedComponentProps, __VizeVueComponentCustomProps {}\n",
    );
    if check_unknown_props {
        // Real HTML attributes are valid Vue fallthrough on any component, so
        // the strict surface accepts every attribute some native element
        // declares — `id`, `type`, `accept`, kebab `aria-*` plus camelized
        // spellings, custom `data-*` — while a name no element knows
        // (`depressed`) stays a strict finding (#4966). Derived from the
        // `__VizeNativeElements` program alias; degrades to `{}` on a `vue`
        // without `NativeElements`, like the native prop checks degrade.
        ts.push_str(concat!(
            "  type __VizeAllowedFallthroughAttrs<C> = __VizeHasFallthroughProps<C> extends true ? Record<string, unknown> : {};\n",
            "  type __VizeAttrCamel<S extends string> = S extends `${infer __H}-${infer __T}` ? `${__H}${Capitalize<__VizeAttrCamel<__T>>}` : S;\n",
            "  type __VizeNativeAttrNames = { [K in keyof __VizeNativeElements & string]: keyof __VizeNativeElements[K] }[keyof __VizeNativeElements & string] & string;\n",
            "  type __VizeGlobalHtmlAttrs = __VizeIsAny<__VizeNativeElements> extends true ? {} : { [K in __VizeNativeAttrNames as K | __VizeAttrCamel<K>]?: unknown } & { [K in `data${string}`]?: unknown };\n",
            "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & __VizeGlobalHtmlAttrs & __VizeAllowedFallthroughAttrs<C> : __VizePublicComponentAttrs & __VizeGlobalHtmlAttrs;\n",
        ));
    } else {
        ts.push_str(
            "  type __VizeComponentCheckTail<C> = __VizeIsGeneratedComponent<C> extends true ? __VizePublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;\n",
        );
    }
    ts.push_str(
        "  type __VizeComponentCheckProps<P, T> = { readonly [K in keyof P]: P[K] } & T;\n",
    );
    ts.push_str(
        "  type __VizePublicProps<C> = C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {};\n",
    );
    ts.push_str(
        "  type __VizeInstanceRawProps<C> = C extends { new (): { readonly __vizeRawProps?: infer __P } } ? __VizeIsAny<__P> extends true ? __VizePublicProps<C> : __P : __VizePublicProps<C>;\n",
    );
    ts.push_str(
        "  type __VizeStaticRawProps<C> = C extends { readonly __vizeRawProps?: infer __P } ? __VizeIsAny<__P> extends true ? __VizePublicProps<C> : __P : __VizePublicProps<C>;\n",
    );
    ts.push_str(
        "  type __VizeHasInstanceRawProps<C> = __VizeIsGeneratedComponent<C> extends true ? C extends { new (): { readonly __vizeRawProps?: infer __P } } ? __VizeIsAny<__P> extends true ? false : true : false : false;\n",
    );
    ts.push_str(
        "  type __VizeHasStaticRawProps<C> = __VizeIsGeneratedComponent<C> extends true ? C extends { readonly __vizeRawProps?: infer __P } ? __VizeIsAny<__P> extends true ? false : true : false : false;\n",
    );
    ts.push_str(
        "  type __VizeComponentRawProps<C> = __VizeHasInstanceRawProps<C> extends true ? __VizeInstanceRawProps<C> : __VizeStaticRawProps<C>;\n",
    );
    ts.push_str(
        "  type __VizeHasRawProps<C> = __VizeHasInstanceRawProps<C> extends true ? true : __VizeHasStaticRawProps<C>;\n",
    );
    ts.push_str(
        "  type __VizeFallthroughValue<C, K extends PropertyKey> = __VizeHasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? __VizeIsAny<__F> extends true ? unknown : __VizePropValue<NonNullable<__F>, K, unknown> : unknown : unknown;\n",
    );
    if usages.iter().any(|(_, usage)| {
        usage
            .events
            .iter()
            .any(|event| !event.name_is_dynamic && !event.name.is_empty())
    }) {
        ts.push_str(
            "  type __VizeEventName<K extends string> = K extends `on${infer E}` ? Uncapitalize<E> | __VizeKebabCase<Uncapitalize<E>> : never;\n",
        );
        ts.push_str(
            "  type __VizeKebabEventAliases<E> = { [K in keyof E & string as __VizeKebabCase<K> extends K ? never : __VizeKebabCase<K>]: E[K] };\n",
        );
        ts.push_str(
            "  type __VizeComponentEvents<C> = C extends { __vizeRawEmits?: infer __R; __vizeEventMap?: infer __E } ? [keyof NonNullable<__R>] extends [never] ? NonNullable<__E> : NonNullable<__R> & __VizeKebabEventAliases<NonNullable<__R>> : { [K in keyof (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {}) & string as __VizeEventName<K>]: (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {})[K] };\n",
        );
    }
    ts.push_str(
        "  type __VizePropChecker<C, P, T = __VizeComponentCheckTail<C>> = __VizeIsAny<C> extends true ? (props: { readonly [K in keyof P]: P[K] } & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? __VizeIsAny<__F> extends true ? (props: __VizeComponentCheckProps<P, T>) => void : __F extends (...args: any[]) => any ? __F : (props: __VizeComponentCheckProps<P, T>) => void : (props: __VizeComponentCheckProps<P, T>) => void;\n",
    );
    ts.push_str(
        "  type __VizePropValue<P, K extends PropertyKey, F = unknown, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? F : __V;\n",
    );
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
    append_ref_callback_helper(ts, usages);
}
