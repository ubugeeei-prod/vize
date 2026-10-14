//! The original configured native owner keeps open props opaque and closed slots strict.
use super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::String;

const APP: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/release-js-pragma-opaque-slots/App.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/Child.vue.txt"
);
const OPAQUE: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/release-js-pragma-opaque-slots/opaque.d.ts.txt"
);

#[test]
fn native_opaque_slot_hosts_do_not_acquire_a_known_empty_contract() {
    let Some(_native) = resolve_test_tsgo_binary() else {
        return;
    };
    for (name, shim, expected) in [
        ("opaque", OPAQUE, None),
        (
            "empty-vue",
            "declare module '*.vue' { import type { DefineComponent } from 'vue'; const component: DefineComponent<{}, {}, any>; export default component; }\n",
            Some("6:16:error Property 'header' does not exist on type '{}'."),
        ),
        (
            "empty-native",
            "declare module '*.vue' { const component: new () => { $props: Record<string, unknown>; $slots: {} }; export default component; }\n",
            Some("6:16:error Property 'header' does not exist on type '{}'."),
        ),
        (
            "typed-correct",
            "declare module '*.vue' { const component: new () => { $props: {}; $slots: { header?: (props: { index: number }) => unknown } }; export default component; }\n",
            None,
        ),
        (
            "typed-wrong",
            "declare module '*.vue' { const component: new () => { $props: {}; $slots: { footer?: (props: { index: number }) => unknown } }; export default component; }\n",
            Some(
                "6:16:error Property 'header' does not exist on type '{ footer?: ((props: { index: number; }) => unknown) | undefined; }'.",
            ),
        ),
    ] {
        let project = super::real_vue::create_project_case(
            name,
            &[
                ("src/App.vue", APP),
                ("src/Child.vue", CHILD),
                ("src/shims.d.ts", shim),
            ],
        );
        std::fs::write(project.join("tsconfig.json"), r#"{"compilerOptions":{"strict":true,"target":"ES2022","module":"ESNext","moduleResolution":"bundler","skipLibCheck":true,"allowJs":false},"include":["src/**/*"]}"#).unwrap();
        let actual = snapshot_project_diagnostics(&project);
        let expected = expected
            .map(|message| {
                vec![(
                    String::from("src/App.vue"),
                    Some(2339),
                    String::from(message),
                )]
            })
            .unwrap_or_default();
        eprintln!("release opaque host={name} complete native vector={actual:#?}");
        assert_eq!(
            actual,
            Some(expected),
            "complete actual diagnostic vector for {name}"
        );
        let _ = std::fs::remove_dir_all(project);
    }
}
