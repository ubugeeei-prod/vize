//! Regression coverage for packaged Vue declarations shadowed by compiled SFCs (#12968).

use std::path::{Path, PathBuf};

use serde_json::json;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const DECLARATION: &str = r#"export interface BadgeProps { label: string }
declare const Badge: new () => {
  $props: BadgeProps;
  $slots: { header(props: { label: string }): unknown };
};
export default Badge;
export type Optional = import("missing-optional-integration").Options;
"#;

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn prepare_project() -> tempfile::TempDir {
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-shipped-declarations-12968-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());

    write(
        project.path(),
        "node_modules/fake-ui/package.json",
        r#"{"name":"fake-ui","types":"index.d.ts"}"#,
    );
    write(
        project.path(),
        "node_modules/fake-ui/index.d.ts",
        "export * from './Badge.vue';\nexport { default as Badge } from './Badge.vue';\n",
    );
    write(
        project.path(),
        "node_modules/fake-ui/Badge.vue",
        r#"<script>export default { props: { label: String } }</script><template><div /></template>"#,
    );
    write(
        project.path(),
        "node_modules/fake-ui/Badge.d.vue.ts",
        DECLARATION,
    );
    write(
        project.path(),
        "node_modules/fake-ui/Badge.vue.d.ts",
        &DECLARATION.replace("label: string", "label: number"),
    );
    write(
        project.path(),
        "src/main.ts",
        "import type { BadgeProps } from 'fake-ui';\nexport const props: BadgeProps = { label: 'ok' };\n",
    );
    write(
        project.path(),
        "src/App.vue",
        r#"<script setup lang="ts">
import { Badge } from 'fake-ui'
</script>
<template>
  <Badge label="ok"><template #header="{ label }">{{ label.toUpperCase() }}</template></Badge>
</template>
"#,
    );
    write(
        project.path(),
        "src/Local.vue",
        r#"<script setup lang="ts">defineProps<{ label: string }>()</script><template>{{ label }}</template>"#,
    );
    write(
        project.path(),
        "src/Local.d.vue.ts",
        &DECLARATION.replace("label: string", "label: number"),
    );
    write(
        project.path(),
        "src/LocalConsumer.vue",
        r#"<script setup lang="ts">import Local from './Local.vue'</script><template><Local label="ok" /></template>"#,
    );
    for (name, skip_lib_check) in [("tsconfig.json", true), ("tsconfig.checked.json", false)] {
        write(
            project.path(),
            name,
            &serde_json::to_string_pretty(&json!({
                "compilerOptions": {
                    "allowArbitraryExtensions": true,
                    "module": "preserve",
                    "moduleResolution": "bundler",
                    "noEmit": true,
                    "skipLibCheck": skip_lib_check,
                    "strict": true,
                    "types": [],
                },
                "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
                "include": ["src"],
            }))
            .unwrap(),
        );
    }
    project
}

#[test]
fn packaged_sfc_uses_its_declaration_without_hiding_consumer_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{} is not a file", tsgo.display());
    let project = prepare_project();

    let clean = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(clean.status.success(), "{}", output_text(&clean));

    let checked = check_project(&tsgo, project.path(), "tsconfig.checked.json");
    let checked_text = output_text(&checked);
    assert!(!checked.status.success(), "{checked_text}");
    assert!(checked_text.contains("TS2307"), "{checked_text}");
    assert!(
        checked_text.contains("missing-optional-integration"),
        "{checked_text}"
    );

    write(
        project.path(),
        "src/main.ts",
        "import type { BadgeProps } from 'fake-ui';\nexport const props: BadgeProps = { label: 123 };\n",
    );
    let consumer_error = check_project(&tsgo, project.path(), "tsconfig.json");
    let consumer_text = output_text(&consumer_error);
    assert!(!consumer_error.status.success(), "{consumer_text}");
    assert!(consumer_text.contains("src/main.ts") && consumer_text.contains("TS2322"));
    assert!(!consumer_text.contains("missing-optional-integration"));

    write(
        project.path(),
        "src/main.ts",
        "import type { BadgeProps } from 'fake-ui';\nexport const props: BadgeProps = { label: 'ok' };\n",
    );
    std::fs::remove_file(project.path().join("node_modules/fake-ui/Badge.d.vue.ts")).unwrap();
    write(
        project.path(),
        "node_modules/fake-ui/Badge.vue.d.ts",
        DECLARATION,
    );
    let fallback = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(fallback.status.success(), "{}", output_text(&fallback));

    std::fs::remove_file(project.path().join("node_modules/fake-ui/Badge.vue.d.ts")).unwrap();
    write(
        project.path(),
        "node_modules/fake-ui/Badge.vue",
        r#"<script lang="ts">export interface BadgeProps { label: string }</script>
<script setup lang="ts">defineProps<BadgeProps>()</script>
<template>{{ label }}</template>"#,
    );
    let source_fallback = check_project(&tsgo, project.path(), "tsconfig.json");
    assert!(
        source_fallback.status.success(),
        "{}",
        output_text(&source_fallback)
    );
}
