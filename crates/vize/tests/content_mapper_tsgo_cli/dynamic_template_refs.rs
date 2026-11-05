//! Pin standard-tsgo acceptance for refs on component-typed dynamic targets (#10198).

use std::path::PathBuf;

use serde::Deserialize;

use super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, output_text,
    workspace_root,
};

const CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-dynamic-template-ref-10198/cases.json"
));

const UNION_CASES: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/content-mapper-dynamic-template-ref-union-10251/cases.json"
));

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    historical_issue: u32,
    cases: Vec<Case>,
}

#[derive(Deserialize)]
struct Case {
    id: String,
    inputs: Vec<Input>,
    diagnostics: Vec<String>,
}

#[derive(Deserialize)]
struct Input {
    file: String,
    source: String,
}

fn fixture_root(pack: &str) -> PathBuf {
    workspace_root()
        .join("tests/_fixtures/differential/typechecker")
        .join(pack)
}

#[test]
fn standard_tsgo_accepts_refs_on_component_typed_dynamic_targets() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let fixture: Fixture = serde_json::from_str(CASES).unwrap();
    assert_eq!(fixture.historical_issue, 10198);
    assert_eq!(fixture.cases.len(), 1, "retain both reported target shapes");
    for case in fixture.cases {
        assert!(
            case.diagnostics.is_empty(),
            "the reported project is accepted"
        );
        let cases_root = workspace_root().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases_root).unwrap();
        let project = tempfile::Builder::new()
            .prefix("content-mapper-dynamic-template-ref-")
            .tempdir_in(cases_root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for input in case.inputs {
            let target = project.path().join(input.file);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(
                fixture_root("content-mapper-dynamic-template-ref-10198").join(input.source),
                target,
            )
            .unwrap();
        }

        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        if !output.status.success() {
            let mut transforms = String::new();
            for source in ["src/MySwitch.vue", "src/MemberSwitch.vue"] {
                let path = project.path().join(source);
                let authored = std::fs::read_to_string(&path).unwrap();
                let transform =
                    vize_canon::generate_vue_content_mapper_transform(&path, &authored).unwrap();
                transforms.push_str(source);
                transforms.push_str(":\n");
                transforms.push_str(&transform.text);
                transforms.push('\n');
            }
            panic!(
                "{}: {}\nvirtual TypeScript:\n{}",
                case.id,
                output_text(&output),
                transforms
            );
        }
        assert_eq!(output.stdout, b"", "{}: {}", case.id, output_text(&output));
        assert_eq!(output.stderr, b"", "{}: {}", case.id, output_text(&output));
    }
}

#[test]
fn standard_tsgo_accepts_component_and_native_dynamic_ref_union() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");

    let fixture: Fixture = serde_json::from_str(UNION_CASES).unwrap();
    assert_eq!(fixture.historical_issue, 10251);
    assert_eq!(fixture.cases.len(), 1, "retain the reported target union");
    for case in fixture.cases {
        assert!(
            case.diagnostics.is_empty(),
            "the reported project is accepted"
        );
        let cases_root = workspace_root().join("target/vize-tests/tests");
        std::fs::create_dir_all(&cases_root).unwrap();
        let project = tempfile::Builder::new()
            .prefix("content-mapper-dynamic-template-ref-union-")
            .tempdir_in(cases_root)
            .unwrap();
        install_mapper_manifest(project.path());
        install_vue_package(project.path());
        for input in case.inputs {
            let target = project.path().join(input.file);
            std::fs::create_dir_all(target.parent().unwrap()).unwrap();
            std::fs::copy(
                fixture_root("content-mapper-dynamic-template-ref-union-10251").join(input.source),
                target,
            )
            .unwrap();
        }

        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        if !output.status.success() {
            let path = project.path().join("src/App.vue");
            let authored = std::fs::read_to_string(&path).unwrap();
            let transform =
                vize_canon::generate_vue_content_mapper_transform(&path, &authored).unwrap();
            panic!(
                "{}: {}\nvirtual TypeScript:\n{}",
                case.id,
                output_text(&output),
                transform.text,
            );
        }
        assert_eq!(output.stdout, b"", "{}: {}", case.id, output_text(&output));
        assert_eq!(output.stderr, b"", "{}: {}", case.id, output_text(&output));
    }
}

const CONFIG: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/static-template-ref-runtime-11377/tsconfig.json.txt"
));
const USE_BOX: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/static-template-ref-runtime-11377/useBox.ts.txt"
));
const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/static-template-ref-runtime-11377/StaticRef.vue.txt"
));

const UNINITIALIZED: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/static-template-ref-uninitialized-runtime-11377/StaticRef.vue.txt"
));

#[test]
fn standard_tsgo_static_template_refs_read_runtime_bindings_and_keep_true_unused() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping exact Content Mapper conformance: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_file(), "{TSGO_ENV} is not a file");
    let unused = ORIGINAL.replace("</script>", "const trueUnused = 1;\n</script>");
    let shadowed = ORIGINAL.replace(
        "<div ref=\"plain\">a</div>",
        "<div v-for=\"plain in [1]\" ref=\"plain\">a</div>",
    );
    let original_unused = "src/StaticRef.vue(7,7): error TS6133: 'trueUnused' is declared but its value is never read.\n";
    let shadow_unused =
        "src/StaticRef.vue(5,7): error TS6133: 'plain' is declared but its value is never read.\n";
    let uninitialized = ORIGINAL
        .replace(
            "const plain = ref<HTMLDivElement | null>(null);",
            "let plain: HTMLDivElement;",
        )
        .replace("import { ref } from \"vue\";", "");
    let authored_read = uninitialized.replace(
        "let plain: HTMLDivElement;",
        "let plain: HTMLDivElement;\nconsole.log(plain);",
    );
    let cases = [
        ("original", ORIGINAL, ""),
        ("authored-unused", unused.as_str(), original_unused),
        ("template-shadow", shadowed.as_str(), shadow_unused),
        ("uninitialized-render-read", UNINITIALIZED, ""),
        (
            "authored-uninitialized-read",
            authored_read.as_str(),
            "src/StaticRef.vue(6,13): error TS2454: Variable 'plain' is used before being assigned.\n",
        ),
        ("restored", ORIGINAL, ""),
    ];
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("content-mapper-static-runtime-ref-")
        .tempdir_in(cases_root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir_all(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    std::fs::write(project.path().join("src/useBox.ts"), USE_BOX).unwrap();
    let mut outputs = Vec::new();
    for (id, source, expected) in cases {
        std::fs::write(project.path().join("src/StaticRef.vue"), source).unwrap();
        let transform = vize_canon::generate_vue_content_mapper_transform_with_options(
            &project.path().join("src/StaticRef.vue"),
            source,
            vize_canon::ContentMapperTransformOptions::default()
                .with_preserve_unused_diagnostics(true),
        )
        .map_err(|error| error.to_string());
        let output = check_project(&tsgo, project.path(), "tsconfig.json");
        println!(
            "static template ref CLI: {}",
            serde_json::json!({
                "case":id,"program":&tsgo,"cwd":project.path(),"status":output.status.code(),
                "stdout":&output.stdout,"stderr":&output.stderr,
                "inputs":{"tsconfig.json":CONFIG,"src/useBox.ts":USE_BOX,"src/StaticRef.vue":source},
                "transform":&transform,
            })
        );
        outputs.push((id, expected, output, transform));
    }
    // Preserve all complete outcomes and generated source before the first verdict.
    assert_eq!(uninitialized, UNINITIALIZED);
    for (id, expected, output, transform) in outputs {
        assert!(transform.is_ok(), "{id}: {transform:?}");
        assert_eq!(
            output.status.success(),
            expected.is_empty(),
            "{id}: {}",
            output_text(&output)
        );
        assert_eq!(
            output.stdout,
            expected.as_bytes(),
            "{id}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{id}: {}", output_text(&output));
    }
}

#[test]
fn native_cli_static_template_refs_keep_unused_and_shadow_diagnostics() {
    use crate::support::corsa_requirement;
    use serde_json::{Value, json};
    use std::process::Command;

    let Some(runtime) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let parent = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&parent).unwrap();
    let project = tempfile::Builder::new()
        .prefix("native-static-runtime-ref-")
        .tempdir_in(parent)
        .unwrap();
    std::fs::create_dir(project.path().join("node_modules")).unwrap();
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(project.path().join("tsconfig.json"), CONFIG).unwrap();
    std::fs::write(project.path().join("src/useBox.ts"), USE_BOX).unwrap();
    let unused = ORIGINAL.replace("</script>", "const trueUnused = 1;\n</script>");
    let shadowed = ORIGINAL.replace(
        "<div ref=\"plain\">a</div>",
        "<div v-for=\"plain in [1]\" ref=\"plain\">a</div>",
    );
    let uninitialized = ORIGINAL
        .replace(
            "const plain = ref<HTMLDivElement | null>(null);",
            "let plain: HTMLDivElement;",
        )
        .replace("import { ref } from \"vue\";", "");
    let authored_read = uninitialized.replace(
        "let plain: HTMLDivElement;",
        "let plain: HTMLDivElement;\nconsole.log(plain);",
    );
    let cases = [
        ("original", ORIGINAL, Vec::<&str>::new()),
        (
            "authored-unused",
            unused.as_str(),
            vec!["error:7:7 [TS6133] 'trueUnused' is declared but its value is never read."],
        ),
        (
            "template-shadow",
            shadowed.as_str(),
            vec!["error:5:7 [TS6133] 'plain' is declared but its value is never read."],
        ),
        (
            "uninitialized-render-read",
            UNINITIALIZED,
            Vec::<&str>::new(),
        ),
        (
            "authored-uninitialized-read",
            authored_read.as_str(),
            vec!["error:6:13 [TS2454] Variable 'plain' is used before being assigned."],
        ),
        ("restored", ORIGINAL, Vec::<&str>::new()),
    ];
    let args = ["check", "src", "--no-config", "--quiet", "--format", "json"];
    let mut outcomes = Vec::new();
    for (id, source, messages) in cases {
        std::fs::write(project.path().join("src/StaticRef.vue"), source).unwrap();
        let output = Command::new(env!("CARGO_BIN_EXE_vize"))
            .current_dir(project.path())
            .env("CORSA_PATH", &runtime)
            .args(args)
            .output()
            .unwrap();
        let config: Value = serde_json::from_str(CONFIG).unwrap();
        let expected = json!({
            "files":[{"file":"src/StaticRef.vue","diagnostics":messages},{"file":"src/useBox.ts","diagnostics":[]}],
            "programs":[{"root":".","tsconfig":"tsconfig.json","compilerOptions":config["compilerOptions"],"files":["src/StaticRef.vue","src/useBox.ts"]}],
            "errorCount":messages.len(),"warningCount":0,"fileCount":2
        });
        let status = i32::from(!messages.is_empty());
        println!(
            "native static template ref CLI: {}",
            json!({
                "case":id,"binary":env!("CARGO_BIN_EXE_vize"),"argv":args,"runtime":runtime,"cwd":project.path(),
                "status":output.status.code(),"stdoutBytes":output.stdout,"stderrBytes":output.stderr,
                "inputs":{"tsconfig.json":CONFIG,"src/useBox.ts":USE_BOX,"src/StaticRef.vue":source},"expected":expected
            })
        );
        outcomes.push((id, status, expected, output));
    }
    // Every original/control process is retained before the first semantic assertion.
    assert_eq!(uninitialized, UNINITIALIZED);
    for (id, status, expected, output) in outcomes {
        assert_eq!(
            output.status.code(),
            Some(status),
            "{id}: {}",
            output_text(&output)
        );
        assert_eq!(output.stderr, b"", "{id}: {}", output_text(&output));
        assert_eq!(
            serde_json::from_slice::<Value>(&output.stdout).unwrap(),
            expected,
            "{id}: {}",
            output_text(&output)
        );
    }
}
