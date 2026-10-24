use super::*;

const COMPONENT_COUNT: usize = 160;
const MIN_SAVED_INSTANTIATIONS_PER_COMPONENT: u64 = 200;

#[test]
fn shared_component_attribute_helpers_reduce_tsgo_instantiations() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping shared-helper performance receipt: {TSGO_ENV} is not set");
        return;
    };
    assert_eq!(
        tsgo.is_file(),
        true,
        "{TSGO_ENV} is not a file: {}",
        tsgo.display()
    );

    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let inline = tempfile::Builder::new()
        .prefix("content-mapper-helper-inline-")
        .tempdir_in(&cases_root)
        .unwrap();
    let shared = tempfile::Builder::new()
        .prefix("content-mapper-helper-shared-")
        .tempdir_in(&cases_root)
        .unwrap();
    prepare_project(inline.path(), false);
    prepare_project(shared.path(), true);

    let inline_output = check_with_extended_diagnostics(&tsgo, inline.path());
    let shared_output = check_with_extended_diagnostics(&tsgo, shared.path());
    assert_success(&inline_output);
    assert_success(&shared_output);

    let inline_instantiations = metric(&inline_output, "Instantiations");
    let shared_instantiations = metric(&shared_output, "Instantiations");
    eprintln!(
        "shared helper receipt: inline={inline_instantiations}, shared={shared_instantiations}, components={COMPONENT_COUNT}"
    );
    assert_eq!(
        shared_instantiations < inline_instantiations,
        true,
        "shared helper must reduce tsgo instantiations: inline={inline_instantiations}, shared={shared_instantiations}"
    );
    let saved_instantiations = inline_instantiations - shared_instantiations;
    let minimum_saved =
        u64::try_from(COMPONENT_COUNT).unwrap() * MIN_SAVED_INSTANTIATIONS_PER_COMPONENT;
    assert_eq!(
        saved_instantiations >= minimum_saved,
        true,
        "shared helper must save at least {MIN_SAVED_INSTANTIATIONS_PER_COMPONENT} instantiations per generated component: inline={inline_instantiations}, shared={shared_instantiations}, saved={saved_instantiations}, minimum={minimum_saved}"
    );
}

fn prepare_project(project: &Path, shared_helpers: bool) {
    install_mapper_manifest(project);
    install_vue_package(project);
    if shared_helpers {
        let package_root = project.join("node_modules/vize");
        let manifest_path = package_root.join("package.json");
        let mut manifest: serde_json::Value =
            serde_json::from_slice(&std::fs::read(&manifest_path).unwrap()).unwrap();
        manifest["exports"] = json!({
            "./content-mapper-helpers": {
                "types": "./content-mapper-helpers.d.ts"
            }
        });
        std::fs::write(manifest_path, serde_json::to_vec_pretty(&manifest).unwrap()).unwrap();
        std::fs::write(
            package_root.join("content-mapper-helpers.d.ts"),
            include_str!("../../../../npm/cli/content-mapper-helpers.d.ts"),
        )
        .unwrap();
    }

    let source_root = project.join("src");
    std::fs::create_dir_all(&source_root).unwrap();
    std::fs::write(
        source_root.join("Child.vue"),
        r#"<script setup lang="ts">
defineProps<{ title: string }>();
defineEmits<{ saveItem: [value: number] }>();
</script>
<template><button :title="title"><slot /></button></template>
"#,
    )
    .unwrap();
    for index in 0..COMPONENT_COUNT {
        std::fs::write(
            source_root.join(cstr!("View{index}.vue").as_str()),
            cstr!(
                r#"<script setup lang="ts">
import Child from "./Child.vue";
const label = "row-{index}";
</script>
<template><Child :title="label" data-test-id="row" aria-label="row" @save-item="() => {{}}">{{ label }}</Child></template>
"#
            ),
        )
        .unwrap();
    }
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true,
                "target": "ESNext",
                "module": "ESNext",
                "moduleResolution": "Bundler",
                "skipLibCheck": false,
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src/**/*.vue"],
        }))
        .unwrap(),
    )
    .unwrap();
}

fn check_with_extended_diagnostics(tsgo: &Path, project: &Path) -> Output {
    run_tsgo(
        tsgo,
        project,
        &[
            "--runExternalCode",
            "--noEmit",
            "--extendedDiagnostics",
            "--singleThreaded",
            "-p",
            "tsconfig.json",
            "--pretty",
            "false",
        ],
    )
}

fn metric(output: &Output, name: &str) -> u64 {
    let text = cstr!(
        "{}\n{}",
        std::str::from_utf8(&output.stdout).unwrap(),
        std::str::from_utf8(&output.stderr).unwrap()
    );
    text.lines()
        .filter_map(|line| line.split_once(':'))
        .find_map(|(label, value)| {
            (label.trim() == name).then(|| value.trim().replace(',', "").parse::<u64>().unwrap())
        })
        .unwrap_or_else(|| panic!("missing {name} in {}", output_text(output)))
}
