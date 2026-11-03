//! Independent generic-scope and declaration-consumer reports #11450/#11454.

mod fallthrough;

use super::super::{
    TSGO_ENV, check_project, install_mapper_manifest, install_vue_package, workspace_root,
};
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::Output,
};
use vize_l0::String;

fn inputs() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/_fixtures/differential/typechecker/reported-generic-declarations-11450-11454/input.json"))).unwrap()
}

fn write(path: &Path, source: impl AsRef<[u8]>) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}

fn project(path: &Path, package: &str, config: &str) {
    fs::create_dir_all(path).unwrap();
    write(&path.join("package.json"), package);
    write(&path.join("tsconfig.json"), config);
    install_mapper_manifest(path);
    install_vue_package(path);
}

fn providers(path: &Path) -> Value {
    let manifests: Vec<_> = [
        path.join("node_modules/vue/package.json"),
        path.join("node_modules/vize/package.json"),
        workspace_root().join("tests/node_modules/vue-tsc/package.json"),
        workspace_root().join("tests/node_modules/typescript/package.json"),
    ].iter().map(|path| json!({"path": path, "realPath": fs::canonicalize(path).unwrap(), "bytes": fs::read(path).unwrap()})).collect();
    json!(manifests)
}

fn process(output: &Output) -> Value {
    json!({"status": output.status.code(), "stdout": output.stdout, "stderr": output.stderr})
}

fn retain(id: &str, host: &Path, observations: Value) {
    let path = workspace_root()
        .join("target/vize-tests/reported-generic-declarations")
        .join(id)
        .with_extension("json");
    let bytes = serde_json::to_vec_pretty(&json!({"schema":"vize.reported-generic-declarations.source.v1", "providerRole":"source-workspace-not-reporter-pin", "host":host, "observations":observations})).unwrap();
    write(&path, &bytes);
    eprintln!("{}", std::str::from_utf8(&bytes).unwrap());
}

#[test]
fn generic_dependents_preserve_authored_unused_and_value_diagnostics() {
    let Some(host) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        return;
    };
    assert!(host.is_file());
    let input = inputs();
    let original = input["genericSource"].as_str().unwrap();
    let config = input["genericConfig"].as_str().unwrap();
    let package = input["genericPackage"].as_str().unwrap();
    let sentinel = original.replace(
        "defineProps<Bar>();",
        "defineProps<Bar>();\nconst bad: number = \"x\"; void bad;",
    );
    let cases: Vec<(&str, String, String, &[u8])> = vec![
        ("original", original.into(), config.into(), b""),
        ("alias", original.replace("interface Bar {", "type Bar = {").replace("  items: Foo[];\n}", "  items: Foo[];\n};").into(), config.into(), b""),
        ("constrained", original.replace("generic=\"TBaz\"", "generic=\"TBaz extends string\"").into(), config.into(), b""),
        ("transitive", original.replace("interface Bar", "type Baz = Foo[];\ninterface Bar").replace("items: Foo[]", "items: Baz").into(), config.into(), b""),
        ("emits", original.replace("defineProps<Bar>();", "defineEmits<{ pick: [bar: Bar] }>();").into(), config.into(), b""),
        ("public-dependency", original.replace("interface Bar", "type Baz = Foo[];\nexport interface Bar").replace("items: Foo[]", "items: Baz").into(), config.into(), b""),
        ("authored-value-error", sentinel.clone().into(), config.into(), b"src/Probe.vue(8,7): error TS2322: Type 'string' is not assignable to type 'number'.\n"),
        ("repair-value-error", sentinel.replace("= \"x\";", "= 1;").into(), config.into(), b""),
        ("authored-unused-type", original.replace("defineProps<Bar>();", "void 0;").into(), config.into(), b"src/Probe.vue(4,11): error TS6196: 'Bar' is declared but never used.\n"),
        ("no-unused-disabled", original.into(), config.replace("\"noUnusedLocals\": true", "\"noUnusedLocals\": false").into(), b""),
    ];
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for (id, source, config, expected) in cases {
        let dir = tempfile::Builder::new()
            .prefix("reported-generic-11450-")
            .tempdir()
            .unwrap();
        project(dir.path(), package, config.as_str());
        write(&dir.path().join("src/Probe.vue"), source.as_bytes());
        let output = check_project(&host, dir.path(), "tsconfig.json");
        observations.push(json!({"id":id,"cwd":dir.path(),"source":source,"config":config,"package":package,"providers":providers(dir.path()),"result":process(&output)}));
        verdicts.push((id, output, expected));
    }
    retain("generic-11450", &host, json!(observations));
    for (id, output, expected) in verdicts {
        assert_eq!(
            output.status.code(),
            Some(if expected.is_empty() { 0 } else { 2 }),
            "{id}"
        );
        assert_eq!(output.stdout.as_slice(), expected, "{id}");
        assert_eq!(output.stderr.as_slice(), b"", "{id}");
    }
}
