use std::{fs, path::Path, process::Command};

#[test]
fn sarif_output_has_precise_code_host_locations() {
    let directory = tempfile::tempdir().unwrap();
    let first = "<template><div id=\"共有🙂\" /></template>";
    write(directory.path(), "src/画面 #1.vue", first);
    write(
        directory.path(),
        "src/Other.vue",
        "<template><div id=\"共有🙂\" /></template>",
    );

    let output = doctor(directory.path(), &["src", "--format", "sarif"]);
    let sarif: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    let result = &sarif["runs"][0]["results"][0];
    let location = result["relatedLocations"]
        .as_array()
        .unwrap()
        .iter()
        .find(|location| {
            location["physicalLocation"]["artifactLocation"]["uri"]
                == "src/%E7%94%BB%E9%9D%A2%20%231.vue"
        })
        .unwrap();

    assert!(output.status.success());
    assert_eq!(sarif["version"], "2.1.0");
    assert_eq!(sarif["runs"][0]["columnKind"], "unicodeCodePoints");
    assert_eq!(
        location["physicalLocation"]["artifactLocation"]["uri"],
        "src/%E7%94%BB%E9%9D%A2%20%231.vue"
    );
    assert_eq!(
        location["physicalLocation"]["region"]["startColumn"],
        first[..first.find("id=\"共有🙂\"").unwrap()]
            .chars()
            .count()
            + 1
    );
}

fn doctor(root: &Path, arguments: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .arg("doctor")
        .args(arguments)
        .arg("--root")
        .arg(root)
        .output()
        .unwrap()
}

fn write(root: &Path, relative: &str, source: &str) {
    let path = root.join(relative);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, source).unwrap();
}
