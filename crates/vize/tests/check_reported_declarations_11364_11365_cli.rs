//! Source CLI declaration controls for the original #11364 and #11365 inputs.
#![cfg(test)]
use crate::support::corsa_requirement;
use serde_json::{Value, json};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};
use vize_l0::cstr;

fn root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
}
fn fixture() -> Value {
    serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"), "/../../tests/_fixtures/differential/typechecker/reported-declarations-11364-11365/input.json")))
    .unwrap()
}
fn write(path: &Path, bytes: impl AsRef<[u8]>) {
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}
fn project(path: &Path, config: &str, package: &str) {
    fs::create_dir_all(path).unwrap();
    write(&path.join("tsconfig.json"), config);
    write(&path.join("package.json"), package);
    let provider = root().join("tests/node_modules");
    assert!(
        provider.join("vue/package.json").is_file(),
        "actual Vue provider required"
    );
    #[cfg(unix)]
    std::os::unix::fs::symlink(provider, path.join("node_modules")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(provider, path.join("node_modules")).unwrap();
}
fn save_outputs(directory: &Path) -> Value {
    let mut entries = Vec::new();
    if directory.exists() {
        let mut pending = vec![directory.to_path_buf()];
        while let Some(dir) = pending.pop() {
            for entry in fs::read_dir(dir).unwrap() {
                let entry = entry.unwrap();
                if entry.file_type().unwrap().is_dir() {
                    pending.push(entry.path());
                } else {
                    let bytes = fs::read(entry.path()).unwrap();
                    entries.push(json!({"file": entry.path().strip_prefix(directory).unwrap(), "text": std::str::from_utf8(&bytes).unwrap(), "bytes": bytes}));
                }
            }
        }
    }
    entries.sort_by(|a, b| a["file"].as_str().cmp(&b["file"].as_str()));
    json!(entries)
}
fn observe(capture: &Path, id: &str, output: &Output, project: &Path, input: Value) {
    let value = json!({"id": id, "input": input, "cwd": project,
        "status": output.status.code(), "osStatus": cstr!("{}", output.status), "stdout": output.stdout, "stderr": output.stderr,
        "outputs": save_outputs(&project.join("out")),
        "vueManifestPath": fs::canonicalize(project.join("node_modules/vue/package.json")).unwrap(), "vueManifest": fs::read_to_string(project.join("node_modules/vue/package.json")).unwrap()});
    write(
        &capture.join(cstr!("{id}.json").as_str()),
        serde_json::to_vec_pretty(&value).unwrap(),
    );
}
fn diagnostic_codes(output: &Output) -> Vec<u32> {
    if let Ok(value) = serde_json::from_slice::<Value>(&output.stdout) {
        if let Some(files) = value["files"].as_array() {
            return files
                .iter()
                .flat_map(|file| file["diagnostics"].as_array().unwrap())
                .map(|row| {
                    let text = row.as_str().expect("CLI diagnostic string");
                    text.split_once("[TS")
                        .unwrap()
                        .1
                        .split_once(']')
                        .unwrap()
                        .0
                        .parse()
                        .unwrap()
                })
                .collect();
        }
    }
    output
        .stdout
        .split(|byte| *byte == b'\n')
        .chain(output.stderr.split(|byte| *byte == b'\n'))
        .filter_map(|line| {
            let line = std::str::from_utf8(line).unwrap();
            let (_, tail) = line.split_once("error TS")?;
            tail.split_once(':')?.0.parse().ok()
        })
        .collect()
}
fn capture() -> PathBuf {
    std::env::var_os("VIZE_REPORTED_DECLARATIONS_CAPTURE")
        .map(PathBuf::from)
        .unwrap_or_else(|| root().join("target/vize-tests/reported-declarations-capture"))
}
fn emit(project: &Path, corsa: &Path, sources: &[&str], json_format: bool) -> Output {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(project)
        .env("CORSA_PATH", corsa)
        .arg("check");
    if json_format {
        command.args(["--format", "json"]);
    }
    command
        .args(["--declaration", "--declaration-dir", "out"])
        .args(sources)
        .output()
        .unwrap()
}

#[test]
fn repeated_component_declarations_preserve_no_unused_and_authored_errors() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(None) else {
        return;
    };
    let fixture = fixture();
    let config = fixture["noUnusedConfig"].as_str().unwrap();
    let child = fixture["repeatedChild"].as_str().unwrap();
    let original = fixture["repeatedConsumer"].as_str().unwrap();
    assert_eq!(
        original,
        include_str!(
            "../../../tests/_fixtures/differential/typechecker/qualification-20261105/Repeated.vue.txt"
        )
    );
    let cases = [
        (
            "original-text-repeated",
            config.to_owned(),
            original.to_owned(),
            true,
        ),
        (
            "original-repeated",
            config.to_owned(),
            original.to_owned(),
            true,
        ),
        (
            "one-use",
            config.to_owned(),
            original.replace("  <Foo :bar=\"2\" />\n", ""),
            true,
        ),
        (
            "no-unused-false",
            config.replace("\"noUnusedLocals\": true", "\"noUnusedLocals\": false"),
            original.to_owned(),
            true,
        ),
        (
            "wrong-owned-prop",
            config.to_owned(),
            original.replace(":bar=\"2\"", "bar=\"wrong\""),
            false,
        ),
        (
            "authored-unused",
            config.to_owned(),
            original.replace("import Foo", "const authoredUnused = 1;\nimport Foo"),
            false,
        ),
        (
            "repaired-repeated",
            config.to_owned(),
            original.to_owned(),
            true,
        ),
    ];
    let mut results = Vec::new();
    for (id, config, consumer, clean) in cases {
        let dir = tempfile::Builder::new()
            .prefix("reported-declarations-11365-")
            .tempdir()
            .unwrap();
        project(
            dir.path(),
            &config,
            fixture["noUnusedPackageJson"].as_str().unwrap(),
        );
        write(&dir.path().join("src/Foo.vue"), child);
        write(&dir.path().join("src/Probe.vue"), &consumer);
        let output = emit(
            dir.path(),
            &corsa,
            &["src/Foo.vue", "src/Probe.vue"],
            id != "original-text-repeated",
        );
        observe(
            &capture(),
            id,
            &output,
            dir.path(),
            json!({"config": config,"Foo.vue":child,"Probe.vue":consumer,"corsa":corsa}),
        );
        let helpers = dir.path().join("out/__vize_helpers.d.ts").is_file();
        let outputs = save_outputs(&dir.path().join("out"));
        results.push((id, clean, output, helpers, outputs));
    }
    for (id, clean, output, helpers, outputs) in results {
        let all = cstr!(
            "{}\n{}",
            std::str::from_utf8(&output.stdout).unwrap(),
            std::str::from_utf8(&output.stderr).unwrap()
        );
        assert_eq!(
            output.status.code(),
            Some(if clean { 0 } else { 1 }),
            "{id}: {all}"
        );
        let expected_codes = if clean {
            vec![]
        } else if id == "wrong-owned-prop" {
            vec![2322]
        } else {
            vec![6133]
        };
        assert_eq!(diagnostic_codes(&output), expected_codes, "{id}: {all}");
        if clean {
            assert!(helpers, "{id}: full helper module absent");
            assert_eq!(
                serde_json::to_string(&outputs)
                    .unwrap()
                    .matches("vize:virtual-ts")
                    .count(),
                0,
                "{id}: private specifier leaked"
            );
            assert_eq!(
                outputs
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|row| row["file"].as_str().unwrap())
                    .collect::<Vec<_>>(),
                vec!["Foo.vue.d.ts", "Probe.vue.d.ts", "__vize_helpers.d.ts"]
            );
        } else {
            let json: Value = serde_json::from_slice(&output.stdout).unwrap();
            let actual = json["files"]
                .as_array()
                .unwrap()
                .iter()
                .flat_map(|file| file["diagnostics"].as_array().unwrap())
                .cloned()
                .collect::<Vec<_>>();
            let expected = if id == "wrong-owned-prop" {
                "error:7:8 [TS2322] Type 'string' is not assignable to type 'number'. (source: <Foo bar=\"wrong\" />; binding: bar)"
            } else {
                "error:2:7 [TS6133] 'authoredUnused' is declared but its value is never read."
            };
            assert_eq!(actual, vec![json!(expected)], "{id}: {all}");
            assert_eq!(outputs, json!([]));
        }
    }
}
