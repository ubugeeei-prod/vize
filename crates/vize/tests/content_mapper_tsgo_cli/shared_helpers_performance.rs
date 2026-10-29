use super::*;

const COMPONENT_COUNT: usize = 160;
const MIN_SAVED_INSTANTIATIONS_PER_COMPONENT: u64 = 200;

#[test]
fn standard_tsgo_reports_each_component_prop_problem_once_with_shared_helpers_reduces_instantiations()
 {
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

#[test]
fn standard_tsgo_reports_each_component_prop_problem_once_with_pattern_injected_listeners() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping shared-helper listener receipt: {TSGO_ENV} is not set");
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
    let receipt = tempfile::Builder::new()
        .prefix("content-mapper-pattern-listener-")
        .tempdir_in(&cases_root)
        .unwrap();
    for shared_helpers in [false, true] {
        for (declared_listener, owner) in [(false, "fallthrough"), (true, "emit")] {
            for valid_listener in [false, true] {
                let mode = if shared_helpers { "shared" } else { "embedded" };
                let case = if valid_listener { "valid" } else { "invalid" };
                let project = receipt.path().join(cstr!("{mode}-{owner}-{case}").as_str());
                prepare_pattern_listener_project(
                    &project,
                    shared_helpers,
                    declared_listener,
                    valid_listener,
                );
                let output = check_project(&tsgo, &project, "tsconfig.json");
                assert_eq!(output.stderr, b"", "{}", output_text(&output));
                let stdout = std::str::from_utf8(&output.stdout).unwrap();
                if valid_listener {
                    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
                    assert_eq!(stdout, "");
                    continue;
                }

                assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
                let diagnostics = stdout
                    .lines()
                    .filter(|line| line.contains(": error TS"))
                    .collect::<Vec<_>>();
                assert_eq!(diagnostics.len(), 1, "{}", output_text(&output));
                let (identity, _) = diagnostics[0]
                    .split_once(": error TS2322: ")
                    .expect("one TS2322 listener diagnostic");
                assert_eq!(identity, "src/App.vue(6,26)");
            }
        }
    }
}

fn install_shared_helper_package(project: &Path) {
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
    // The helper's type-only `.js` specifier resolves to this declaration
    // under the fixture's Bundler module resolution, as in the package.
    let generated_root = package_root.join("generated");
    std::fs::create_dir_all(&generated_root).unwrap();
    let shared_types = include_str!("../../../../npm/cli/generated/content-mapper-types.d.ts");
    let installed_types = generated_root.join("content-mapper-types.d.ts");
    std::fs::write(&installed_types, shared_types).unwrap();
    assert_eq!(
        std::fs::read_to_string(installed_types).unwrap(),
        shared_types
    );
}

fn prepare_mapper_project(project: &Path, shared_helpers: bool) {
    install_mapper_manifest(project);
    install_vue_package(project);
    if shared_helpers {
        install_shared_helper_package(project);
    }
}

fn prepare_project(project: &Path, shared_helpers: bool) {
    prepare_mapper_project(project, shared_helpers);

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

fn prepare_pattern_listener_project(
    project: &Path,
    shared_helpers: bool,
    declared_listener: bool,
    valid_listener: bool,
) {
    prepare_mapper_project(project, shared_helpers);
    let source_root = project.join("src");
    std::fs::create_dir_all(&source_root).unwrap();
    std::fs::write(
        source_root.join("augment.d.ts"),
        r#"export {};

type Decorate<T extends Record<string, any>> = {
  [K in keyof T as K extends string ? `$${K}` : never]: T[K];
};

declare module "vue" {
  interface ComponentCustomProperties extends Decorate<Record<string, unknown>> {}
}
"#,
    )
    .unwrap();
    let emits = if declared_listener {
        "click: [event: MouseEvent]"
    } else {
        "change: []"
    };
    std::fs::write(
        source_root.join("MyItem.vue"),
        cstr!("<script setup lang=\"ts\">\ndefineEmits<{{ {emits} }}>();\n</script>\n"),
    )
    .unwrap();
    let (listener, body) = if valid_listener {
        ("MouseEvent", "event.button")
    } else {
        ("string", "event.length")
    };
    std::fs::write(
        source_root.join("App.vue"),
        cstr!(
            r#"<script setup lang="ts">
import MyItem from "./MyItem.vue";
const onClick = (event: {listener}) => {body};
</script>
<template>
  <MyItem role="button" @click="onClick" />
</template>
"#
        ),
    )
    .unwrap();
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "target": "ESNext",
                "module": "preserve",
                "moduleResolution": "bundler",
                "strict": true,
                "noEmit": true,
                "skipLibCheck": true,
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"],
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
