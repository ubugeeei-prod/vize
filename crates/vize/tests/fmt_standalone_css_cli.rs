//! `vize fmt` discovers and formats standalone CSS files.
#![cfg(feature = "glyph")]

use std::{fs, path::Path, process::Command};

fn fmt(root: &Path, args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .arg("fmt")
        .arg("--no-config")
        .args(args)
        .output()
        .expect("run vize fmt")
}

#[test]
fn default_discovery_formats_css_and_leaves_preprocessor_files() {
    let root = tempfile::tempdir().expect("temporary project");
    let root = root.path();
    let unformatted = ".a{color:red}\n\n\n.b{margin:0}";
    let preprocessor = ".a{color:red}";
    fs::write(root.join("theme.css"), unformatted).expect("write css");
    fs::write(root.join("theme.scss"), preprocessor).expect("write scss");
    fs::write(root.join("theme.less"), preprocessor).expect("write less");

    let check = fmt(root, &["--check"]);
    assert_eq!(check.status.code(), Some(1), "{check:?}");
    assert_eq!(
        check.stderr,
        b"Found 1 file(s)\nWould reformat: theme.css\n\nChecked 1 file(s)\n  1 file(s) would be reformatted\n"
    );

    let write = fmt(root, &["--write"]);
    assert_eq!(write.status.code(), Some(0), "{write:?}");
    let formatted = ".a {\n  color: red;\n}\n\n.b {\n  margin: 0;\n}\n";
    assert_eq!(
        fs::read_to_string(root.join("theme.css")).expect("read css"),
        formatted
    );
    for file in ["theme.scss", "theme.less"] {
        assert_eq!(
            fs::read_to_string(root.join(file)).expect("read preprocessor file"),
            preprocessor
        );
    }

    let recheck = fmt(root, &["--check", "theme.css"]);
    assert_eq!(recheck.status.code(), Some(0), "{recheck:?}");
}
