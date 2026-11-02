//! Template reads of `defineModel()` props resolve like Vue's PROPS bindings
//! (misskey MkTab TS2451/TS2339), while unknown names still report.

use std::path::PathBuf;

use serde_json::json;

use super::{
    TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

#[test]
fn template_model_props_keep_only_authored_errors() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-template-model-props");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-template-model-props-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    for name in ["Tab", "Named", "Only"] {
        std::fs::copy(
            fixture.join(cstr!("{name}.vue.txt").as_str()),
            project.join(cstr!("src/{name}.vue").as_str()),
        )
        .unwrap();
    }
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext", "moduleResolution": "bundler",
                "types": [], "lib": ["ES2022", "DOM"]
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = check_project(&tsgo, project, "tsconfig.json");
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let mut found = stdout
        .lines()
        .filter_map(|line| {
            let (at, rest) = line.split_once(": error ")?;
            Some(cstr!("{at} {}", rest.split_once(':')?.0))
        })
        .collect::<Vec<_>>();
    found.sort();
    assert_eq!(
        found,
        [
            "src/Named.vue(5,41) TS2339",
            "src/Named.vue(5,64) TS2339",
            "src/Only.vue(5,51) TS2339",
            "src/Only.vue(5,67) TS18048",
        ],
        "{}",
        output_text(&output)
    );
}
