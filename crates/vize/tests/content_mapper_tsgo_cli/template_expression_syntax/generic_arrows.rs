//! Public five-line TS arrow reproducer: native syntax and authored errors.
use super::super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, workspace_root,
};
use serde_json::json;
use std::path::PathBuf;
use vize_l0::String;

const ORIGINAL: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../tests/_fixtures/differential/typechecker/generic-arrow-tsx-projection/App.vue.txt"
));

#[test]
fn copied_generic_arrows_keep_native_syntax_and_authored_error_positions() {
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping unconfigured standard mapper host: {TSGO_ENV}");
        return;
    };
    assert!(host.is_file(), "{TSGO_ENV} must identify the actual host");
    let root = workspace_root().join("target/vize-tests/content-mapper-generic-arrows");
    std::fs::create_dir_all(&root).unwrap();
    let project = tempfile::Builder::new()
        .prefix("generic-arrows-")
        .tempdir_in(&root)
        .unwrap();
    install_mapper_manifest(project.path());
    install_vue_package(project.path());
    std::fs::create_dir(project.path().join("src")).unwrap();
    std::fs::write(
        project.path().join("tsconfig.json"),
        serde_json::to_vec_pretty(&json!({
            "compilerOptions": { "strict": true, "noEmit": true, "skipLibCheck": true,
                "target": "ESNext", "module": "preserve", "types": [], "jsx": "preserve" },
            "contentMappers": [{ "package": "vize", "extensions": [".vue"] }], "include": ["src"]
        }))
        .unwrap(),
    )
    .unwrap();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for (id, source, expected, status) in cases() {
        std::fs::write(project.path().join("src/App.vue"), source.as_bytes()).unwrap();
        let output = check_project(&host, project.path(), "tsconfig.json");
        observations.push(
            json!({ "id": id, "source": source, "status": output.status.code(),
            "stdoutBytes": output.stdout, "stderrBytes": output.stderr }),
        );
        verdicts.push((id, output, expected, status));
    }
    let vue = std::fs::read(project.path().join("node_modules/vue/package.json")).unwrap();
    let mapper = std::fs::read(project.path().join("node_modules/vize/package.json")).unwrap();
    let packet = serde_json::to_vec_pretty(
        &json!({ "schema": "vize.generic-arrow.native.v1", "host": host,
        "vueManifestBytes": vue, "mapperManifestBytes": mapper, "observations": observations }),
    )
    .unwrap();
    std::fs::write(root.join("observations.json"), &packet).unwrap();
    eprintln!("{}", std::str::from_utf8(&packet).unwrap());
    for (id, output, expected, status) in verdicts {
        assert_eq!(output.status.code(), Some(status), "{id}");
        assert_eq!(output.stdout.as_slice(), expected, "{id}");
        assert_eq!(output.stderr.as_slice(), b"", "{id}");
    }
}

fn cases() -> Vec<(&'static str, String, &'static [u8], i32)> {
    let bad_return = ORIGINAL.replace(": T => value;", ": number => 'bad';");
    let bad_count = ORIGINAL.replace("const count =", "const count: string =");
    let commented = ORIGINAL.replace("<T>", "<T /* , > inside comment */>");
    let nested = ORIGINAL.replace("const count = identity(1);", "const outer = <U>(value: U) => <V>(other: V) => [value, other];\nconst count = identity(1);\nvoid outer;");
    let with_ref = ORIGINAL.replace("const identity =", "import { useTemplateRef } from 'vue';\nconst element = useTemplateRef('target'); const identity =")
        .replace("const count =", "const count: string =")
        .replace("<template>{{ count }}</template>", "<template><span ref=\"target\">{{ count }}</span></template>");
    vec![
        ("original", ORIGINAL.into(), b"", 0),
        (
            "explicit-comma",
            ORIGINAL.replace("<T>", "<T,>").into(),
            b"",
            0,
        ),
        (
            "bad-return",
            bad_return.clone().into(),
            b"src/App.vue(2,43): error TS2322: Type 'string' is not assignable to type 'number'.\n",
            2,
        ),
        ("repair-return", ORIGINAL.into(), b"", 0),
        (
            "restore-bad-return",
            bad_return.into(),
            b"src/App.vue(2,43): error TS2322: Type 'string' is not assignable to type 'number'.\n",
            2,
        ),
        (
            "bad-count",
            bad_count.into(),
            b"src/App.vue(3,7): error TS2322: Type 'number' is not assignable to type 'string'.\n",
            2,
        ),
        ("repair-count", ORIGINAL.into(), b"", 0),
        (
            "comments-crlf",
            commented.replace('\n', "\r\n").into(),
            b"",
            0,
        ),
        ("nested", nested.into(), b"", 0),
        (
            "same-line-template-ref",
            with_ref.into(),
            b"src/App.vue(4,7): error TS2322: Type 'number' is not assignable to type 'string'.\n",
            2,
        ),
    ]
}
