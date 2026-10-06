//! Lossless process/input custody for the native indexed-access fixture.
#![expect(clippy::disallowed_macros, reason = "fixture paths and error capture")]
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Output},
};

pub(super) fn write(root: &Path, name: &str, bytes: &[u8]) {
    let path = root.join(name);
    fs::create_dir_all(path.parent().unwrap()).unwrap();
    fs::write(path, bytes).unwrap();
}

pub(super) fn hash(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect()
}

pub(super) fn save(root: &Path, name: &str, value: &Value) {
    write(root, name, &serde_json::to_vec_pretty(value).unwrap());
}

pub(super) fn inputs(project: &Path, capture: &Path, names: &[&str]) {
    for name in names {
        let result = fs::read(project.join(name));
        if let Err(error) = &result {
            save(
                capture,
                "input-error.json",
                &json!({"path":name,"error":error.to_string()}),
            );
        }
        write(capture, name, &result.unwrap());
    }
}

pub(super) fn process(root: &Path, name: &str, command: &mut Command) -> Output {
    let result = command.output();
    let path = root.join(name);
    fs::create_dir_all(&path).unwrap();
    let argv = json!({
        "program": command.get_program().to_string_lossy(),
        "args": command.get_args().map(|arg| arg.to_string_lossy()).collect::<Vec<_>>(),
        "cwd": command.get_current_dir().map(|path| path.display().to_string())
    });
    match &result {
        Ok(output) => {
            write(&path, "stdout", &output.stdout);
            write(&path, "stderr", &output.stderr);
            save(
                &path,
                "process.json",
                &json!({"argv":argv,"status":output.status.code(),"success":output.status.success()}),
            );
        }
        Err(error) => save(
            &path,
            "process.json",
            &json!({"argv":argv,"error":error.to_string()}),
        ),
    }
    result.unwrap()
}

pub(super) fn symlink_vue(workspace: &Path, project: &Path) {
    let vue = workspace
        .join("playground/node_modules/vue")
        .canonicalize()
        .unwrap();
    fs::create_dir(project.join("node_modules")).unwrap();
    #[cfg(unix)]
    std::os::unix::fs::symlink(vue, project.join("node_modules/vue")).unwrap();
    #[cfg(windows)]
    std::os::windows::fs::symlink_dir(vue, project.join("node_modules/vue")).unwrap();
}

fn files(root: &Path, relative: &Path, output: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root.join(relative)).unwrap() {
        let entry = entry.unwrap();
        let name = relative.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            files(root, &name, output);
        } else {
            output.push(name);
        }
    }
}

pub(super) fn native_session(project: &Path, capture: &Path) -> Value {
    let root = project.join(".vize/patina");
    let entries = fs::read_dir(&root)
        .unwrap()
        .map(|entry| entry.unwrap())
        .collect::<Vec<_>>();
    save(
        capture,
        "native-session-entries.json",
        &json!(entries.iter().map(|entry| entry.path()).collect::<Vec<_>>()),
    );
    assert_eq!(entries.len(), 1, "one live fixture-owned native session");
    let session = entries.first().unwrap().path();
    let mut names = Vec::new();
    files(&session, Path::new(""), &mut names);
    names.sort();
    let rows = names
        .iter()
        .map(|name| {
            let bytes = fs::read(session.join(name)).unwrap();
            write(
                &capture.join("native-session"),
                &name.to_string_lossy(),
                &bytes,
            );
            json!({"path":name.to_string_lossy(),"bytes":bytes.len(),"sha256":hash(&bytes)})
        })
        .collect::<Vec<_>>();
    save(
        capture,
        "native-session.json",
        &json!({"session":session,"files":rows}),
    );
    serde_json::from_slice(&fs::read(session.join("tsconfig.json")).unwrap()).unwrap()
}

pub(super) fn expected_session(flag: &Value) -> Value {
    let mut value = json!({
        "compilerOptions": {
            "target":"ES2022", "module":"ESNext", "moduleResolution":"bundler",
            "allowImportingTsExtensions":true, "lib":["ES2022","DOM","DOM.Iterable"],
            "rootDirs":[".","../../.."], "strict":true, "noEmit":true, "skipLibCheck":true,
            "types":[]
        },
        "include":["**/*.patina.ts"]
    });
    if !flag.is_null() {
        value["compilerOptions"]["noUncheckedIndexedAccess"] = flag.clone();
    }
    value
}
