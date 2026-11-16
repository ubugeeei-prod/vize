#![cfg(test)]
#![expect(clippy::disallowed_macros, reason = "fixtures use std strings")]
#![expect(clippy::disallowed_types, reason = "fixtures use std strings")]
//! `vize lint` on `.tsrx` modules: every markup region through the shared linter.

use serde_json::{Value, json};
use std::{fs, path::Path, process::Command};

fn lint(root: &Path, args: &[&str]) -> (Option<i32>, Value, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_vize"))
        .current_dir(root)
        .args(["lint", "--no-config", "--format", "json"])
        .args(args)
        .output()
        .unwrap();
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    let json = serde_json::from_str(&stdout)
        .unwrap_or_else(|error| panic!("{error}\nstdout:\n{stdout}\nstderr:\n{stderr}"));
    (output.status.code(), json, stderr)
}

fn project(files: &[(&str, &str)]) -> tempfile::TempDir {
    let project = tempfile::tempdir().unwrap();
    for (path, source) in files {
        let file = project.path().join(path);
        fs::create_dir_all(file.parent().unwrap()).unwrap();
        fs::write(file, source).unwrap();
    }
    project
}

fn documented(heading: &str) -> String {
    let page = fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../docs/content/rules/reference/parser-tsrx.md"),
    )
    .unwrap();
    let section = page.split_once(heading).unwrap().1;
    let fence = section.split_once("```").unwrap().1;
    let body = fence.split_once('\n').unwrap().1;
    format!("{}\n", body.split_once("\n```").unwrap().0)
}

const AVATAR: &str = "export function Avatar({ src, name }: { src: string; name: string }) @{
  const label = name.trim();

  <figure  class=\"avatar\">
    <img src={src} />
    <figcaption>{label}</figcaption>
  </figure>
}
";

#[test]
fn markup_findings_are_reported_at_module_spans() {
    let project = project(&[("src/Avatar.tsrx", AVATAR)]);
    let (code, json, stderr) = lint(project.path(), &["src/Avatar.tsrx"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        json,
        json!([{
            "file": "src/Avatar.tsrx",
            "messages": [
                {
                    "ruleId": "vue/no-multi-spaces",
                    "ruleDocsPath": "docs/content/rules/reference/vue-no-multi-spaces.md",
                    "severity": 1,
                    "message": "[vize:vue/no-multi-spaces] Multiple consecutive spaces",
                    "line": 4, "column": 10, "endLine": 4, "endColumn": 12,
                },
                {
                    "ruleId": "a11y/alt-text",
                    "ruleDocsPath": "docs/content/rules/reference/a11y-alt-text.md",
                    "severity": 1,
                    "message": "[vize:a11y/alt-text] <img> elements must have an alt attribute",
                    "line": 5, "column": 5, "endLine": 5, "endColumn": 22,
                    "help": "Add an alt attribute: <img alt=\"Description\"> or <img alt=\"\"> for decorative images",
                },
            ],
            "errorCount": 0,
            "warningCount": 2,
        }])
    );
}

#[test]
fn a_clean_module_has_no_findings_and_is_discovered_by_default() {
    let project = project(&[
        (
            "src/Badge.tsrx",
            "export function Badge({ text }: { text: string }) @{\n  <span class=\"badge\">{text}</span>\n}\n",
        ),
        ("src/App.vue", "<template>\n  <div>ok</div>\n</template>\n"),
    ]);
    let (code, json, stderr) = lint(project.path(), &[]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        json,
        json!([
            {"file": "src/App.vue", "messages": [], "errorCount": 0, "warningCount": 0},
            {"file": "src/Badge.tsrx", "messages": [], "errorCount": 0, "warningCount": 0},
        ])
    );
}

#[test]
fn the_documented_syntax_error_fails_and_its_repair_is_clean() {
    let project = project(&[
        ("Bad.tsrx", &documented("## Bad\n")),
        ("Good.tsrx", &documented("## Good\n")),
    ]);
    let (code, json, stderr) = lint(project.path(), &["Bad.tsrx", "Good.tsrx"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        json,
        json!([
            {
                "file": "Bad.tsrx",
                "messages": [{
                    "ruleId": "parser/tsrx",
                    "ruleDocsPath": "docs/content/rules/reference/parser-tsrx.md",
                    "severity": 2,
                    "message": "[vize:parser/tsrx] TSRX1002: Expected closing tag to match opening tag. Expected '</div>' but found '</span>'",
                    "line": 2, "column": 14, "endLine": 2, "endColumn": 21,
                }],
                "errorCount": 1,
                "warningCount": 0,
            },
            {"file": "Good.tsrx", "messages": [], "errorCount": 0, "warningCount": 0},
        ])
    );
}

#[test]
fn an_unsupported_construct_is_an_error_at_its_span() {
    let project = project(&[(
        "Status.tsrx",
        "export function Status({ state }: { state: string }) @{
  @switch (state) {
    @case 'ok': {
      <p>ok</p>
    }
  }
}
",
    )]);
    let (code, json, stderr) = lint(project.path(), &["Status.tsrx"]);
    assert_eq!(code, Some(1), "{stderr}");
    assert_eq!(
        json,
        json!([{
            "file": "Status.tsrx",
            "messages": [{
                "ruleId": "parser/tsrx",
                "ruleDocsPath": "docs/content/rules/reference/parser-tsrx.md",
                "severity": 2,
                "message": "[vize:parser/tsrx] Switch is not supported in TSRX modules yet, so this module's markup is not linted",
                "line": 2, "column": 3, "endLine": 6, "endColumn": 4,
            }],
            "errorCount": 1,
            "warningCount": 0,
        }])
    );
}

#[test]
fn fix_rewrites_every_region_and_converges() {
    let project = project(&[(
        "List.tsrx",
        "export function List({ items }: { items: string[] }) @{
  <ul  class=\"list\">
    {items.map((item) => <li  title={item}>{item}</li>)}
  </ul>
}

export function Footer() @{
  <footer  class=\"footer\">done</footer>
}
",
    )]);
    let (code, json, stderr) = lint(project.path(), &["--fix", "List.tsrx"]);
    assert_eq!(code, Some(0), "{stderr}");
    assert_eq!(
        json,
        json!([{"file": "List.tsrx", "messages": [], "errorCount": 0, "warningCount": 0}])
    );
    assert_eq!(
        fs::read_to_string(project.path().join("List.tsrx")).unwrap(),
        "export function List({ items }: { items: string[] }) @{
  <ul class=\"list\">
    {items.map((item) => <li title={item}>{item}</li>)}
  </ul>
}

export function Footer() @{
  <footer class=\"footer\">done</footer>
}
"
    );
}
