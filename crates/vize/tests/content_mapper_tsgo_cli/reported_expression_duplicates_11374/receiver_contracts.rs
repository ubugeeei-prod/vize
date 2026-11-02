//! Synthetic contract controls; original seven reporter states stay unchanged.

use super::{
    PathBuf, TSGO_ENV, Value, check_project, install_mapper_manifest, install_vue_package, json,
    workspace_root, write,
};

#[test]
fn receiver_echoes_preserve_contextual_and_typed_fallthrough_contracts() {
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
    let mut states = input["states"].as_array().unwrap().clone();
    // A mutable binding is deliberately ineligible for independent receiver
    // ownership. These same-contract controls distinguish an existing unchecked
    // fallthrough route from a diagnostic hidden by the new receiver intervals.
    for original in input["states"].as_array().unwrap() {
        if original["id"]
            .as_str()
            .unwrap()
            .starts_with("typed-fallthrough-")
        {
            let mut state = original.clone();
            state["id"] = json!(format!("{}-mutable", original["id"].as_str().unwrap()));
            let parent = state["sources"]["Parent.vue"].as_str().unwrap();
            state["sources"]["Parent.vue"] =
                json!(parent.replace("const entries:", "let entries:"));
            states.push(state);
        }
    }
    for state in &states {
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
        let stdout = String::from_utf8(output.stdout).unwrap();
        let diagnostics = stdout
            .lines()
            .filter_map(|line| {
                let (_, suffix) = line.split_once(": error TS")?;
                let (code, _) = suffix.split_once(':')?;
                Some((line, code.parse::<u64>().unwrap()))
            })
            .collect::<Vec<_>>();
        let codes = diagnostics
            .iter()
            .map(|(_, code)| *code)
            .collect::<Vec<_>>();
        let expected = state["expectedCodes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|code| code.as_u64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(codes, expected, "{id}: {stdout}");
        if expected.is_empty() {
            assert_eq!(stdout, "", "{id}");
        } else {
            assert!(
                diagnostics
                    .iter()
                    .all(|(line, _)| line.starts_with("src/Parent.vue(")),
                "{id}: {stdout}"
            );
            assert!(
                stdout.contains(state["expectedMessage"].as_str().unwrap()),
                "{id}: {stdout}"
            );
        }
    }
}
