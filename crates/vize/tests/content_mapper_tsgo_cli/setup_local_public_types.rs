//! `defineEmits`/`defineExpose` types naming setup-local aliases stay valid
//! through the module-level public aliases (misskey widgets).

use std::path::PathBuf;

use serde_json::json;

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

fn fixture(name: &str) -> CompactString {
    std::fs::read_to_string(
        workspace_root()
            .join(
                "tests/_fixtures/differential/typechecker/content-mapper-setup-local-public-types",
            )
            .join(name),
    )
    .unwrap()
    .into()
}

fn check(widget: &str, user: &str) -> std::process::Output {
    let tsgo = PathBuf::from(std::env::var_os(TSGO_ENV).unwrap());
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-setup-local-public-types-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    let src = project.join("src");
    std::fs::create_dir(&src).unwrap();
    std::fs::write(src.join("types.ts"), fixture("types.ts.txt").as_bytes()).unwrap();
    std::fs::write(src.join("Widget.vue"), widget).unwrap();
    std::fs::write(src.join("Use.vue"), user).unwrap();
    std::fs::write(
        project.join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": {
                "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ES2022", "module": "ESNext",
                "moduleResolution": "bundler", "types": []
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    check_project(&tsgo, project, "tsconfig.json")
}

#[test]
fn setup_local_emits_and_expose_types_keep_only_authored_errors() {
    if std::env::var_os(TSGO_ENV).is_none() {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    }
    let widget = fixture("Widget.vue.txt");
    let user = fixture("Use.vue.txt");
    let output = check(&widget, &user);
    assert_eq!(output.status.code(), Some(0), "{}", output_text(&output));
    assert_eq!(output.stdout, b"", "{}", output_text(&output));

    let widget = widget
        .replace("current: () => def", "current: () => 1")
        .replace("emit('update', def);\n", "emit('update', 1);\n")
        .replace("emit('update', def)\"", "emit('update', 2)\"");
    let user = user
        .replace("size: number =", "size: string =")
        .replace("value: { size: number;", "value: { size: string;");
    let output = check(&widget, &user);
    let stdout = std::str::from_utf8(&output.stdout).unwrap();
    let mut identities = stdout
        .lines()
        .filter_map(|line| {
            let (at, rest) = line.split_once(": error ")?;
            Some(cstr!("{at} {}", rest.split_once(":")?.0))
        })
        .collect::<Vec<_>>();
    identities.sort();
    assert_eq!(
        identities,
        [
            "src/Use.vue(5,7) TS2322",
            "src/Use.vue(9,33) TS2322",
            "src/Widget.vue(6,58) TS2322",
            "src/Widget.vue(7,16) TS2345",
            "src/Widget.vue(9,42) TS2345",
        ],
        "{}",
        output_text(&output)
    );
}
