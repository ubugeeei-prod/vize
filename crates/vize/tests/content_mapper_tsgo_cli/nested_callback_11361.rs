//! Q16solver #11361: inference-only callback copies cannot own nested errors.

use std::{fs, path::PathBuf};

use serde_json::json;
use vize_canon::generate_vue_content_mapper_transform;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-callback-inference-11361/Probe.vue.txt"
));
const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-callback-inference-11361/tsconfig.json.txt"
));
const PACKAGE: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/nested-callback-inference-11361/package.json.txt"
));

#[test]
fn nested_callback_keeps_original_acceptance_and_checked_body_errors() {
    let Some(native) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        assert_ne!(
            std::env::var("VIZE_TEST_REQUIRE_NESTED_CALLBACK").as_deref(),
            Ok("1"),
            "required nested callback Content Mapper provider is missing"
        );
        eprintln!("skipping Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(native.is_file(), "{TSGO_ENV} is not a file");
    let root = workspace_root().join("target/vize-tests/tests");
    fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("nested-callback-11361-")
        .tempdir_in(root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    fs::create_dir(project.path().join("src")).unwrap();
    fs::write(project.path().join("package.json"), PACKAGE).unwrap();
    fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    let capture = std::env::var_os("VIZE_NESTED_CALLBACK_CAPTURE").map(PathBuf::from);
    if let Some(capture) = &capture {
        fs::create_dir_all(capture).unwrap();
    }
    let bad_member = ORIGINAL.replace("console.log(baz)", "baz.nope()");
    let bad_argument = ORIGINAL.replace("console.log(baz)", "baz.toFixed('wrong')");
    let outer_member = ORIGINAL.replace("        foo.bar", "        foo.nope();\n        foo.bar");
    let expression_body = ORIGINAL.replace(
        "(foo) => {\n        foo.bar((baz) => console.log(baz));\n      }",
        "(foo) => foo.bar((baz) => baz)",
    );
    let mut outcomes = Vec::new();
    for (phase, source, expected) in [
        ("original", ORIGINAL.to_owned(), ""),
        ("expression-body", expression_body, ""),
        (
            "outer-member",
            outer_member,
            "src/Probe.vue(15,13): error TS2339: Property 'nope' does not exist on type 'Foo'.\n",
        ),
        (
            "nested-member",
            bad_member.clone(),
            "src/Probe.vue(15,30): error TS2339: Property 'nope' does not exist on type 'number'.\n",
        ),
        ("repair", ORIGINAL.to_owned(), ""),
        (
            "nested-argument",
            bad_argument,
            "src/Probe.vue(15,38): error TS2345: Argument of type 'string' is not assignable to parameter of type 'number'.\n",
        ),
        (
            "restored-member",
            bad_member,
            "src/Probe.vue(15,30): error TS2339: Property 'nope' does not exist on type 'number'.\n",
        ),
        ("restored-original", ORIGINAL.to_owned(), ""),
    ] {
        fs::write(project.path().join("src/Probe.vue"), &source).unwrap();
        let transform =
            generate_vue_content_mapper_transform(std::path::Path::new("src/Probe.vue"), &source)
                .unwrap();
        let output = check_project(&native, project.path(), "tsconfig.json");
        let observation = json!({
            "phase":phase,"source":source,"package":PACKAGE,"config":CONFIG,
            "cwd":project.path(),"program":native,
            "args":["--runExternalCode","--noEmit","-p","tsconfig.json","--pretty","false"],
            "transform":transform,"status":output.status.code(),"success":output.status.success(),
            "signal":process_signal(&output.status),"expectedStdout":expected,
            "stdout":output.stdout,"stderr":output.stderr,
            "vueManifest":fs::read(project.path().join("node_modules/vue/package.json")).unwrap(),
            "mapperManifest":fs::read(project.path().join("node_modules/vize/package.json")).unwrap(),
            "scope":"Source-owned original input; actual hosted provider, not reporter runtime qualification"
        });
        // Every complete source/process packet survives before any semantic verdict.
        println!("nested-callback-11361-observation {observation}");
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
