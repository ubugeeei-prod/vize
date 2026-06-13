use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
};

fn temp_project_dir(test_name: &str) -> PathBuf {
    let nonce = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "vize-build-cli-{}-{}-{}",
        std::process::id(),
        test_name,
        nonce
    ))
}

fn write_project_file(root: &Path, path: &str, content: &str) {
    let file_path = root.join(path);
    if let Some(parent) = file_path.parent() {
        fs::create_dir_all(parent).unwrap();
    }
    fs::write(file_path, content).unwrap();
}

#[test]
fn build_stats_profile_reports_compile_cache_counters() {
    let project_root = temp_project_dir("stats-profile-cache-counters");
    let source = r#"<template><div>Hello</div></template>
"#;
    write_project_file(&project_root, "src/App.vue", source);
    write_project_file(&project_root, "src/Foo.vue", source);

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "--format",
            "stats",
            "--profile",
            "--threads",
            "1",
            "src/App.vue",
            "src/Foo.vue",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Stats compile cache"), "{stderr}");
    assert!(stderr.contains("cache.stats_compile.hits"), "{stderr}");
    assert!(stderr.contains("cache.stats_compile.misses"), "{stderr}");
    assert!(stderr.contains("cache.stats_compile.stores"), "{stderr}");

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn build_resolves_imported_base_interface_props_in_normal_script() {
    let project_root = temp_project_dir("imported-base-interface-props");
    write_project_file(
        &project_root,
        "src/primitive.ts",
        r#"export interface PrimitiveProps {
  asChild?: boolean
  as?: string
}
"#,
    );
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<script lang="ts">
import type { PrimitiveProps } from './primitive'

export interface AppProps extends PrimitiveProps {
  feature?: 'focusable' | 'hidden'
}
</script>

<script setup lang="ts">
withDefaults(defineProps<AppProps>(), {
  as: 'span',
  feature: 'focusable',
})
</script>

<template>
  <div
    :data-as="as"
    :data-as-child="asChild"
    :data-feature="feature"
  ></div>
</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args(["build", "--format", "js", "src/App.vue", "--output", "dist"])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(project_root.join("dist/App.js")).unwrap();
    assert!(
        js.contains("asChild: {\n      type: Boolean,\n      required: false\n    }"),
        "{js}"
    );
    assert!(
        js.contains(
            "as: {\n      type: String,\n      required: false,\n      default: \"span\"\n    }"
        ),
        "{js}"
    );
    assert!(
        js.contains("feature: {\n      type: String,\n      required: false,\n      default: \"focusable\"\n    }"),
        "{js}"
    );
    assert!(js.contains("\"data-as\": __props.as"), "{js}");
    assert!(js.contains("\"data-as-child\": __props.asChild"), "{js}");
    assert!(js.contains("\"data-feature\": __props.feature"), "{js}");
    assert!(!js.contains("_ctx.as"), "{js}");
    assert!(!js.contains("_ctx.asChild"), "{js}");

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn build_resolves_props_from_mixed_reexported_vue_interface() {
    let project_root = temp_project_dir("mixed-reexported-vue-interface-props");
    write_project_file(
        &project_root,
        "src/primitive.ts",
        r#"export type AsTag = 'div' | 'span' | ({} & string)

export interface PrimitiveProps {
  asChild?: boolean
  as?: AsTag
}
"#,
    );
    write_project_file(
        &project_root,
        "src/content/Content.vue",
        r#"<script lang="ts">
import type { PrimitiveProps } from '../primitive'

export interface ContentProps extends PrimitiveProps {
  forceMount?: boolean
}
</script>

<script setup lang="ts">
defineProps<ContentProps>()
</script>

<template><div></div></template>
"#,
    );
    write_project_file(
        &project_root,
        "src/content/index.ts",
        r#"export {
  default as Content,
  type ContentProps,
} from './Content.vue'
"#,
    );
    write_project_file(
        &project_root,
        "src/Wrapper.vue",
        r#"<script lang="ts">
import type { ContentProps } from './content'

export interface WrapperProps extends ContentProps {}
</script>

<script setup lang="ts">
import { Content } from './content'

const props = defineProps<WrapperProps>()
</script>

<template>
  <Content
    :as-child="props.asChild"
    :as="as"
    :force-mount="props.forceMount"
  />
</template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "--format",
            "js",
            "src/Wrapper.vue",
            "--output",
            "dist",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let js = fs::read_to_string(project_root.join("dist/Wrapper.js")).unwrap();
    assert!(
        js.contains("asChild: {\n      type: Boolean,\n      required: false\n    }"),
        "{js}"
    );
    assert!(
        js.contains("forceMount: {\n      type: Boolean,\n      required: false\n    }"),
        "{js}"
    );
    assert!(js.contains("as: __props.as"), "{js}");
    assert!(!js.contains("_ctx.as"), "{js}");

    let _ = fs::remove_dir_all(project_root);
}

#[test]
fn build_respects_configured_template_syntax_quirks() {
    let project_root = temp_project_dir("configured-template-syntax-quirks");
    write_project_file(
        &project_root,
        "vize.config.ts",
        r#"export default {
  compiler: {
    templateSyntax: "quirks",
  },
}
"#,
    );
    write_project_file(
        &project_root,
        "src/App.vue",
        r#"<template><div /></template>
"#,
    );

    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(&project_root)
        .args([
            "build",
            "--format",
            "json",
            "src/App.vue",
            "--output",
            "dist",
        ])
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    let json: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(project_root.join("dist/App.json")).unwrap())
            .unwrap();
    assert_eq!(
        json["warnings"].as_array().unwrap().len(),
        0,
        "quirks syntax should not warn for invalid self-closing HTML: {json:#}"
    );

    let _ = fs::remove_dir_all(project_root);
}
