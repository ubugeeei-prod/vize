use super::*;

#[test]
fn setup_helpers_use_runtime_macro_types() {
    assert_virtual_ts_snapshot("virtual_ts_vue_setup_helpers", VUE_SETUP_HELPERS);
}

#[test]
fn vue3_default_excludes_legacy_members() {
    let v3 = template_context(&VirtualTsOptions::default(), VueVersion::V3);
    assert_eq!(v3.matches("$listeners").count(), 0);
    assert_eq!(v3.matches("$children").count(), 0);
    assert_eq!(v3.matches("$scopedSlots").count(), 0);
    assert_eq!(v3.matches("$createElement").count(), 0);
    assert_eq!(v3.matches("Vue 2 instance members").count(), 0);
}

// `__Global` and its two helpers are declared only where a global is typed
// through them: resolved CSS-module and typed-route globals carry their own
// type, and TypeScript cannot share per-file aliases across files.
#[test]
fn template_context_declares_the_global_helper_only_when_referenced() {
    use crate::virtual_ts::types::{CSS_MODULE_GLOBAL_MARKER, TYPED_ROUTE_GLOBAL_MARKER};

    const HELPER: &str = "    type $GlobalContextIsAny<T> = 0 extends (1 & T) ? true : false;\n    type $GlobalContextProperty<T, F> = $GlobalContextIsAny<T> extends true ? T : unknown extends T ? F : T;\n    type __Global<K extends string, F = unknown> = K extends keyof __Ctx ? $GlobalContextProperty<__Ctx[K], F> : F;\n";
    let resolved = |name: &str, type_annotation: &str, marker: &str| TemplateGlobal {
        name: name.into(),
        type_annotation: type_annotation.into(),
        default_value: marker.into(),
    };
    let plugin = TemplateGlobal {
        name: "$t".into(),
        type_annotation: "(...args: any[]) => string".into(),
        default_value: "(() => '') as any".into(),
    };
    let style = resolved(
        "$style",
        r#"Record<string, string> & { "root": string; }"#,
        CSS_MODULE_GLOBAL_MARKER,
    );
    let route = resolved("$route", "TypedRoute", TYPED_ROUTE_GLOBAL_MARKER);
    let cases = [
        (vec![style.clone(), route.clone()], vec![], 0),
        (vec![style.clone(), plugin.clone()], vec![], 1),
        (vec![style], vec!["$css".into()], 1),
    ];
    for (template_globals, css_modules, declared) in cases {
        let options = VirtualTsOptions {
            template_globals,
            css_modules,
            ..Default::default()
        };
        let ctx = template_context(&options, VueVersion::V3);
        assert_eq!(ctx.matches(HELPER).count(), declared, "{ctx}");
        assert_eq!(ctx.matches("__Global<'").count(), declared, "{ctx}");
        assert_eq!(
            ctx.matches(
                r#"    const $style: Record<string, string> & { "root": string; } = (void 0) as any;"#
            )
            .count(),
            1,
            "{ctx}"
        );
    }
}
