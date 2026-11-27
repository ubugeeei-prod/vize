//! Vue's compiler gives `<component is>` and its built-in components their
//! component before it reads setup bindings. A binding spelled like such a
//! root's tag does not own it, so the root keeps the open attributes surface
//! an unresolved component root has.

use super::super::{resolve_test_tsgo_binary, snapshot_project_diagnostics};

type Diagnostic = (vize_carton::String, Option<u32>, vize_carton::String);

fn check(name: &str, foo: &str, app: &str) -> Vec<Diagnostic> {
    let root = super::real_vue::create_project_case(
        name,
        &[
            ("src/Foo.vue", foo),
            ("src/Leaf.vue", LEAF),
            ("src/App.vue", app),
        ],
    );
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ESNext",
    "lib": ["ESNext", "DOM"]
  },
  "vueCompilerOptions": { "strictTemplates": true, "fallthroughAttributes": true },
  "include": ["src/**/*.vue"]
}"#,
    )
    .unwrap();
    let snapshot = snapshot_project_diagnostics(&root);
    let _ = std::fs::remove_dir_all(&root);
    snapshot.unwrap()
}

const LEAF: &str = r#"<script setup lang="ts">
defineProps<{ size: string }>();
</script>

<template>
  <span />
</template>
"#;

const SPREAD: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";

const baz = { bar: "baz" };
</script>

<template>
  <Foo class="qux" :="baz" />
</template>
"#;

const WRONG_PROP: &str = r#"<script setup lang="ts">
import Foo from "./Foo.vue";
</script>

<template>
  <Foo class="qux" :bar="1" />
</template>
"#;

/// `Foo` declares `bar` and renders `root`, with `binding` in its setup.
fn foo(binding: &str, root: &str) -> vize_carton::String {
    vize_carton::cstr!(
        "<script setup lang=\"ts\">\nimport Leaf from \"./Leaf.vue\";\ndefineProps<{{ bar: string }}>();\n{binding}\n</script>\n\n<template>\n  {root}\n</template>\n"
    )
}

const ROOTS: &[(&str, &str, &str)] = &[
    (
        "dynamic",
        "const component = \"div\";",
        "<component :is=\"component\" />",
    ),
    (
        "dynamic-literal",
        "const component = Leaf;",
        "<component :is=\"'div'\" />",
    ),
    (
        "dynamic-static",
        "const component = Leaf;",
        "<component is=\"div\" />",
    ),
    (
        "dynamic-pascal",
        "const Component = Leaf;",
        "<Component :is=\"'div'\" />",
    ),
    (
        "transition",
        "const Transition = Leaf;",
        "<Transition><div /></Transition>",
    ),
    (
        "transition-kebab",
        "const TransitionGroup = Leaf;",
        "<transition-group><div /></transition-group>",
    ),
    (
        "keep-alive",
        "const KeepAlive = Leaf;",
        "<keep-alive><div /></keep-alive>",
    ),
    (
        "teleport",
        "const teleport = Leaf;",
        "<teleport to=\"body\"><div /></teleport>",
    ),
    (
        "suspense",
        "const Suspense = Leaf;",
        "<Suspense><div /></Suspense>",
    ),
];

#[test]
fn a_binding_spelled_like_a_builtin_root_does_not_type_its_fallthrough() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, binding, root) in ROOTS {
        let foo = foo(binding, root);
        // Only the parent reads `Foo`'s fallthrough surface.
        let parent: Vec<Diagnostic> = check(
            vize_carton::cstr!("fallthrough-builtin-root-{name}").as_str(),
            foo.as_str(),
            SPREAD,
        )
        .into_iter()
        .filter(|(file, _, _)| file.as_str() == "src/App.vue")
        .collect();
        assert_eq!(parent, [], "{name}");
    }
}

#[test]
fn a_builtin_root_still_checks_the_component_props() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    let foo = foo(
        "const component = \"div\";",
        "<component :is=\"component\" />",
    );
    assert_eq!(
        check(
            "fallthrough-builtin-root-wrong-prop",
            foo.as_str(),
            WRONG_PROP
        ),
        vec![(
            "src/App.vue".into(),
            Some(2322),
            "6:21:error Type 'number' is not assignable to type 'string'.".into(),
        )],
    );
}

macro_rules! reported {
    ($file:literal) => {
        include_str!(concat!(
            "../../../../../../../tests/_fixtures/differential/typechecker/dynamic-component-fallthrough-root-13720/",
            $file
        ))
    };
}

/// The reported project, byte for byte: a setup binding named `component`
/// beside `<component :is>` leaves the spread on `Foo` valid, and a wrong
/// value for `Foo`'s own prop is still reported.
#[test]
fn reported_project_keeps_a_dynamic_root_open() {
    if resolve_test_tsgo_binary().is_none() {
        return;
    }
    for (name, app, expected) in [
        ("spread", reported!("Spread.vue.txt"), Vec::new()),
        (
            "wrong-prop",
            reported!("WrongProp.vue.txt"),
            vec![(
                "App.vue".into(),
                Some(2322),
                "8:21:error Type 'number' is not assignable to type 'string'.".into(),
            )],
        ),
    ] {
        let root = super::real_vue::create_project_case(
            vize_carton::cstr!("dynamic-component-fallthrough-root-{name}").as_str(),
            &[
                ("tsconfig.json", reported!("tsconfig.json.txt")),
                ("Foo.vue", reported!("DynamicRoot.vue.txt")),
                ("App.vue", app),
            ],
        );
        let snapshot = snapshot_project_diagnostics(&root);
        let _ = std::fs::remove_dir_all(&root);
        let expected: Vec<Diagnostic> = expected;
        assert_eq!(snapshot.unwrap(), expected, "{name}");
    }
}
