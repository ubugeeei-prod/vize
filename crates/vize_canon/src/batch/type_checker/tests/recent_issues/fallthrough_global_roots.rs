//! A component root that no setup binding owns is the component Vue resolves
//! from the registry, which `GlobalComponents` types. Its props, emits and
//! own fallthrough are what the outer component forwards, for the unknown
//! event check and the strict attribute check alike.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

type Diagnostic = (vize_carton::String, Option<u32>, vize_carton::String);

/// The attributes `Bar` forwards from `Foo`'s `<input>` root, as the strict
/// attribute check prints them.
const INPUT_ROOT_ATTRS: &str = "{ about?: unknown; accept?: unknown; accesskey?: unknown; alt?: unknown; \"aria-activedescendant\"?: unknown; \"aria-atomic\"?: unknown; \"aria-autocomplete\"?: unknown; \"aria-busy\"?: unknown; \"aria-checked\"?: unknown; ... 208 more ...; width?: unknown; }";

macro_rules! reported {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../../tests/_fixtures/differential/typechecker/global-component-fallthrough-root/",
            $file
        ))
    };
}

fn check(name: &str, tsconfig: &str, bar: &str, app: &str) -> Vec<Diagnostic> {
    let root = super::real_vue::create_project_case(
        vize_carton::cstr!("global-component-fallthrough-root-{name}").as_str(),
        &[
            ("tsconfig.json", tsconfig),
            ("components.d.ts", reported!("components.d.ts.txt")),
            ("Foo.vue", reported!("InputRoot.vue.txt")),
            ("Bar.vue", bar),
            ("App.vue", app),
        ],
    );
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot.unwrap()
}

/// The reported projects, byte for byte: the native listener and the native
/// attribute of `Foo`'s `<input>` root reach `Bar` through its global root,
/// typed by that root, while a name the root does not declare stays unknown.
#[test]
fn reported_projects_forward_through_a_global_component_root() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let events = reported!("tsconfig-events.json.txt");
    let attrs = reported!("tsconfig-attrs.json.txt");
    let emits_bar = reported!("EmitsGlobalRoot.vue.txt");
    let bar = reported!("GlobalRoot.vue.txt");
    for (name, tsconfig, bar, app, expected) in [
        (
            "listener",
            events,
            emits_bar,
            reported!("Listener.vue.txt"),
            Vec::new(),
        ),
        (
            "listener-payload",
            events,
            emits_bar,
            reported!("ListenerPayload.vue.txt"),
            vec![(
                "App.vue".into(),
                Some(2339),
                "6:35:error Property 'nope' does not exist on type 'KeyboardEvent'.".into(),
            )],
        ),
        (
            "attribute",
            attrs,
            bar,
            reported!("Attribute.vue.txt"),
            Vec::new(),
        ),
        (
            "unknown-attribute",
            attrs,
            bar,
            reported!("UnknownAttribute.vue.txt"),
            vec![(
                "App.vue".into(),
                Some(2353),
                vize_carton::cstr!(
                    "6:8:error Object literal may only specify known properties, and '\"nope\"' does not exist in type '$ComponentCheckProps<Props, $PublicComponentAttrs & {INPUT_ROOT_ATTRS}>'."
                ),
            )],
        ),
    ] {
        let expected: Vec<Diagnostic> = expected;
        assert_eq!(check(name, tsconfig, bar, app), expected, "{name}");
    }
}

/// A root tag nothing registers keeps the open surface: its listeners are
/// not the outer component's known events.
#[test]
fn an_unregistered_root_keeps_the_open_surface() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let bar = r#"<script setup lang="ts">
defineEmits<{ baz: [] }>();
</script>

<template>
  <Missing />
</template>
"#;
    let diagnostics = check(
        "unregistered",
        reported!("tsconfig-events.json.txt"),
        bar,
        reported!("Listener.vue.txt"),
    );
    assert!(
        diagnostics
            .iter()
            .any(|(file, code, _)| file.as_str() == "App.vue" && *code == Some(2353)),
        "{diagnostics:?}"
    );
}

/// A component whose root may be itself reads its own surface back the same
/// way whether the root is its `GlobalComponents` entry or a self import.
#[test]
fn a_recursive_global_root_matches_a_recursive_bound_root() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let app = r#"<script setup lang="ts">
import Tree from "./Tree.vue";
</script>

<template>
  <Tree :depth="2" disabled @keydown="(event) => event.key" />
</template>
"#;
    let mut results = Vec::new();
    for (name, import, registry) in [
        (
            "global",
            "",
            "declare module \"vue\" {\n  export interface GlobalComponents {\n    Tree: typeof import(\"./Tree.vue\").default;\n  }\n}\n\nexport {};\n",
        ),
        (
            "bound",
            "import Tree from \"./Tree.vue\";\n",
            "export {};\n",
        ),
    ] {
        let tree = vize_carton::cstr!(
            "<script setup lang=\"ts\">\n{import}defineProps<{{ depth: number }}>();\n</script>\n\n<template>\n  <Tree v-if=\"depth > 0\" :depth=\"depth - 1\" />\n  <input v-else />\n</template>\n"
        );
        let root = super::real_vue::create_project_case(
            vize_carton::cstr!("global-component-fallthrough-root-recursive-{name}").as_str(),
            &[
                ("tsconfig.json", reported!("tsconfig-attrs.json.txt")),
                ("components.d.ts", registry),
                ("Tree.vue", tree.as_str()),
                ("App.vue", app),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&root);
        let _ = std::fs::remove_dir_all(&root);
        let snapshot: Vec<Diagnostic> = snapshot
            .unwrap()
            .into_iter()
            .filter(|(file, _, _)| file.as_str() == "App.vue")
            .collect();
        results.push(snapshot);
    }
    assert_eq!(results[0], results[1]);
}
