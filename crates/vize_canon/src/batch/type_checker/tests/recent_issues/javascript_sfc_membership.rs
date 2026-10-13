//! The configured project declaration owns JavaScript SFCs denied by allowJs.

use super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};
use vize_carton::{String, cstr};

const APP: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/App.vue.txt"
);
const CHILD: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/Child.vue.txt"
);
const SHIM: &str = include_str!(
    "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/shims.d.ts.txt"
);

#[test]
fn javascript_sfc_module_membership_preserves_native_diagnostics() {
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    for (name, flags, child, shim, expected) in [
        (
            "denied",
            r#""allowJs":false,"checkJs":false"#,
            CHILD.into(),
            SHIM,
            Some((
                2339,
                "6:16:error Property 'actions' does not exist on type '{}'.",
            )),
        ),
        (
            "allowed",
            r#""allowJs":true,"checkJs":false"#,
            CHILD.into(),
            SHIM,
            None,
        ),
        ("checked", r#""checkJs":true"#, CHILD.into(), SHIM, None),
        (
            "checked-explicit",
            r#""allowJs":true,"checkJs":true"#,
            CHILD.into(),
            SHIM,
            None,
        ),
        (
            "checked-denied",
            r#""allowJs":false,"checkJs":true"#,
            CHILD.into(),
            SHIM,
            Some((
                2339,
                "6:16:error Property 'actions' does not exist on type '{}'.",
            )),
        ),
        (
            "typescript",
            r#""allowJs":false"#,
            CHILD
                .replace("<script setup>", "<script setup lang=\"ts\">")
                .into(),
            SHIM,
            None,
        ),
        (
            "template-only",
            r#""allowJs":false"#,
            String::from("<template><slot name=\"actions\" :index=\"1\" /></template>\n"),
            SHIM,
            None,
        ),
        (
            "missing-shim",
            r#""allowJs":false"#,
            CHILD.into(),
            "",
            Some((
                2307,
                "2:19:error Cannot find module './Child.vue' or its corresponding type declarations.",
            )),
        ),
        (
            "typed-shim",
            r#""allowJs":false"#,
            CHILD.into(),
            r#"declare module '*.vue' {
export default class Child {
  $props: {};
  $slots: { actions?: (props: { index: number }) => unknown };
}
}
"#,
            None,
        ),
        (
            "unknown-component",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: any; export default component; }\n",
            None,
        ),
        (
            "unknown-constructor-instance",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: new () => any; export default component; }\n",
            None,
        ),
        (
            "unknown-constructor-slots",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: new () => { $props: {}; $slots: any }; export default component; }\n",
            None,
        ),
        (
            "unknown-constructor-slot-type",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: new () => { $props: {}; $slots: unknown }; export default component; }\n",
            None,
        ),
        (
            "unrelated-constructor-static-fields",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: (new () => any) & { __isFragment: string; __isTeleport: string; __isSuspense: string; slots: {} }; export default component; }\n",
            None,
        ),
        (
            "unknown-indexed-constructor",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: (new () => any) & Record<string, any> & { slots: {} }; export default component; }\n",
            None,
        ),
        (
            "empty-constructor-slots",
            r#""allowJs":false"#,
            CHILD.into(),
            "declare module '*.vue' { const component: new () => { $props: {}; $slots: {} }; export default component; }\n",
            Some((
                2339,
                "6:16:error Property 'actions' does not exist on type '{}'.",
            )),
        ),
        (
            "typed-vue-options",
            r#""allowJs":false"#,
            CHILD.into(),
            r#"declare module '*.vue' {
import type { DefineComponent, SlotsType } from 'vue';
const component: DefineComponent<{}, {}, any, {}, {}, {}, {}, {}, string, {}, {}, {}, SlotsType<{ actions?: (props: { index: number }) => unknown }>>;
export default component;
}
"#,
            None,
        ),
        (
            "inherited-denied",
            r#""allowJs":false,"checkJs":true"#,
            CHILD.into(),
            SHIM,
            Some((
                2339,
                "6:16:error Property 'actions' does not exist on type '{}'.",
            )),
        ),
        (
            "pragma-denied",
            r#""allowJs":false"#,
            CHILD
                .replace("const index", "// @ts-check\nconst index")
                .into(),
            SHIM,
            Some((
                2339,
                "6:16:error Property 'actions' does not exist on type '{}'.",
            )),
        ),
    ] {
        let project = super::real_vue::create_project_case(
            cstr!("javascript-sfc-membership-{name}").as_str(),
            &[
                ("src/App.vue", APP),
                ("src/Child.vue", child.as_str()),
                ("src/shims.d.ts", shim),
            ],
        );
        let config = cstr!(
            "{{\"compilerOptions\":{{\"strict\":true,\"target\":\"ES2022\",\"module\":\"ESNext\",\"moduleResolution\":\"bundler\",\"skipLibCheck\":true,{flags}}},\"include\":[\"src/**/*\"]}}"
        );
        std::fs::write(project.join("tsconfig.json"), config.as_str()).unwrap();
        if matches!(name, "checked" | "inherited-denied") {
            std::fs::write(project.join("base.json"), config.as_str()).unwrap();
            std::fs::write(
                project.join("tsconfig.json"),
                r#"{"extends":"./base.json","include":["src/**/*"]}"#,
            )
            .unwrap();
        }
        let vue = std::fs::read_to_string(project.join("node_modules/vue/package.json"))
            .expect("actual installed Vue metadata");
        eprintln!(
            "javascript SFC original provider native={} vue={vue}",
            native.display()
        );
        let actual = snapshot_project_diagnostics(&project);
        // Native TypeScript rejects this contradictory pair before semantic
        // diagnostics; policy precedence has a separate pure ownership law.
        let expected = if matches!(name, "checked-denied" | "inherited-denied") {
            vec![(
                String::from("tsconfig.json"),
                Some(5052),
                String::from(
                    "1:1:error Option 'checkJs' cannot be specified without specifying option 'allowJs'.",
                ),
            )]
        } else {
            expected
                .map(|(code, message)| {
                    vec![(
                        String::from("src/App.vue"),
                        Some(code),
                        String::from(message),
                    )]
                })
                .unwrap_or_default()
        };
        eprintln!(
            "javascript SFC membership case={name} config={config} whole_diagnostics={actual:#?}"
        );
        assert_eq!(
            actual,
            Some(expected),
            "whole native diagnostic vector for {name}"
        );
        let _ = std::fs::remove_dir_all(project);
    }
}

#[test]
fn non_strict_owned_slot_markers_preserve_known_quoted_and_camel_keys() {
    let Some(native) = resolve_test_tsgo_binary() else {
        return;
    };
    let original = include_str!(
        "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/NonStrictSlots.vue.txt"
    );
    let child = include_str!(
        "../../../../../../../tests/_fixtures/differential/canon/javascript-sfc-membership/NonStrictChild.vue.txt"
    );
    let wrong = original.replace("<!-- @vue-expect-error -->", "");
    let repaired = wrong
        .replace("last-columnsasd", "last-columns")
        .replace("lastColumnsasd", "lastColumns");
    for (name, source, expected) in [
        ("original", original, Vec::new()),
        (
            "wrong",
            wrong.as_str(),
            vec![
                (
                    String::from("src/main.vue"),
                    Some(2339),
                    String::from(
                        "16:14:error Property 'last-columnsasd' does not exist on type 'Partial<ISlots>'.",
                    ),
                ),
                (
                    String::from("src/main.vue"),
                    Some(2339),
                    String::from(
                        "19:14:error Property 'lastColumnsasd' does not exist on type 'Partial<ISlots>'.",
                    ),
                ),
            ],
        ),
        ("repaired", repaired.as_str(), Vec::new()),
    ] {
        let project = super::real_vue::create_project_case(
            cstr!("non-strict-known-slots-{name}").as_str(),
            &[("src/main.vue", source), ("src/component.vue", child)],
        );
        let mut config: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../../../../tests/_fixtures/vue-language-tools/upstream/test-workspace/tsconfig.base.json"
        )).unwrap();
        config["compilerOptions"]["strict"] = serde_json::json!(false);
        config["compilerOptions"]["checkJs"] = serde_json::json!(true);
        config["include"] = serde_json::json!(["src/**/*"]);
        std::fs::write(
            project.join("tsconfig.json"),
            serde_json::to_vec(&config).unwrap(),
        )
        .unwrap();
        let actual = snapshot_project_diagnostics(&project);
        eprintln!(
            "non-strict known slot provider native={} case={name} whole_diagnostics={actual:#?}",
            native.display()
        );
        assert_eq!(
            actual,
            Some(expected),
            "whole original/wrong/repaired slot vector for {name}"
        );
        let _ = std::fs::remove_dir_all(project);
    }
}
