//! #13092: `openProject` answers before the first transform, so its cost must
//! not grow with the number of ordinary tsconfig inputs.
#![expect(clippy::unwrap_used, reason = "fixture assertions panic")]

use std::path::{Path, PathBuf};

use serde_json::{Value, json};
use vize_l0::{String as CompactString, cstr, path::canonicalize_non_verbatim};

use super::test_support::{exchange, frames, initialize_request, open_project_request};
use crate::commands::check::tsconfig_inputs::walk_probe;

const ORDINARY_INPUTS: usize = 256;

const VUE_TEMPLATE_REF: &str = "export {}\ndeclare global {\n  const useTemplateRef: typeof import(\"vue\").useTemplateRef\n}\n";

fn write(root: &Path, relative: &str, content: &str) {
    let path = root.join(relative);
    std::fs::create_dir_all(path.parent().unwrap()).unwrap();
    std::fs::write(path, content).unwrap();
}

fn write_ordinary_inputs(root: &Path, directory: &str) {
    for index in 0..ORDINARY_INPUTS {
        write(
            root,
            &cstr!("{directory}/m{index}.ts"),
            &cstr!("export const value{index}: number = {index};\n"),
        );
    }
}

/// Open `config` through the server and return the reply plus every file the
/// input walk examined past its directory entry.
fn open_project(root: &Path, config: &Path) -> (Value, Vec<PathBuf>) {
    let mut request = open_project_request(2, Value::Null, json!({}));
    request["params"]["configFileName"] = json!(config);
    walk_probe::observe(root);
    let responses = exchange(&frames(&[initialize_request(), request]));
    let examined = walk_probe::take(root);
    (responses[1]["result"].clone(), examined)
}

fn relative(root: &Path, files: &[PathBuf]) -> Vec<CompactString> {
    files
        .iter()
        .map(|path| {
            let mut relative = CompactString::default();
            for component in path.strip_prefix(root).unwrap().components() {
                if !relative.is_empty() {
                    relative.push('/');
                }
                relative.push_str(&component.as_os_str().to_string_lossy());
            }
            relative
        })
        .collect()
}

#[test]
fn open_project_examines_declarations_but_no_ordinary_input() {
    let directory = tempfile::tempdir().unwrap();
    let root = canonicalize_non_verbatim(directory.path());
    write_ordinary_inputs(&root, "src/gen");
    write(&root, "src/App.vue", "<template><div /></template>\n");
    write(&root, "src/env.d.ts", VUE_TEMPLATE_REF);
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"include":["src"]}"#).unwrap();

    let (result, examined) = open_project(&root, &config);

    // Ordinary inputs are rejected by their directory-entry name; only the
    // declaration is matched against the tsconfig patterns and then read.
    assert_eq!(relative(&root, &examined), vec!["src/env.d.ts"]);
    let identity = result["configIdentity"].as_str().unwrap();
    assert_eq!(identity.rsplit_once(':').unwrap().1, "1", "{identity}");
    let watched_sources = result["watchedFiles"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(Value::as_str)
        .filter(|path| Path::new(path).starts_with(root.join("src")))
        .collect::<Vec<_>>();
    assert_eq!(
        watched_sources,
        vec![root.join("src/env.d.ts").to_str().unwrap()]
    );
}

#[test]
fn root_with_own_inputs_keeps_its_references_closed_without_listing_them() {
    let directory = tempfile::tempdir().unwrap();
    let root = canonicalize_non_verbatim(directory.path());
    // The root names its own program but declares nothing, so whether `tsc -p`
    // expands its reference depends on whether it selects any input at all.
    write_ordinary_inputs(&root, "src");
    write(&root, "lib/env.d.ts", VUE_TEMPLATE_REF);
    write(&root, "lib/tsconfig.json", r#"{"include":["."]}"#);
    let config = root.join("tsconfig.json");
    std::fs::write(
        &config,
        r#"{"include":["src"],"references":[{"path":"./lib"}]}"#,
    )
    .unwrap();

    let (result, examined) = open_project(&root, &config);

    // The referenced project is a separate program, so its declaration does
    // not own the root's template refs.
    let identity = result["configIdentity"].as_str().unwrap();
    assert_eq!(identity.rsplit_once(':').unwrap().1, "0", "{identity}");
    // Finding one input answers the question; the walk stops there instead
    // of listing the root's inputs.
    let ordinary = examined
        .iter()
        .filter(|path| path.starts_with(root.join("src")))
        .count();
    assert!(
        ordinary >= 1 && ordinary < ORDINARY_INPUTS / 4,
        "examined {ordinary} of {ORDINARY_INPUTS} ordinary inputs"
    );
}

#[test]
fn solution_style_root_still_expands_into_referenced_declarations() {
    let directory = tempfile::tempdir().unwrap();
    let root = canonicalize_non_verbatim(directory.path());
    write_ordinary_inputs(&root, "app/src");
    write(&root, "app/src/env.d.ts", VUE_TEMPLATE_REF);
    write(&root, "app/tsconfig.json", r#"{"include":["src"]}"#);
    let config = root.join("tsconfig.json");
    std::fs::write(&config, r#"{"files":[],"references":[{"path":"./app"}]}"#).unwrap();

    let (result, examined) = open_project(&root, &config);

    let identity = result["configIdentity"].as_str().unwrap();
    assert_eq!(identity.rsplit_once(':').unwrap().1, "1", "{identity}");
    assert_eq!(relative(&root, &examined), vec!["app/src/env.d.ts"]);
}
