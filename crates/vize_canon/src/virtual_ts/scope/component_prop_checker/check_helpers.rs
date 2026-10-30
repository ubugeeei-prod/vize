//! The shared type helpers every per-usage prop check resolves through.

use vize_carton::String;
use vize_croquis::croquis::ComponentUsage;

use super::super::component_ref_props::append_ref_callback_helper;
use super::is_inline_callback_prop;
use crate::virtual_ts::helpers::is_component_aria_prop_name;

/// Emit only the policy aliases whose shape depends on project options. Every
/// other component-check helper is ambient in batch projects, so TypeScript can
/// reuse one declaration identity and its instantiation cache across all SFCs.
fn append_hoisted_component_check_policy(
    ts: &mut String,
    check_unknown_props: bool,
    check_unknown_fallthrough_props: bool,
    strict_component_attrs: bool,
) {
    if check_unknown_props {
        if strict_component_attrs {
            ts.push_str(
                "  type $AllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof import(\"vize:virtual-ts\").$FallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $PublicComponentAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n",
            );
            ts.push_str(
                "  type $ComponentCheckTail<C> = import(\"vize:virtual-ts\").$IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & $AllowedFallthroughAttrs<C> : $PublicComponentAttrs;\n",
            );
        } else {
            ts.push_str(if check_unknown_fallthrough_props {
                "  type $AllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof import(\"vize:virtual-ts\").$FallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $PublicComponentAttrs | keyof import(\"vize:virtual-ts\").ComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            } else {
                "  type $AllowedFallthroughAttrs<C> = import(\"vize:virtual-ts\").$HasFallthroughProps<C> extends true ? Record<string, unknown> : {};\n"
            });
            ts.push_str(
                "  type $ComponentCheckTail<C> = import(\"vize:virtual-ts\").$IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & import(\"vize:virtual-ts\").ComponentGlobalHtmlAttrs & $AllowedFallthroughAttrs<C> : $PublicComponentAttrs & import(\"vize:virtual-ts\").ComponentGlobalHtmlAttrs;\n",
            );
        }
    } else {
        ts.push_str(
            "  type $ComponentCheckTail<C> = import(\"vize:virtual-ts\").$IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;\n",
        );
    }
}

/// Vue public props are package-relative types. Keep these aliases in each
/// generated module so workspace packages resolve their own Vue installation
/// and `ComponentCustomProps` augmentation instead of the batch helper's.
fn append_vue_public_component_attrs(ts: &mut String) {
    ts.push_str("  type $VueVNodeProps = import('vue').VNodeProps;\n");
    ts.push_str("  type $VueAllowedComponentProps = import('vue').AllowedComponentProps;\n");
    ts.push_str("  type $VueComponentCustomProps = import('vue').ComponentCustomProps;\n");
    ts.push_str(
        "  interface $PublicComponentAttrs extends $VueVNodeProps, $VueAllowedComponentProps, $VueComponentCustomProps {}\n",
    );
}

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
    hoist_shared_preamble: bool,
) {
    let needs_aria_prop_value = usages.iter().any(|(_, usage)| {
        usage
            .props
            .iter()
            .any(|prop| !prop.name_is_dynamic && is_component_aria_prop_name(prop.name.as_str()))
    });
    // `PropertyKey` is a capturable authored alias. The keyword constraint
    // keeps literal raw/camel keys intact even when the SFC shadows that name.
    if needs_aria_prop_value && hoist_shared_preamble {
        // Vue camelizes `aria-label` onto a declared `ariaLabel` prop, while
        // vue-tsc's default `htmlAttributes: ['aria-*']` lets an optional
        // `string | undefined` value pass under exactOptionalPropertyTypes.
        // The dedicated per-prop assignment owns the real value check. Keep
        // the narrowing conditional inline: in hoisted batch programs an
        // authored module alias named `$NotUndefined` is otherwise the
        // nearest lexical declaration and can capture this generated helper.
        ts.push_str(
            "  function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>(raw: R, camel: C, value: T): { readonly [K in R | C]: T extends undefined ? never : T } { return { [raw]: value, [camel]: value } as unknown as { readonly [K in R | C]: T extends undefined ? never : T }; }\n",
        );
    }
    if hoist_shared_preamble {
        append_vue_public_component_attrs(ts);
        append_hoisted_component_check_policy(
            ts,
            check_unknown_props,
            check_unknown_fallthrough_props,
            strict_component_attrs,
        );
        return;
    }
    ts.push_str("  type $IsAny<T> = 0 extends (1 & T) ? true : false;\n");
    if needs_aria_prop_value {
        // Vue camelizes `aria-label` onto a declared `ariaLabel` prop, while
        // vue-tsc's default `htmlAttributes: ['aria-*']` lets an optional
        // `string | undefined` value pass under exactOptionalPropertyTypes.
        // The dedicated per-prop assignment owns the real value check. This
        // whole-object witness therefore removes only `undefined`: required
        // props remain present, generic inference retains the authored value
        // type, and wrong values are still diagnosed once (#11048, #9658).
        ts.push_str(
            "  function __vizeComponentAriaPropEntries<const R extends keyof any, const C extends keyof any, const T>(raw: R, camel: C, value: T): { readonly [K in R | C]: $NotUndefined<T> } { return { [raw]: value, [camel]: value } as unknown as { readonly [K in R | C]: $NotUndefined<T> }; }\n",
        );
    }
    ts.push_str(
        "  type $ComponentNavigationProps<P> = $IsAny<P> extends true ? Record<string, unknown> : unknown extends P ? {} : P;\n",
    );
    // Inline parameter shapes keep `TS2345` messages close to `vue-tsc` while
    // preserving optionality and contextual typing. The strict tail is used
    // only when Vize knows the generated child's fallthrough surface.
    ts.push_str(
        "  type $EmitListeners<C> = C extends { __vizeEmitProps?: infer __E } ? $IsAny<__E> extends true ? {} : NonNullable<__E> : {};\n",
    );
    ts.push_str(
        "  type $IsGeneratedComponent<C> = C extends { readonly __vizeComponentMarker: true } ? true : false;\n",
    );
    ts.push_str(
        "  type $HasFallthroughProps<C> = $IsGeneratedComponent<C> extends true ? C extends { readonly __vizeHasFallthroughProps: true } ? true : false : false;\n",
    );
    ts.push_str(
        "  type $FallthroughProps<C> = $HasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $IsAny<__F> extends true ? {} : NonNullable<__F> : {} : {};\n",
    );
    // Only a child generated under `checkRequiredFallthroughAttributes` has
    // non-optional fallthrough props; every other child's surface is
    // `Partial<...>` or an open record, whose required keys are `never`.
    //
    // The no-key case resolves to the literal `{}` rather than an empty mapped
    // type, so the intersection it joins still displays as the child's `Props`
    // in diagnostics, the way `vue-tsc` prints it.
    ts.push_str(
        "  type $RequiredFallthroughProps<C, __F = $FallthroughProps<C>, __K = $RequiredKeys<__F>> = [__K] extends [never] ? {} : { [K in keyof __F as K extends __K ? K : never]: __F[K] };\n",
    );
    append_vue_public_component_attrs(ts);
    if check_unknown_props {
        if strict_component_attrs {
            // Standard strict attrs accept Vue public attrs and only keys the
            // child's enabled fallthrough producer actually declares.
            ts.push_str(
                "  type $AllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $FallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $PublicComponentAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            );
            ts.push_str(
                "  type $ComponentCheckTail<C> = $IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & $AllowedFallthroughAttrs<C> : $PublicComponentAttrs;\n"
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
                "  type $AllowedFallthroughAttrs<C, __K = Exclude<keyof { [K in keyof $FallthroughProps<C> as string extends K ? never : K]: unknown }, keyof $PublicComponentAttrs | keyof $ComponentGlobalHtmlAttrs>> = [__K] extends [never] ? {} : { [K in __K & PropertyKey]?: unknown };\n"
            } else {
                "  type $AllowedFallthroughAttrs<C> = $HasFallthroughProps<C> extends true ? Record<string, unknown> : {};\n"
            });
            ts.push_str(
                "  type $ComponentCheckTail<C> = $IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & $ComponentGlobalHtmlAttrs & $AllowedFallthroughAttrs<C> : $PublicComponentAttrs & $ComponentGlobalHtmlAttrs;\n",
            );
        }
    } else {
        ts.push_str(
            "  type $ComponentCheckTail<C> = $IsGeneratedComponent<C> extends true ? $PublicComponentAttrs & Record<string, unknown> : Record<string, unknown>;\n",
        );
    }
    ts.push_str("  type $ComponentCheckProps<P, T> = { readonly [K in keyof P]: P[K] } & T;\n");
    // Abstract component constructors carry the same instance props as concrete
    // constructors. Only an authored `any` instance has no prop contract; keep
    // typed instances closed rather than opening every constructor's attrs.
    ts.push_str(
        "  type $PublicProps<C> = C extends abstract new (...args: any[]) => infer __I ? $IsAny<__I> extends true ? Record<string, unknown> : __I extends { $props: infer __P } ? __P : {} : C extends (props: infer __P, ...args: any[]) => any ? __P : {};\n",
    );
    ts.push_str(
        "  type $InstanceRawProps<C> = C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $IsAny<__P> extends true ? $PublicProps<C> : __P : $PublicProps<C>;\n",
    );
    ts.push_str(
        "  type $StaticRawProps<C> = C extends { readonly __vizeRawProps?: infer __P } ? $IsAny<__P> extends true ? $PublicProps<C> : __P : $PublicProps<C>;\n",
    );
    ts.push_str(
        "  type $HasInstanceRawProps<C> = $IsGeneratedComponent<C> extends true ? C extends { new (): { readonly __vizeRawProps?: infer __P } } ? $IsAny<__P> extends true ? false : true : false : false;\n",
    );
    ts.push_str(
        "  type $HasStaticRawProps<C> = $IsGeneratedComponent<C> extends true ? C extends { readonly __vizeRawProps?: infer __P } ? $IsAny<__P> extends true ? false : true : false : false;\n",
    );
    // An empty options object matches an optional-only shape and infers
    // `unknown`. Trust raw metadata only when its generated identity is proven,
    // otherwise use the public component contract (including the empty one).
    ts.push_str(
        "  type $ComponentRawProps<C, __P = $HasInstanceRawProps<C> extends true ? $InstanceRawProps<C> : $HasStaticRawProps<C> extends true ? $StaticRawProps<C> : $PublicProps<C>> = unknown extends __P ? Record<string, unknown> : __P;\n",
    );
    ts.push_str(
        "  type $HasRawProps<C> = $HasInstanceRawProps<C> extends true ? true : $HasStaticRawProps<C>;\n",
    );
    ts.push_str(
        "  type $FallthroughValue<C, K extends PropertyKey> = $HasFallthroughProps<C> extends true ? C extends { readonly __vizeFallthroughProps?: infer __F } ? $IsAny<__F> extends true ? unknown : $PropValue<NonNullable<__F>, K, unknown> : unknown : unknown;\n",
    );
    if usages.iter().any(|(_, usage)| {
        usage
            .events
            .iter()
            .any(|event| !event.name_is_dynamic && !event.name.is_empty())
    }) {
        ts.push_str(
            "  type $EventName<K extends string> = K extends `on${infer E}` ? Uncapitalize<E> | $KebabCase<Uncapitalize<E>> : never;\n",
        );
        ts.push_str(
            "  type $KebabEventAliases<E> = { [K in keyof E & string as $KebabCase<K> extends K ? never : $KebabCase<K>]: E[K] };\n",
        );
        ts.push_str(
            "  type $ComponentEvents<C> = C extends { __vizeRawEmits?: infer __R; __vizeEventMap?: infer __E } ? [keyof NonNullable<__R>] extends [never] ? NonNullable<__E> : NonNullable<__R> & $KebabEventAliases<NonNullable<__R>> : { [K in keyof (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {}) & string as $EventName<K>]: (C extends { new (...args: any[]): { $props: infer __P } } ? __P : C extends (props: infer __P, ...args: any[]) => any ? __P : {})[K] };\n",
        );
    }
    // A generic SFC compiled by vue-tsc (a library such as reka-ui) is a
    // function returning `VNode & { __ctx?: ... }`. Extracting its props
    // instantiates the generics with their constraints, so it is called
    // directly instead, as vue-tsc does, and TypeScript infers them.
    ts.push_str(
        "  type $IsLibraryGenericExport<C> = C extends { __vizeCheck: any } | { readonly __vizeComponentMarker: true } ? false : C extends (...args: any[]) => infer __R ? 0 extends (1 & __R) ? false : '__ctx' extends keyof __R ? true : false : false;\n",
    );
    ts.push_str(
        "  type $PropChecker<C, P, T = $ComponentCheckTail<C>> = $IsAny<C> extends true ? (props: { readonly [K in keyof P]: P[K] } & Record<string, unknown>) => void : C extends { __vizeCheck: infer __F } ? $IsAny<__F> extends true ? (props: $ComponentCheckProps<P, T>) => void : unknown extends __F ? (props: $ComponentCheckProps<P, T>) => void : __F extends (...args: any[]) => any ? __F : (props: $ComponentCheckProps<P, T>) => void : $IsLibraryGenericExport<C> extends true ? C : (props: $ComponentCheckProps<P, T>) => void;\n",
    );
    ts.push_str(
        "  type $PropValue<P, K extends PropertyKey, F = unknown, __V = P extends unknown ? (K extends keyof P ? P[K] : never) : never> = [__V] extends [never] ? F : __V;\n",
    );
    if typed_fallthrough_values
        && usages
            .iter()
            .any(|(_, usage)| !usage.spread_props.is_empty())
    {
        ts.push_str(
            "  type $InvalidFallthroughSpreadKeys<P, F, A> = { [K in keyof A]-?: K extends keyof P | keyof $KebabProps<P> ? never : K extends keyof F ? $IsAny<A[K]> extends true ? never : A[K] extends F[K] ? never : K extends \"id\" ? A[K] extends string | number ? never : K : K : never }[keyof A];\n",
        );
        ts.push_str(
            "  type $FallthroughSpreadGuard<C, P, A, __K = $InvalidFallthroughSpreadKeys<P, $FallthroughProps<C>, A>> = [__K] extends [never] ? unknown : { readonly __vizeInvalidFallthroughSpreadValues: __K };\n",
        );
        ts.push_str(
            "  type $FallthroughSpreadChecker<C, P> = <A>(props: A & $FallthroughSpreadGuard<C, P, NoInfer<A>>) => A;\n",
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
    // correct code (#3446). `$CallableProp` remains the safe fallback for
    // components without Vize's resolver. A generic Vize child is invoked once
    // through `$PropsResolver`; `$ResolvedProp` then selects the
    // instantiated callback type so return errors surface inside the authored
    // body, at the same leaf byte as vue-tsc. `any` is excluded from the
    // fallback so a genuinely `any` prop stays assignable from a non-function
    // value, and a resolved non-generic prop type is returned untouched.
    if usages
        .iter()
        .any(|(_, usage)| usage.props.iter().any(is_inline_callback_prop))
    {
        ts.push_str(
            "  type $CallableProp<T> = $IsAny<T> extends true ? T : unknown extends T ? (...args: any[]) => any : T;\n",
        );
        ts.push_str(
            "  type $PropsResolver<C> = C extends { __vizeResolveProps?: infer __F } ? (__F extends (...args: any[]) => any ? __F : (props: any) => {}) : $IsLibraryGenericExport<C> extends true ? C : (props: any) => {};\n",
        );
        // A vue-tsc library export returns its instantiated context; its props
        // are what the callback is resolved against.
        ts.push_str(
            "  type $ResolvedContextProps = <R>(resolved: R) => R extends { __ctx?: infer __X } ? NonNullable<__X> extends { props: infer __P } ? __P : R : R;\n",
        );
        ts.push_str(
            "  type $PropsSelector<R> = <A extends Partial<R> & Record<string, unknown>>(props: A) => A;\n",
        );
        ts.push_str("  type $MissingProp = { readonly __vizeMissingProp: unique symbol };\n");
        ts.push_str(
            "  type $ResolvedPropEntry<R, K extends PropertyKey> = R extends unknown ? K extends keyof R ? { value: R[K] } : $MissingProp : never;\n",
        );
        ts.push_str(
            "  type $SelectedProps<R, A> = R extends unknown ? A extends Partial<R> ? R : never : never;\n",
        );
        ts.push_str(
            "  type $ResolvedProp<R, A, K extends PropertyKey, F, __S = $SelectedProps<R, A>, __E = $ResolvedPropEntry<__S, K>, __A = $ResolvedPropEntry<R, K>, __P = Extract<__E, { value: unknown }>> = [__S] extends [never] ? F : [__P] extends [never] ? [Extract<__A, { value: unknown }>] extends [never] ? F : never : __P extends { value: infer V } ? V : never;\n",
        );
    }
    append_ref_callback_helper(ts, usages);
}
