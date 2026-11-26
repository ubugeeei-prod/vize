//! The mapper types `useTemplateRef` from the template wherever `vize check`
//! does: through a Vue namespace import, inside a local composable, and for
//! the static and ref-valued targets of `<component>`.
#![expect(clippy::disallowed_types, reason = "fixture IO uses std strings")]
#![expect(clippy::disallowed_methods, reason = "fixture IO uses std strings")]

use std::path::PathBuf;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const SCOPED: &str = include_str!(
    "../../../../tests/_fixtures/differential/typechecker/template-ref-inference-scope/Scoped.vue.txt"
);
const CONFIG: &str = r#"{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true
  },
  "include": ["src/**/*.vue"],
  "contentMappers": [{ "package": "vize", "extensions": [".vue"] }]
}
"#;

#[test]
fn namespace_local_function_and_component_targets_read_the_template_registry() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping template-ref inference scope conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} must name an executable");
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-template-ref-scope-")
        .tempdir_in(&cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    let page = project.path().join("src/Scoped.vue");
    std::fs::write(&page, SCOPED).unwrap();

    let output = check_project(&native, project.path(), "tsconfig.json");
    assert!(output.status.success(), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));

    // Misspelled keys are errors on their literals, and the component
    // targets are elements, not `any`.
    let probe = SCOPED
        .replace("Vue.useTemplateRef(\"panel\")", "Vue.useTemplateRef(\"pannel\")")
        .replace("useTemplateRef(\"dialog\")", "useTemplateRef(\"dialgo\")")
        .replace(
            "console.log(panel, area, fixed)",
            "console.log(panel, area, fixed, dynamicTarget.value?.__vizeMissing, staticTarget.value?.__vizeMissing)",
        );
    assert_ne!(probe, SCOPED);
    std::fs::write(&page, probe).unwrap();
    let output = check_project(&native, project.path(), "tsconfig.json");
    assert_eq!(output.status.code(), Some(2), "{}", output_text(&output));
    assert_eq!(
        String::from_utf8_lossy(&output.stdout),
        "src/Scoped.vue(6,41): error TS2339: Property 'pannel' does not exist on type '$TemplateRefs'.\n\
         src/Scoped.vue(11,33): error TS2339: Property 'dialgo' does not exist on type '$TemplateRefs'.\n\
         src/Scoped.vue(12,30): error TS2339: Property 'showModal' does not exist on type '{}'.\n\
         src/Scoped.vue(17,9): error TS2322: Type 'unknown' is not assignable to type 'HTMLDivElement | null'.\n\
         src/Scoped.vue(20,56): error TS2339: Property '__vizeMissing' does not exist on type 'HTMLElement'.\n\
         src/Scoped.vue(20,91): error TS2339: Property '__vizeMissing' does not exist on type 'HTMLDivElement'.\n",
        "{}",
        output_text(&output)
    );
}
