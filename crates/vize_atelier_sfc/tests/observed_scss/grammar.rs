use super::*;
use serde_json::{Value, json};
use std::{
    io::Write,
    path::Path,
    process::{Command, Stdio},
};

#[test]
#[ignore = "runs the project's original SCSS preprocessor across selectors, values and nesting"]
fn admitted_grammar_matches_the_complete_project_preprocessor() {
    let mut rows = Vec::new();
    let mut candidates = 0;
    let mut check = |source: String, required: bool| {
        candidates += 1;
        match ScssProducts::read(whole(&source)).css() {
            Ok(css) => rows.push(json!({"name":"grammar.scss","source":source,"css":css.as_str()})),
            Err(refusal) => assert!(!required, "{source:?}: {refusal:?}"),
        }
    };
    for root in [".a", "main", "_x.card", ".card .child", "Item.a.b"] {
        for child in [
            "&",
            ".nested",
            "&.active",
            "&__child",
            "&-wide .item",
            "div.x",
        ] {
            for literal in [
                "#AbC",
                "Red",
                "0",
                "999999999px",
                "10%",
                "1px solid #aaBBcc",
                "hidden",
                "NaN",
            ] {
                check(
                    format!(
                        "$theme_color:Red;{root}{{x:$theme-color;{child}{{margin:{literal}}}y:{literal}}}"
                    ),
                    true,
                );
                check(
                    format!("{root}{{x:{literal};{child}{{}}y:red}}{root}{{z:{literal}}}"),
                    true,
                );
            }
        }
    }
    for suffix in [
        "a:b",
        ";",
        "{}",
        ".c{}",
        "&-{}",
        ".b{x:y}",
        ".b{&.d{x:y};z:k}",
    ] {
        for trailing in ["", ";", " .c{}", ";p:z", ";; .d{q:e} x:y;"] {
            check(format!(".a{{p:red; {suffix}{trailing}}}"), false);
        }
    }
    for space in [
        "", " ", "\t", "\r", "\n", "\r\n", "\x0c", "\x0b", "\0", "\u{a0}", "\u{2003}",
    ] {
        check(
            format!("{space}.a{space}{{{space}color{space}:{space}red{space};{space}}}{space}"),
            false,
        );
        check(format!(".a{space}.b{{ x:1px{space}solid; }}"), false);
    }
    for value in [
        "$missing",
        "not",
        "true",
        "NULL",
        "and",
        "a and b",
        "2.0",
        "0e2",
        "#abcd",
        "0.1s",
        "a,b",
        "inherit",
        "currentColor",
        "apple_2",
        "1%",
        "000px",
        "1234567890ms",
        "1rem",
        "url(a)",
        "var(--x)",
    ] {
        check(format!("$x:{value};.sample{{y:$x;}}"), false);
        check(format!(".sample{{x:{value};}}"), false);
    }
    check("$x:1;$x:$x;.a{x:$x}$x:3px;.b{y:$x}".to_owned(), true);
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let mut child = Command::new("node")
        .current_dir(&root)
        .arg("crates/vize_atelier_sfc/tests/observed_scss/oracle.mjs")
        .arg("--correctness-only")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .take()
        .unwrap()
        .write_all(&serde_json::to_vec(&rows).unwrap())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(output.status.success(), "original preprocessing failed");
    let result: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(result["checked"], rows.len());
    eprintln!(
        "scss_grammar candidates={candidates} admitted={}",
        rows.len()
    );
}
