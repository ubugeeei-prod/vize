//! Template-usage anchors must not assert definite assignment of setup `let`s
//! assigned only in callbacks (misskey MkMediaAudio TS2454).

use std::path::PathBuf;

use serde_json::json;

use super::{
    CompactString, TSGO_ENV, check_project, cstr, install_mapper_manifest, install_vue_package,
    output_text, workspace_root,
};

fn identities(no_unused_locals: bool) -> (Vec<CompactString>, std::process::Output) {
    let tsgo = PathBuf::from(std::env::var_os(TSGO_ENV).unwrap());
    let fixture = workspace_root()
        .join("tests/_fixtures/differential/typechecker/content-mapper-let-anchor-assignment");
    let root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-let-anchor-assignment-")
        .tempdir_in(root)
        .unwrap();
    let project = project.path();
    install_mapper_manifest(project);
    install_vue_package(project);
    std::fs::create_dir(project.join("src")).unwrap();
    for name in ["Media", "Bad"] {
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
                "types": [], "lib": ["ES2022", "DOM"], "noUnusedLocals": no_unused_locals
            },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }],
            "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let output = check_project(&tsgo, project, "tsconfig.json");
    let mut found = std::str::from_utf8(&output.stdout)
        .unwrap()
        .lines()
        .filter_map(|line| {
            let (at, rest) = line.split_once(": error ")?;
            Some(cstr!("{at} {}", rest.split_once(':')?.0))
        })
        .collect::<Vec<_>>();
    found.sort();
    (found, output)
}

#[test]
fn callback_assigned_lets_keep_only_authored_errors() {
    if std::env::var_os(TSGO_ENV).is_none() {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    }
    for (no_unused_locals, expected) in [
        (
            false,
            &[
                "src/Bad.vue(3,14) TS2454",
                "src/Media.vue(15,98) TS7005",
                "src/Media.vue(7,5) TS7034",
            ][..],
        ),
        (
            true,
            &[
                "src/Bad.vue(3,14) TS2454",
                "src/Bad.vue(4,5) TS6133",
                "src/Media.vue(15,98) TS7005",
                "src/Media.vue(7,5) TS7034",
            ][..],
        ),
    ] {
        let (found, output) = identities(no_unused_locals);
        assert_eq!(found, expected, "{}", output_text(&output));
    }
}
