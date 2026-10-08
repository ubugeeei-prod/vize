//! Original manifestless package roots and input-set transitions conserve raw files.

use super::*;

fn receipt(path: &Path) -> Value {
    fn visit(root: &Path, path: &Path, rows: &mut Vec<Value>) {
        let relative = path.strip_prefix(root).unwrap().as_os_str();
        let metadata = match std::fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) => {
                rows.push(
                    json!({"path":relative,"kind":"metadata-error","error":error.to_string()}),
                );
                return;
            }
        };
        if metadata.file_type().is_symlink() {
            let target = std::fs::read_link(path).unwrap();
            rows.push(json!({"path":relative,"kind":"symlink","target":target.as_os_str()}));
        } else if metadata.is_dir() {
            rows.push(json!({"path":relative,"kind":"directory"}));
            let mut entries = std::fs::read_dir(path)
                .unwrap()
                .map(|entry| entry.unwrap().path())
                .collect::<Vec<_>>();
            entries.sort();
            for entry in entries {
                visit(root, &entry, rows);
            }
        } else {
            assert!(metadata.is_file());
            rows.push(json!({"path":relative,"kind":"file",
                "bytes":std::fs::read(path).unwrap(),"readonly":metadata.permissions().readonly()}));
        }
    }
    let mut rows = Vec::new();
    visit(path, path, &mut rows);
    json!(rows)
}

pub(super) fn guarded_packet(
    name: &str,
    phase: &str,
    mode: &str,
    root: &Path,
    command: &mut Command,
) -> Value {
    let package = root.join("node_modules");
    let before = receipt(&package);
    let sentinel = root.join("raw-sentinel");
    let raw_before = std::fs::symlink_metadata(&sentinel)
        .is_ok()
        .then(|| receipt(&sentinel));
    let whole = packet(command);
    let after = receipt(&package);
    let raw_after = (raw_before.is_some() || std::fs::symlink_metadata(&sentinel).is_ok())
        .then(|| receipt(&sentinel));
    if let Some(capture) = std::env::var_os("VIZE_TSCONFIG_TYPES_CAPTURE") {
        let dir = PathBuf::from(capture).join("ignore-literal").join(name);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(
            dir.join(cstr!("{phase}-{mode}.json").as_str()),
            serde_json::to_vec_pretty(
                &json!({"wholeInput":serde_json::from_str::<Value>(INPUT).unwrap(),
            "sourceSha":std::env::var("SOURCE_SHA").ok(),"projectRoot":root,
            "program":command.get_program(),"args":command.get_args().collect::<Vec<_>>(),
            "wholePacket":whole,"packageBefore":before,"packageAfter":after,
            "rawSentinelBefore":raw_before,"rawSentinelAfter":raw_after}),
            )
            .unwrap(),
        )
        .unwrap();
    }
    assert_eq!(
        after, before,
        "whole raw package/link conservation {name}/{phase}/{mode}"
    );
    assert_eq!(
        raw_after, raw_before,
        "whole raw endpoint conservation {name}/{phase}/{mode}"
    );
    whole
}

fn project(name: &str, corpus: &Value) -> PathBuf {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .join("target/vize-tests/tests")
        .join(cstr!("literal-package-{name}-{}", std::process::id()).as_str());
    assert!(!root.exists());
    write(&root, &corpus["commonFiles"]);
    write(&root, &corpus["cases"]["literal-dependency"]["files"]);
    std::fs::canonicalize(root).unwrap()
}

fn stock(name: &str, phase: &str, root: &Path, native: &Path, case: &Value) {
    let config = guarded_packet(
        name,
        phase,
        "show-config",
        root,
        Command::new(native)
            .current_dir(root)
            .args(["--project", "tsconfig.json", "--showConfig"]),
    );
    assert_eq!(config, case["officialShowConfig"]["typescript-7.0.2"]);
    let result = guarded_packet(
        name,
        phase,
        "stock",
        root,
        Command::new(native).current_dir(root).args([
            "--project",
            "tsconfig.json",
            "--pretty",
            "false",
        ]),
    );
    assert_eq!(
        result,
        case["officialWholeOutputs"]["typescript-7.0.2"][phase]
    );
}

fn check(name: &str, phase: &str, root: &Path, native: &Path, case: &Value, explicit: bool) {
    let mut command = Command::new(env!("CARGO_BIN_EXE_vize"));
    command
        .current_dir(root)
        .env("CORSA_PATH", native)
        .args(["check", "--quiet", "--format", "json"]);
    if explicit {
        for file in case["reportedFiles"].as_array().unwrap() {
            command.arg(file.as_str().unwrap());
        }
    }
    let whole = guarded_packet(
        name,
        phase,
        if explicit { "explicit" } else { "default" },
        root,
        &mut command,
    );
    assert_eq!(whole["exitCode"], i32::from(phase == "broken"), "{whole}");
    assert_eq!(whole["stderr"], "", "{whole}");
    let value: Value = serde_json::from_str(whole["stdout"].as_str().unwrap()).unwrap();
    assert_eq!(
        value,
        expected(case, phase == "broken"),
        "whole {name}/{phase}"
    );
}

fn native() -> Option<PathBuf> {
    let native = corsa_requirement::required_or_skip(None::<PathBuf>)?;
    assert_eq!(
        packet(Command::new(&native).arg("--version")),
        json!({"exitCode":0,"stdout":"Version 7.0.2\n","stderr":""})
    );
    Some(native)
}

#[test]
fn fresh_explicit_literal_package_preserves_the_original_seven_root_program_and_repair() {
    let Some(native) = native() else {
        return;
    };
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let name = "fresh-explicit";
    let root = project(name, &corpus);
    for phase in ["clean", "broken", "repair"] {
        write(
            &root,
            if phase == "broken" {
                &corpus["brokenFiles"]
            } else {
                &corpus["commonFiles"]
            },
        );
        let case = &corpus["cases"]["literal-dependency"];
        stock(name, phase, &root, &native, case);
        check(name, phase, &root, &native, case, true);
    }
    std::fs::remove_dir_all(root).unwrap();
    eprintln!("complete fresh explicit literal package: 3 public native CLI invocations accepted");
}

#[cfg(unix)]
#[test]
fn narrowed_then_expanded_literal_package_keeps_owned_writes_off_raw_package_and_symlink_paths() {
    let Some(native) = native() else {
        return;
    };
    let corpus: Value = serde_json::from_str(INPUT).unwrap();
    let boundary:Value=serde_json::from_str(include_str!(concat!(env!("CARGO_MANIFEST_DIR"),
        "/../../tests/_fixtures/differential/typechecker/tsconfig-ignore-3984/literal-package-boundary.json"))).unwrap();
    let root = project("narrowed-expanded", &corpus);
    write(&root, &boundary["files"]);
    for (path, target) in boundary["links"].as_object().unwrap() {
        std::os::unix::fs::symlink(target.as_str().unwrap(), root.join(path)).unwrap();
    }
    for phase in ["clean", "broken", "repair"] {
        write(
            &root,
            if phase == "broken" {
                &corpus["brokenFiles"]
            } else {
                &corpus["commonFiles"]
            },
        );
        for (name, case_name) in [("narrowed", "direct"), ("expanded", "literal-dependency")] {
            let case = &corpus["cases"][case_name];
            write(&root, &case["files"]);
            stock(name, phase, &root, &native, case);
            check(name, phase, &root, &native, case, false);
        }
    }
    std::fs::remove_dir_all(root).unwrap();
    eprintln!(
        "complete narrowed-expanded literal package: 6 public native CLI invocations accepted"
    );
}
