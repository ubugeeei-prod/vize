//! `<component>` resolution, pinned against `@vue/compiler-vapor` 3.6.0-rc.6.

#![expect(clippy::disallowed_types, reason = "test fixtures compare std strings")]

use super::{VaporCompilerOptions, compile_vapor};
use vize_carton::Allocator;

fn compiled(source: &str, retained: bool) -> std::string::String {
    let allocator = Allocator::new();
    let result = compile_vapor(
        &allocator,
        source,
        VaporCompilerOptions {
            davinci_retained_lane: retained,
            ..Default::default()
        },
    );
    assert!(
        result.error_messages.is_empty(),
        "{:?}",
        result.error_messages
    );
    std::string::String::from(result.code.as_str())
}

#[test]
fn a_static_is_resolves_the_component_once() {
    // Upstream: `_createComponentWithFallback(_resolveDynamicComponent("a"), …)`
    // with `is` kept out of the props; only a bound `:is` stays dynamic.
    for retained in [false, true] {
        let code = compiled(
            r#"<component is="a" :id="x" v-bind="attrs"><slot /></component>"#,
            retained,
        );
        assert!(
            code.contains(r#"_createComponentWithFallback(_resolveDynamicComponent("a"), "#),
            "{code}"
        );
        assert!(!code.contains("createDynamicComponent"), "{code}");
        assert!(!code.contains("is: "), "{code}");

        let code = compiled(r#"<component :is="view" :id="x" />"#, retained);
        assert!(
            code.contains("_createDynamicComponent(() => (_ctx.view), "),
            "{code}"
        );

        let code = compiled(r#"<component is="a" :is="view" />"#, retained);
        assert!(
            code.contains(r#"_createComponentWithFallback(_resolveDynamicComponent("a"), "#),
            "{code}"
        );
        assert!(!code.contains("createDynamicComponent"), "{code}");

        let code = compiled(r#"<component :is="view" is="a" />"#, retained);
        assert!(
            code.contains("_createDynamicComponent(() => (_ctx.view), "),
            "{code}"
        );
    }
}

#[test]
fn kebab_case_builtins_resolve_like_their_pascal_names() {
    // Upstream `isBuiltInType` accepts both spellings: `<transition>` is
    // `VaporTransition`, never a `resolveComponent("transition")`.
    for retained in [false, true] {
        for (pascal, kebab) in [
            ("Transition", "transition"),
            ("TransitionGroup", "transition-group"),
            ("KeepAlive", "keep-alive"),
            ("Teleport", "teleport"),
            ("Suspense", "suspense"),
        ] {
            let body = r#" to="body" tag="ul"><div v-if="ok">x</div></"#;
            let expected = compiled(&vize_carton::cstr!("<{pascal}{body}{pascal}>"), retained);
            let code = compiled(&vize_carton::cstr!("<{kebab}{body}{kebab}>"), retained);
            assert!(!code.contains("resolveComponent"), "{kebab}: {code}");
            assert_eq!(code, expected, "{kebab}");
        }
    }
}
