//! Four contextual contract controls; the original seven states stay unchanged.
//! Optional fallthrough probes remain separate, explicitly unqualified input data.

use super::{
    PathBuf, TSGO_ENV, Value, check_project, install_mapper_manifest, install_vue_package, json,
    workspace_root, write,
};

#[test]
fn receiver_echoes_preserve_contextual_contracts() {
    let Some(tsgo) = std::env::var_os(TSGO_ENV).map(PathBuf::from) else {
        eprintln!("skipping receiver contract controls: {TSGO_ENV} is not set");
        return;
    };
    assert!(tsgo.is_absolute() && tsgo.is_file());
    let input: Value = serde_json::from_str(include_str!(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/reported-expression-duplicates-11374/receiver-controls.json"
    ))).unwrap();
    let cases_root = workspace_root().join("target/vize-tests/tests");
    std::fs::create_dir_all(&cases_root).unwrap();
    let mut observations = Vec::new();
    let mut verdicts = Vec::new();
    for state in input["states"].as_array().unwrap() {
        let id = state["id"].as_str().unwrap();
        let project = tempfile::Builder::new()
            .prefix("reported-receiver-contract-11374-")
            .tempdir_in(&cases_root)
            .unwrap();
        let root = project.path();
        write(root, "package.json", input["packageJson"].as_str().unwrap());
        write(root, "tsconfig.json", state["config"].as_str().unwrap());
        for (name, source) in state["sources"].as_object().unwrap() {
            write(root, &format!("src/{name}"), source.as_str().unwrap());
        }
        install_mapper_manifest(root);
        install_vue_package(root);
        let vue_manifest =
            std::fs::canonicalize(root.join("node_modules/vue/package.json")).unwrap();
        let output = check_project(&tsgo, root, "tsconfig.json");
        observations.push(json!({
            "id": id,
            "scope": input["scope"],
            "cwd": root,
            "sources": state["sources"],
            "config": state["config"],
            "package": input["packageJson"],
            "mapperManifest": std::fs::read(root.join("node_modules/vize/package.json")).unwrap(),
            "vueManifestPath": vue_manifest,
            "vueManifest": std::fs::read(&vue_manifest).unwrap(),
            "host": tsgo,
            "argv": ["--runExternalCode", "--noEmit", "-p", "tsconfig.json", "--pretty", "false"],
            "status": output.status.code(),
            "osStatus": output.status.to_string(),
            "stdout": output.stdout,
            "stderr": output.stderr,
            "expectedStatus": state["expectedStatus"],
            "expectedStdout": state["expectedStdout"],
            "expectedCodes": state["expectedCodes"],
            "expectedMessage": state["expectedMessage"],
        }));
        verdicts.push((state.clone(), output));
    }
    // Every full native process is retained before the first semantic assertion.
    eprintln!(
        "reported-receiver-contracts-11374: {}",
        serde_json::to_string(&observations).unwrap()
    );
    for (state, output) in verdicts {
        let id = state["id"].as_str().unwrap();
        assert_eq!(
            output.status.code().map(i64::from),
            state["expectedStatus"].as_i64(),
            "{id}"
        );
        assert_eq!(output.stderr, b"", "{id}");
        assert_eq!(
            output.stdout.as_slice(),
            state["expectedStdout"].as_str().unwrap().as_bytes(),
            "{id}"
        );
    }
}
