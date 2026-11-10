//! Each authored slot and `v-for` pattern diagnostic is reported once at its
//! binding, whether or not the component-props pass replays its scope
//! (#12479, #12520).

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use std::path::PathBuf;
use std::process::Command;

const CHILD: &str = r#"<script setup lang="ts">
defineSlots<{ default(props: { value: string; item: { id: number; label?: string } }): any }>();
</script>
<template><slot value="x" :item="{ id: 1 }" /></template>
"#;

const OTHER: &str = r#"<script setup lang="ts">
defineProps<{ num: number; text?: string }>();
</script>
<template><span>{{ num }}</span></template>
"#;

fn parent(template: &str) -> String {
    format!(
        "<script setup lang=\"ts\">\nimport Child from \"./Child.vue\";\nimport Other from \"./Other.vue\";\nconst items = [{{ id: 1, name: \"a\" }}];\nconst pairs: [number, string][] = [[1, \"a\"]];\n</script>\n<template>\n  {template}\n</template>\n"
    )
}

const CASES: [(&str, &str); 11] = [
    (
        "Missing.vue",
        r#"<Child><template #default="{ missing }">{{ missing }}</template></Child>"#,
    ),
    (
        "WrongUse.vue",
        r#"<Child><template #default="{ value }">{{ value.toFixed() }}</template></Child>"#,
    ),
    (
        "Nested.vue",
        r#"<Child><template #default="{ value = 'd', item: { id, missing = 1 } }">{{ value }}{{ id }}{{ missing }}</template></Child>"#,
    ),
    (
        "OnTag.vue",
        r#"<Child v-slot="{ missing }">{{ missing }}</Child>"#,
    ),
    (
        "LoopAlone.vue",
        r#"<div v-for="{ nme } in items">{{ nme }}</div>"#,
    ),
    (
        "LoopWithChild.vue",
        r#"<div v-for="{ nme } in items">{{ nme }}</div><Other :num="1" />"#,
    ),
    (
        "LoopArray.vue",
        r#"<div v-for="[id, label, extra] in pairs">{{ id }}{{ label.toFixed() }}{{ extra }}</div><Other :num="1" />"#,
    ),
    (
        "LoopCheck.vue",
        r#"<template v-for="{ id, nme } in items"><Other :num="id" :text="nme" /></template>"#,
    ),
    (
        "SlotLoop.vue",
        r#"<Child v-slot="{ value }"><div v-for="{ nme } in items">{{ value }}{{ nme }}</div></Child>"#,
    ),
    (
        "SlotLoopCheck.vue",
        r#"<Child v-slot="{ value }"><template v-for="{ nme } in items"><Other :num="value" :text="nme" /></template></Child>"#,
    ),
    (
        "WithComponent.vue",
        r#"<Child v-slot="{ value, missing }"><Other :num="value" :text="missing" /></Child>"#,
    ),
];

const ITEM: &str = "'{ id: number; name: string; }'";
const PAYLOAD: &str = "'{ value: string; item: { id: number; label?: string | undefined; }; }'";

const RENAMED_CHILD: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/slot-renamed-key-12511/Child.vue.txt"
));
const RENAMED_PARENT: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/slot-renamed-key-12511/Parent.vue.txt"
));
const RENAMED_CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/slot-renamed-key-12511/tsconfig.json.txt"
));

#[test]
#[ignore = "requires the authenticated beta host and actual source CLI in native-canon Actions"]
fn standard_tsgo_reports_each_replayed_pattern_diagnostic_once() {
    let host = std::env::var_os(TSGO_ENV)
        .map(PathBuf::from)
        .expect("this explicitly selected regression requires the real beta content-mapper host");
    let version = Command::new(&host).arg("--version").output().unwrap();
    assert_eq!(version.stdout, b"Version 7.1.0-beta\n");
    let parent_dir = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent_dir).unwrap();
    let project = tempfile::Builder::new()
        .prefix("replayed-pattern-duplicates-")
        .tempdir_in(&parent_dir)
        .unwrap();
    let root = project.path();
    install_mapper_manifest(root);
    install_vue_package(root);
    std::fs::create_dir_all(root.join("src")).unwrap();
    std::fs::write(
        root.join("tsconfig.json"),
        r#"{
  "compilerOptions": {
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "noEmit": true,
    "skipLibCheck": true,
    "strict": true,
    "target": "ES2022",
    "types": []
  },
  "include": ["src/**/*.vue"],
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
}"#,
    )
    .unwrap();
    std::fs::write(root.join("src/Child.vue"), CHILD).unwrap();
    std::fs::write(root.join("src/Other.vue"), OTHER).unwrap();
    for (name, template) in CASES {
        std::fs::write(root.join("src").join(name), parent(template)).unwrap();
    }

    let output = check_project(&host, root, "tsconfig.json");
    let expected = [
        format!("src/LoopAlone.vue(8,17): error TS2339: Property 'nme' does not exist on type {ITEM}."),
        "src/LoopArray.vue(8,27): error TS2493: Tuple type '[number, string]' of length '2' has no element at index '2'.".into(),
        "src/LoopArray.vue(8,61): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?".into(),
        format!("src/LoopCheck.vue(8,26): error TS2339: Property 'nme' does not exist on type {ITEM}."),
        format!("src/LoopWithChild.vue(8,17): error TS2339: Property 'nme' does not exist on type {ITEM}."),
        format!("src/Missing.vue(8,32): error TS2339: Property 'missing' does not exist on type {PAYLOAD}."),
        "src/Nested.vue(8,57): error TS2339: Property 'missing' does not exist on type '{ id: number; label?: string | undefined; }'.".into(),
        format!("src/OnTag.vue(8,20): error TS2339: Property 'missing' does not exist on type {PAYLOAD}."),
        format!("src/SlotLoop.vue(8,43): error TS2339: Property 'nme' does not exist on type {ITEM}."),
        format!("src/SlotLoopCheck.vue(8,48): error TS2339: Property 'nme' does not exist on type {ITEM}."),
        "src/SlotLoopCheck.vue(8,72): error TS2322: Type 'string' is not assignable to type 'number'.".into(),
        format!("src/WithComponent.vue(8,27): error TS2339: Property 'missing' does not exist on type {PAYLOAD}."),
        "src/WithComponent.vue(8,46): error TS2322: Type 'string' is not assignable to type 'number'.".into(),
        "src/WrongUse.vue(8,50): error TS2551: Property 'toFixed' does not exist on type 'string'. Did you mean 'fixed'?".into(),
    ];
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    assert_eq!(
        stdout.lines().collect::<Vec<_>>(),
        expected,
        "{}",
        output_text(&output)
    );
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));

    let renamed = tempfile::Builder::new()
        .prefix("slot-renamed-key-")
        .tempdir_in(parent_dir)
        .unwrap();
    install_mapper_manifest(renamed.path());
    install_vue_package(renamed.path());
    std::fs::create_dir_all(renamed.path().join("src")).unwrap();
    std::fs::write(renamed.path().join("tsconfig.json"), RENAMED_CONFIG).unwrap();
    std::fs::write(renamed.path().join("src/Child.vue"), RENAMED_CHILD).unwrap();
    std::fs::write(renamed.path().join("src/Parent.vue"), RENAMED_PARENT).unwrap();
    let output = check_project(&host, renamed.path(), "tsconfig.json");
    assert_eq!(
        std::str::from_utf8(&output.stdout)
            .unwrap()
            .lines()
            .collect::<Vec<_>>(),
        [
            "src/Parent.vue(5,32): error TS2339: Property 'absent' does not exist on type '{ value: string; }'."
        ],
        "{}",
        output_text(&output)
    );
    assert_eq!(output.stderr, b"", "{}", output_text(&output));
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
}
