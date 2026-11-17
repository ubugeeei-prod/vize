#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `vize build` routes `.tsrx` modules through `vize_l4::lang::tsrx::compile`:
//! the written module is the entry's output byte for byte, and a refused
//! module is a compile error that never gets compiled code.

use std::{fs, path::Path, process::Command};
use vize_l4::lang::tsrx::{CompileOptions, Target, compile};

const BADGE: &str = "export function Badge({ text }: { text: string }) @{\n  <span class=\"badge\">{text}</span>\n}\n";
const ARROW: &str = "export const arrow = () => <span>hi</span>;\n";
const APP: &str = "<template>\n  <div>ok</div>\n</template>\n";
const OUTSIDE: &str = "1:28 [unsupported] OutsideComponent is not emitted for the DOM target yet";

fn build(root: &Path, args: &[&str]) -> (Option<i32>, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .env("RAYON_NUM_THREADS", "1")
        .args(["build", "--no-config", "--slow-threshold", "600000"])
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8(output.stdout).unwrap();
    let stderr = String::from_utf8(output.stderr).unwrap();
    let (head, time) = stderr.rsplit_once(" compiled in ").unwrap();
    let (_, tail) = time.split_once('s').unwrap();
    (
        output.status.code(),
        stdout,
        format!("{head} compiled in <TIME>s{tail}"),
    )
}

fn refused(file: &str, lines: &[&str], compiled: usize) -> String {
    let lines: String = lines.iter().map(|line| format!("      {line}\n")).collect();
    format!(
        "\n\x1b[31m✗ 1 error(s) occurred:\x1b[0m\n\n  \x1b[31mCompile errors (1):\x1b[0m\n    \x1b[1m{file}\x1b[0m\n{lines}\n\x1b[31m✗ 1 file(s) failed\x1b[0m, {compiled} compiled in <TIME>s\n"
    )
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    for (name, source) in files {
        fs::write(project.path().join(name), source).unwrap();
    }
    project
}

#[test]
fn a_dom_module_is_written_as_the_compile_entry_emits_it() {
    // The default `--script-ext downcompile` erases types; `preserve` keeps them.
    for (args, written, erase_types) in [
        (&["Badge.tsrx"][..], "Badge.js", true),
        (
            &["--script-ext", "preserve", "Badge.tsrx"][..],
            "Badge.ts",
            false,
        ),
    ] {
        let project = project(&[("Badge.tsrx", BADGE)]);
        let (code, stdout, stderr) = build(project.path(), args);
        assert_eq!(code, Some(0), "{stderr}");
        assert_eq!(stdout, "");
        assert_eq!(
            stderr,
            format!(
                "Built: Badge.tsrx -> ./dist/{written}\n\x1b[32m✓ 1 file compiled in <TIME>s\x1b[0m\n"
            )
        );
        let options = CompileOptions {
            filename: "Badge.tsrx",
            target: Target::Dom,
            source_map: false,
            erase_types,
        };
        assert_eq!(
            fs::read_to_string(project.path().join("dist").join(written)).unwrap(),
            compile(BADGE, &options).unwrap().code.as_str()
        );
    }
}

#[test]
fn refused_targets_and_options_write_nothing() {
    let project = project(&[("Badge.tsrx", BADGE), ("Arrow.tsrx", ARROW)]);
    for (args, file, line) in [
        (
            &["--ssr", "Badge.tsrx"][..],
            "Badge.tsrx",
            "1:1 [unsupported] Ssr emission of TSRX modules is not supported yet",
        ),
        (&["Arrow.tsrx"][..], "Arrow.tsrx", OUTSIDE),
    ] {
        let (code, stdout, stderr) = build(project.path(), args);
        assert_eq!(code, Some(1), "{args:?}");
        assert_eq!(stdout, "", "{args:?}");
        assert_eq!(stderr, refused(file, &[line], 0), "{args:?}");
        assert!(!project.path().join("dist").exists(), "{args:?}");
    }
}

#[test]
fn syntax_errors_keep_their_code_line_and_column() {
    let project = project(&[(
        "Notice.tsrx",
        "export function Notice() @{\n  <div>Notice</span>\n}\n",
    )]);
    let (code, stdout, stderr) = build(project.path(), &["Notice.tsrx"]);
    assert_eq!(code, Some(1));
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        refused(
            "Notice.tsrx",
            &[
                "2:14 [TSRX1002] Expected closing tag to match opening tag. Expected '</div>' but found '</span>'"
            ],
            0
        )
    );
    assert!(!project.path().join("dist").exists());
}

#[test]
fn a_directory_builds_its_sfcs_and_tsrx_modules_and_refuses_unsupported_ones() {
    let project = project(&[
        ("App.vue", APP),
        ("Badge.tsrx", BADGE),
        ("Arrow.tsrx", ARROW),
    ]);
    let (code, stdout, stderr) = build(project.path(), &["."]);
    assert_eq!(code, Some(1));
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        format!(
            "Built: ./App.vue -> ./dist/App.js\nBuilt: ./Badge.tsrx -> ./dist/Badge.js\n{}",
            refused("./Arrow.tsrx", &[OUTSIDE], 2)
        )
    );
    let mut written: Vec<_> = fs::read_dir(project.path().join("dist"))
        .unwrap()
        .map(|entry| entry.unwrap().file_name().into_string().unwrap())
        .collect();
    written.sort();
    assert_eq!(written, ["App.js", "Badge.js"]);
}

#[test]
fn continue_on_error_gives_a_refused_module_the_empty_fallback_of_any_failed_file() {
    let project = project(&[("Arrow.tsrx", ARROW)]);
    let (code, stdout, stderr) = build(project.path(), &["--continue-on-error", "Arrow.tsrx"]);
    assert_eq!(code, Some(1));
    assert_eq!(stdout, "");
    assert_eq!(
        stderr,
        format!(
            "Built: Arrow.tsrx -> ./dist/Arrow.js\n{}",
            refused("Arrow.tsrx", &[OUTSIDE], 0)
        )
    );
    assert_eq!(
        fs::read_to_string(project.path().join("dist/Arrow.js")).unwrap(),
        ""
    );
}
