use vize_carton::String;
use vize_croquis::{Croquis, ScopeData};

/// Higher-order inference keeps a constructor's type parameters until the
/// authored props instantiate its emit tuple, as the slot factory does.
const EMIT_INFERENCE_HELPERS: &str = r#"declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => __EmitProps<E>;
type __VizeNativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
type __VizeEmitPropsFactory<C> = __VizeIsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : __VizeNativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : __VizeNativeEmitPropsFactory<C> : __VizeNativeEmitPropsFactory<C>;
"#;

/// `forwards_roots`: a generic component instantiates the listeners its root
/// forwards the same way, whether or not its own template listens to anything.
pub(crate) fn emit_event_inference_helpers(
    ts: &mut String,
    summary: &Croquis,
    forwards_roots: bool,
) -> bool {
    let needed = forwards_roots
        || summary.scopes.iter().any(|scope| {
            matches!(scope.data(), ScopeData::EventHandler(data) if data.target_component.is_some())
        });
    if needed {
        ts.push_str(EMIT_INFERENCE_HELPERS);
    }
    needed
}
