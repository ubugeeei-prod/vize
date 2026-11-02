//! A comment must not recheck an authored else-if inside its own narrowed branch.

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};
use serde_json::json;
use std::{fs, path::PathBuf};
use vize_canon::generate_vue_content_mapper_transform;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/commented-vif-11378/CommentInElseIf.vue.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/commented-vif-11378/tsconfig.json.txt"
));

#[test]
fn commented_else_if_checks_authored_condition_once() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert_ne!(
            std::env::var("VIZE_TEST_REQUIRE_COMMENTED_VIF").as_deref(),
            Ok("1"),
            "required commented else-if provider is missing"
        );
        eprintln!("skipping Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} is not a file");
    let root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("commented-vif-11378-")
        .tempdir_in(root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    let no_comment = ORIGINAL.replace("        // comment\n", "");
    assert_ne!(no_comment, ORIGINAL);
    fs::write(project.path().join("src/NoComment.vue"), &no_comment).unwrap();
    let wrong = ORIGINAL.replace(
        "props.item.kind !== 'b'",
        "props.item.kind !== 'b' && Number(0).nope",
    );
    assert_ne!(wrong, ORIGINAL);
    let expected_wrong = "src/CommentInElseIf.vue(13,46): error TS2339: Property 'nope' does not exist on type 'number'.\n";
    let capture = std::env::var_os("VIZE_COMMENTED_VIF_CAPTURE").map(PathBuf::from);
    if let Some(capture) = &capture {
        fs::create_dir_all(capture).unwrap();
    }
    let mut outcomes = Vec::new();
    for (phase, source, expected) in [
        ("original", ORIGINAL.to_owned(), ""),
        ("wrong-member", wrong.clone(), expected_wrong),
        ("repair", ORIGINAL.to_owned(), ""),
        ("restored-wrong", wrong, expected_wrong),
        ("restored-original", ORIGINAL.to_owned(), ""),
    ] {
        fs::write(project.path().join("src/CommentInElseIf.vue"), &source).unwrap();
        let transform = generate_vue_content_mapper_transform(
            std::path::Path::new("src/CommentInElseIf.vue"),
            &source,
        )
        .unwrap();
        let output = check_project(&native, project.path(), "tsconfig.json");
        let observation = json!({
            "phase":phase,"source":source,"noCommentSource":no_comment,"config":CONFIG,
            "cwd":project.path(),"program":native,
            "args":["--runExternalCode","--noEmit","-p","tsconfig.json","--pretty","false"],
            "transform":transform,"transformRole":"Independent default API companion, not a mapper-pipe capture",
            "status":output.status.code(),"success":output.status.success(),"signal":process_signal(&output.status),
            "expectedStdout":expected,"stdout":output.stdout,"stderr":output.stderr,
            "vueManifest":fs::read(project.path().join("node_modules/vue/package.json")).unwrap(),
            "mapperManifest":fs::read(project.path().join("node_modules/vize/package.json")).unwrap(),
            "scope":"Exact reported source/config and derived controls; actual hosted provider, not original macOS/package qualification"
        });
        println!("commented-vif-11378-observation {observation}");
        if let Some(capture) = &capture {
            fs::write(
                capture.join(format!("{phase}.json")),
                serde_json::to_vec_pretty(&observation).unwrap(),
            )
            .unwrap();
        }
        outcomes.push((phase, output, expected));
    }
    for (phase, output, expected) in outcomes {
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(
            output.stdout,
            expected.as_bytes(),
            "{phase}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{phase}: {}", output_text(&output));
    }
}

fn process_signal(status: &std::process::ExitStatus) -> Option<i32> {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        status.signal()
    }
    #[cfg(not(unix))]
    {
        let _ = status;
        None
    }
}
