//! Complete public checker vectors for the unchanged original #7874 project.
#[path = "support/corsa_requirement.rs"]
mod corsa_requirement;
#[path = "support/unknown_template_fixture.rs"]
mod fixture;
use serde_json::{Value, json};
use std::{
    path::{Path, PathBuf},
    process::Command,
};

#[test]
fn original_unknown_template_options_have_complete_true_false_and_absent_vectors() {
    let Some(corsa) = corsa_requirement::required_or_skip::<PathBuf>(
        std::env::var_os("CORSA_PATH").map(PathBuf::from),
    ) else {
        return;
    };
    let messages = fixture::messages(&corsa).unwrap();
    let full: Vec<_> = messages
        .iter()
        .zip([(2353, 8, 24), (2339, 9, 6), (2339, 10, 10)])
        .map(|(message, (code, line, column))| {
            vize_l0::cstr!("error:{line}:{column} [TS{code}] {message}")
        })
        .collect();
    let executable = Path::new(env!("CARGO_BIN_EXE_vize"));
    for (name, config, indexes) in fixture::cases().unwrap() {
        let directory = tempfile::tempdir().unwrap();
        let root = directory.path().canonicalize().unwrap();
        fixture::project(&root, &config).unwrap();
        let mut command = Command::new(executable);
        command.current_dir(&root).env("CORSA_PATH", &corsa).args([
            "check",
            "--no-config",
            "--format",
            "json",
        ]);
        for variable in [
            "VIZE_VUE_PACKAGE",
            "VIZE_VUE_NAMESPACE_PACKAGE",
            "VIZE_VUE_RUNTIME_DOM_PACKAGE",
            "VIZE_RUNTIME_NODE_MODULES",
        ] {
            command.env_remove(variable);
        }
        let output = command.output().unwrap();
        fixture::retain(&root, name, &corsa, &command, &output).unwrap();
        let expected: Vec<_> = indexes
            .iter()
            .map(|index| full.get(*index).unwrap())
            .collect();
        assert_eq!(
            output.status.code(),
            Some(i32::from(!expected.is_empty())),
            "{name}: {output:?}"
        );
        assert_eq!(output.stderr, Vec::<u8>::new(), "{name}: complete stderr");
        let actual: Value = serde_json::from_slice(&output.stdout).unwrap();
        assert_eq!(
            actual,
            json!({"fileCount":2,"errorCount":expected.len(),"warningCount":0,
                "programs":[{"root":".","tsconfig":"tsconfig.json",
                    "compilerOptions":serde_json::from_str::<Value>(fixture::CONFIG).unwrap().get("compilerOptions"),
                    "files":["src/Child.vue","src/Parent.vue"]}],"files":[
                {"file":"src/Child.vue","diagnostics":[]},{"file":"src/Parent.vue","diagnostics":expected}
            ]}),
            "{name}: whole public JSON report"
        );
        assert_eq!(
            std::fs::read(root.join("src/Child.vue")).unwrap(),
            fixture::CHILD.as_bytes()
        );
        assert_eq!(
            std::fs::read(root.join("src/Parent.vue")).unwrap(),
            fixture::PARENT.as_bytes()
        );
        assert_eq!(std::fs::read(root.join("tsconfig.json")).unwrap(), config);
    }
}
