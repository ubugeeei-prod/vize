use vize_carton::String;
use vize_croquis::{Croquis, ScopeData};

/// Resolve emitted payloads from the props written at this call site. Generated
/// components expose their own/public keys separately from inherited root keys;
/// validating each authored key preserves that priority even for index signatures.
/// Unknown keys outside both surfaces remain the canonical prop checker's policy.
/// External constructors retain higher-order inference, and generated business
/// generics retain their existing resolver branches. Vue 2 has no root metadata.
const EMIT_INFERENCE_HELPERS: &str =
    include_str!("../../helpers/declarations/emit-inference.d.ts.txt");

const LEGACY_EMIT_INFERENCE_HELPERS: &str = r#"declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => __EmitProps<E>;
type __VizeNativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
type __VizeEmitPropsFactory<C> = __VizeIsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : __VizeNativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : __VizeNativeEmitPropsFactory<C> : __VizeNativeEmitPropsFactory<C>;
"#;

/// `forwards_roots`: a generic component instantiates the listeners its root
/// forwards the same way, whether or not its own template listens to anything.
pub(crate) fn emit_event_inference_helpers(
    ts: &mut String,
    summary: &Croquis,
    forwards_roots: bool,
    legacy_vue2: bool,
    hoist_shared_preamble: bool,
) -> bool {
    let needed = forwards_roots
        || summary.scopes.iter().any(|scope| {
            matches!(scope.data(), ScopeData::EventHandler(data) if data.target_component.is_some())
        });
    if needed && (!hoist_shared_preamble || legacy_vue2) {
        ts.push_str(if legacy_vue2 {
            LEGACY_EMIT_INFERENCE_HELPERS
        } else {
            EMIT_INFERENCE_HELPERS
        });
    }
    needed
}
