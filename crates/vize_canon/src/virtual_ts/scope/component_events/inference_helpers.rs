use vize_carton::String;
use vize_croquis::{Croquis, ScopeData};

use crate::virtual_ts::helpers::shared_types::EMIT_PROPS;

/// Resolve emitted payloads from the props written at this call site. Generated
/// components expose their own/public keys separately from inherited root keys;
/// validating each authored key preserves that priority even for index signatures.
/// Unknown keys outside both surfaces remain the canonical prop checker's policy.
/// External constructors retain higher-order inference, and generated business
/// generics retain their existing resolver branches. Vue 2 has no root metadata.
const EMIT_INFERENCE_HELPERS: &str =
    include_str!("../../helpers/declarations/emit-inference.d.ts.txt");

const LEGACY_EMIT_INFERENCE_HELPERS_PREFIX: &str = r#"declare function __vizeConstructorEmitProps<P, E>(component: new (props: P, ...args: any[]) => { $emit: E }): (props: P) => "#;
const LEGACY_EMIT_INFERENCE_HELPERS_SUFFIX: &str = r#"<E>;
type $NativeEmitPropsFactory<C> = C extends new (...args: any[]) => { $emit: any } ? typeof __vizeConstructorEmitProps : (component: C) => (props: any) => {};
type $EmitPropsFactory<C> = $IsAny<C> extends true ? (component: C) => (props: any) => {} : C extends { __vizeResolveEmitProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : C extends { __vizeResolveProps?: infer F } ? F extends (...args: any[]) => any ? (component: C) => F : $NativeEmitPropsFactory<C> : $NativeEmitPropsFactory<C>;
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
        if legacy_vue2 {
            ts.push_str(LEGACY_EMIT_INFERENCE_HELPERS_PREFIX);
            ts.push_str(EMIT_PROPS.select(hoist_shared_preamble));
            ts.push_str(LEGACY_EMIT_INFERENCE_HELPERS_SUFFIX);
        } else {
            crate::virtual_ts::helpers::hygiene::emit_inference_helpers(
                ts,
                summary,
                EMIT_INFERENCE_HELPERS,
            );
        }
    }
    needed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legacy_emit_props_identity_follows_helper_storage() {
        for (hoisted, expected, rejected) in [
            (
                false,
                ") => __EmitProps<E>;",
                ") => import(\"vize:virtual-ts\").__EmitProps<E>;",
            ),
            (
                true,
                ") => import(\"vize:virtual-ts\").__EmitProps<E>;",
                ") => __EmitProps<E>;",
            ),
        ] {
            let mut output = String::default();
            assert!(emit_event_inference_helpers(
                &mut output,
                &Croquis::default(),
                true,
                true,
                hoisted,
            ));
            assert_eq!(output.matches(expected).count(), 1, "{output}");
            assert_eq!(output.matches(rejected).count(), 0, "{output}");
        }
    }
}
