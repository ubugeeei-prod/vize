use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use serde_json::json;
use vize_s0::{String as CompactString, cstr};

const TSGO_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_TSGO";
const VUE_ENV: &str = "VIZE_TEST_CONTENT_MAPPER_VUE";

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("workspace root should exist")
}

fn copy_fixture(source: &Path, destination: &Path) {
    std::fs::create_dir_all(destination).unwrap();
    for entry in std::fs::read_dir(source).unwrap() {
        let entry = entry.unwrap();
        if entry.file_name() == "node_modules" {
            continue;
        }
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_fixture(&source_path, &destination_path);
        } else {
            std::fs::copy(source_path, destination_path).unwrap();
        }
    }
}

fn install_packages(project_root: &Path) {
    let mapper_root = project_root.join("node_modules/vize");
    std::fs::create_dir_all(&mapper_root).unwrap();
    std::fs::write(
        mapper_root.join("package.json"),
        serde_json::to_vec_pretty(&json!({
            "name": "vize",
            "private": true,
            "typescript": {
                "contentMapper": {
                    "exec": [env!("CARGO_BIN_EXE_vize"), "content-mapper"],
                    "compilerOptions": ["noUnusedLocals"],
                },
            },
        }))
        .unwrap(),
    )
    .unwrap();

    let vue_source = std::env::var_os(VUE_ENV)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            let store = workspace_root().join("node_modules/.pnpm");
            let mut candidates = std::fs::read_dir(&store)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", store.display()))
                .filter_map(Result::ok)
                .filter(|entry| entry.file_name().to_string_lossy().starts_with("vue@3."))
                .map(|entry| entry.path().join("node_modules/vue"))
                .filter(|path| path.join("package.json").is_file())
                .collect::<Vec<_>>();
            candidates.sort();
            candidates.pop().unwrap_or_else(|| {
                panic!(
                    "no Vue 3 package found under {}; set {VUE_ENV}",
                    store.display()
                )
            })
        });
    assert!(vue_source.join("package.json").is_file());
    let vue_target = project_root.join("node_modules/vue");
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue_source, vue_target).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue_source, vue_target).unwrap();
}

fn run_build(tsgo: &Path, project_root: &Path) -> Output {
    Command::new(tsgo)
        .current_dir(project_root)
        .args([
            "--build",
            "references/tsconfig.json",
            "--runExternalCode",
            "--pretty",
            "false",
            "--verbose",
        ])
        .output()
        .unwrap_or_else(|error| panic!("failed to run {}: {error}", tsgo.display()))
}

fn output_text(output: &Output) -> CompactString {
    let stdout = std::str::from_utf8(&output.stdout).unwrap_or("<non-UTF-8 stdout>");
    let stderr = std::str::from_utf8(&output.stderr).unwrap_or("<non-UTF-8 stderr>");
    cstr!(
        "status: {}\nstdout:\n{}\nstderr:\n{}",
        output.status,
        stdout,
        stderr,
    )
}

#[test]
fn standard_tsgo_builds_vue_project_references_incrementally() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper build conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(
        tsgo.is_file(),
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-build-")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let initial = run_build(&tsgo, project.path());
    assert!(initial.status.success(), "{}", output_text(&initial));
    for declaration in [
        "references/ui/dist/Counter.d.vue.ts",
        "references/ui/dist/index.d.ts",
        "references/app/dist/App.d.vue.ts",
        "references/app/dist/main.d.ts",
    ] {
        assert!(
            project.path().join(declaration).is_file(),
            "missing build output {declaration}"
        );
    }

    let unchanged = run_build(&tsgo, project.path());
    assert!(unchanged.status.success(), "{}", output_text(&unchanged));
    assert!(
        output_text(&unchanged).contains("up to date"),
        "no incremental no-op reported: {}",
        output_text(&unchanged)
    );

    let counter = project.path().join("references/ui/src/Counter.vue");
    let valid_counter = std::fs::read_to_string(&counter).unwrap();
    let invalid_counter = valid_counter.replace("count.toFixed(0)", "count.missing()");
    assert_ne!(valid_counter, invalid_counter);
    std::fs::write(&counter, invalid_counter).unwrap();

    let broken = run_build(&tsgo, project.path());
    assert!(!broken.status.success(), "broken build passed unexpectedly");
    let broken_text = output_text(&broken);
    assert!(
        broken_text.contains("Counter.vue")
            && broken_text.contains("TS2339")
            && broken_text.contains("Property 'missing' does not exist on type 'number'"),
        "{broken_text}"
    );

    std::fs::write(&counter, valid_counter).unwrap();
    let repaired = run_build(&tsgo, project.path());
    assert!(repaired.status.success(), "{}", output_text(&repaired));
}

#[test]
fn standard_tsgo_emits_authored_vue_declaration_maps() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper declaration-map oracle: {TSGO_ENV} is not set");
        return;
    };
    let fixture =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/content_mapper_project");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content mapper declaration maps ")
        .tempdir_in(cases_root)
        .unwrap();
    copy_fixture(&fixture, project.path());
    install_packages(project.path());

    let app = project.path().join("src/App.vue");
    let source = std::fs::read_to_string(&app).unwrap().replace('\n', "\r\n");
    std::fs::write(&app, cstr!("<!-- 💥 -->\r\n{source}")).unwrap();

    let spaced = project.path().join("src/Spaced Child.vue");
    std::fs::write(
        spaced,
        r#"<script setup lang="ts">
import type { VNode } from "vue";

defineProps<{
  render: () => VNode;
  unicodeLabel: "💥";
}>();
</script>

<template>
  <span>{{ unicodeLabel }}</span>
</template>
"#,
    )
    .unwrap();

    let emit = Command::new(&tsgo)
        .current_dir(project.path())
        .args([
            "--runExternalCode",
            "-p",
            "tsconfig.emit.json",
            "--pretty",
            "false",
        ])
        .output()
        .unwrap();
    assert!(emit.status.success(), "{}", output_text(&emit));
    assert!(project.path().join("dist/main.d.ts.map").is_file());

    let declarations = std::fs::read_dir(project.path().join("dist"))
        .unwrap()
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            let name = path
                .file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("");
            name.ends_with(".d.vue.ts") || name.ends_with(".vue.d.ts")
        })
        .collect::<Vec<_>>();
    assert!(!declarations.is_empty());

    let expected_components = [
        "App",
        "CallSignatureChild",
        "Child",
        "ConditionalGenericChild",
        "DefaultModelChild",
        "DynamicGenericChild",
        "GenericChild",
        "ModelChild",
        "NestedGenericChild",
        "Options",
        "Public",
        "RuntimeChild",
        "SlotGenericChild",
        "SlotProvider",
        "Spaced Child",
        "TsxScript",
    ]
    .into_iter()
    .map(str::to_string)
    .collect::<BTreeSet<_>>();
    let mut mapped_components = BTreeSet::new();

    for declaration in declarations {
        let name = declaration.file_name().unwrap().to_string_lossy();
        let component = name
            .strip_suffix(".d.vue.ts")
            .or_else(|| name.strip_suffix(".vue.d.ts"))
            .unwrap();
        mapped_components.insert(component.to_string());
        let mut map_name = declaration.as_os_str().to_os_string();
        map_name.push(".map");
        let map_path = PathBuf::from(map_name);
        assert!(
            map_path.is_file(),
            "missing map for {}",
            declaration.display()
        );
        let declaration_text = std::fs::read_to_string(&declaration).unwrap();
        let expected_mapping_url = format!("//# sourceMappingURL={}.map", name.replace(' ', "%20"));
        assert_eq!(
            declaration_text.lines().last(),
            Some(expected_mapping_url.as_str()),
            "declaration must end with an adjacent sourceMappingURL for {name}:\n{declaration_text}"
        );
        let map: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&map_path).unwrap()).unwrap();
        assert_eq!(map["version"], 3);
        assert_eq!(map["file"], name.as_ref());
        assert!(
            !map["mappings"].as_str().unwrap_or("").is_empty(),
            "{} has no declaration mappings: {map}",
            map_path.display()
        );
        let sources = map["sources"].as_array().expect("map sources");
        let expected = cstr!("{component}.vue");
        assert!(
            sources
                .iter()
                .filter_map(serde_json::Value::as_str)
                .any(|source| source.ends_with(expected.as_str())),
            "{} did not map to {expected}: {map}",
            map_path.display()
        );
        for source in sources.iter().filter_map(serde_json::Value::as_str) {
            assert!(
                !source.contains("__vize")
                    && !source.contains(".vue.ts")
                    && !source.contains(".vue.js")
                    && !source.contains("node_modules/.vize"),
                "{} leaked a generated or virtual source path: {map}",
                map_path.display()
            );
        }
    }

    assert_eq!(
        mapped_components, expected_components,
        "declaration-map oracle must cover every emitted Vue input"
    );
}
