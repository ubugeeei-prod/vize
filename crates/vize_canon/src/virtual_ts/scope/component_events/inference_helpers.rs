use vize_carton::String;
use vize_croquis::{Croquis, ScopeData};

/// Resolve emitted payloads from the props written at this call site. Generated
/// components expose their own/public keys separately from inherited root keys;
/// validating each authored key preserves that priority even for index signatures.
/// Unknown keys outside both surfaces remain the canonical prop checker's policy.
/// External constructors retain higher-order inference, and generated business
/// generics retain their existing resolver branches. Vue 2 has no root metadata.
const EMIT_INFERENCE_HELPERS: &str = r#"type $VueEmitRequiredKeys<T> = { [K in keyof T]-?: {} extends Pick<T, K> ? never : K }[keyof T];
type $VueEmitRequiredFields<T> = T extends unknown ? Pick<T, $VueEmitRequiredKeys<T>> : never;
type $VueEmitFiniteFields<T> = { [K in keyof T as {} extends Record<K, unknown> ? never : K]: T[K] };
type $VueEmitFallthrough<F> = $VueIsAny<F> extends true ? {} : [NonNullable<F>] extends [never] ? {} : NonNullable<F>;
type $VueEmitOwnedKeys<C> = $VueComponentOwnKeys<C extends { readonly __vizeRawProps?: infer P } ? P : {}, C extends { readonly __vizeEmitProps?: infer E } ? E : {}>;
type $VueEmitInput<P, F, O extends PropertyKey, A> = P extends unknown ? F extends unknown ? { [K in keyof A]: K extends O ? K extends keyof P ? P[K] : never : K extends keyof F ? F[K] : K extends keyof P ? P[K] : unknown } & $VueEmitRequiredFields<P> & $VueEmitRequiredFields<Omit<$VueEmitFiniteFields<F>, O>> : never : never;
declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => __EmitProps<E>;
type $VueConstructorEmitPropsFactory<F, O extends PropertyKey> = <P, E>(component: new (props: P, ...args: any[]) => { $emit: E }) => <A>(props: $VueEmitInput<P, $VueEmitFallthrough<F>, O, A>) => __EmitProps<E>;
type $VueNativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? C extends { readonly __vizeComponentMarker: true } ? $VueConstructorEmitPropsFactory<C extends { readonly __vizeFallthroughProps?: infer F } ? F : {}, $VueEmitOwnedKeys<C>> : typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
type $VueEmitPropsFactory<C> = $VueIsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $VueNativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $VueNativeEmitPropsFactory<C> : $VueNativeEmitPropsFactory<C>;
"#;

const LEGACY_EMIT_INFERENCE_HELPERS: &str = r#"declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => __EmitProps<E>;
type $VueNativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
type $VueEmitPropsFactory<C> = $VueIsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $VueNativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $VueNativeEmitPropsFactory<C> : $VueNativeEmitPropsFactory<C>;
"#;

/// `forwards_roots`: a generic component instantiates the listeners its root
/// forwards the same way, whether or not its own template listens to anything.
pub(crate) fn emit_event_inference_helpers(
    ts: &mut String,
    summary: &Croquis,
    forwards_roots: bool,
    legacy_vue2: bool,
) -> bool {
    let needed = forwards_roots
        || summary.scopes.iter().any(|scope| {
            matches!(scope.data(), ScopeData::EventHandler(data) if data.target_component.is_some())
        });
    if needed {
        ts.push_str(if legacy_vue2 {
            LEGACY_EMIT_INFERENCE_HELPERS
        } else {
            EMIT_INFERENCE_HELPERS
        });
    }
    needed
}
