//! Run documentation projects through the real CLI; do not credit missing context.
use serde_json::Value;
use std::{fs, path::Path, process::Command};
use vize_l0::{String, cstr};

#[test]
fn documented_cli_project_bad_and_good_pairs_match_the_supported_findings() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/content/rules/project");
    let mut pages: Vec<_> = fs::read_dir(root)
        .expect("project reference")
        .map(|entry| entry.expect("reference entry").path())
        .collect();
    pages.sort();
    let mut checked = 0;
    let mut unavailable = 0;
    let mut failures = Vec::new();
    for path in pages {
        let page = fs::read_to_string(path).expect("project page");
        // Reserved contracts, library-only codes and graph-only scenarios do
        // not claim a currently executable public CLI Bad/Good witness.
        let id = page
            .lines()
            .find_map(|line| line.strip_prefix("# `").and_then(|id| id.strip_suffix('`')))
            .expect("published rule ID");
        let no_source_finding = page.contains("Current support: `no-source-async-fact`");
        assert_eq!(no_source_finding, id == "vize:croquis/cf/async-no-suspense");
        let illustrative = page.contains("Example qualification: `illustrative-source-pair`");
        assert_eq!(
            illustrative,
            id == "vize:croquis/cf/circular-reactive-dependency",
            "only the existing tracked-identity graph example has illustrative source"
        );
        if illustrative {
            continue;
        }
        if !page.contains("## Shared project files")
            || (!page.contains("vp run lint") && !no_source_finding)
        {
            continue;
        }
        checked += 1;
        unavailable += usize::from(no_source_finding);
        let shared = section(&page, "## Shared project files", "## Bad");
        for (heading, next, should_report) in [("## Bad", "## Good", true), ("## Good", "", false)]
        {
            let project = tempfile::tempdir().expect("documentation project");
            for (file, source) in files(shared)
                .into_iter()
                .chain(files(section(&page, heading, next)))
            {
                let path = project.path().join(file.as_str());
                fs::create_dir_all(path.parent().expect("file parent"))
                    .expect("project directories");
                fs::write(path, source).expect("literal documentation source");
            }
            let result = Command::new(env!("CARGO_BIN_EXE_vize"))
                .current_dir(project.path())
                .args([
                    "lint",
                    "--no-config",
                    "--preset",
                    "incremental",
                    "--cross-file",
                    "**/*.vue",
                    "**/*.ts",
                    "--format",
                    "json",
                ])
                .output()
                .expect("public lint invocation");
            let reports: Value = serde_json::from_slice(&result.stdout).unwrap_or_else(|error| {
                panic!(
                    "{id} {heading}: invalid CLI JSON ({error}): stdout={}, stderr={}",
                    std::str::from_utf8(&result.stdout).unwrap_or("non-UTF8 stdout"),
                    std::str::from_utf8(&result.stderr).unwrap_or("non-UTF8 stderr")
                )
            });
            let messages: Vec<_> = reports
                .as_array()
                .expect("file reports")
                .iter()
                .flat_map(|file| file["messages"].as_array().expect("complete messages"))
                .collect();
            let count = messages
                .iter()
                .filter(|message| {
                    message["ruleId"].as_str() == Some(id)
                        || (id.starts_with("vize:croquis/cf/")
                            && message["message"]
                                .as_str()
                                .is_some_and(|text| text.contains(id)))
                })
                .count();
            let should_report = should_report && !no_source_finding;
            if (count > 0) != should_report {
                failures.push(cstr!(
                    "{id} {heading}: expected report={should_report}, found {count}; {reports}"
                ));
            }
            if !should_report
                && messages.iter().any(|message| {
                    message["ruleId"]
                        .as_str()
                        .is_some_and(|id| id.starts_with("parser/"))
                })
            {
                failures.push(cstr!("{id} Good must parse successfully: {reports}"));
            }
        }
    }
    assert_eq!(
        checked, 25,
        "18 emitted codes, one audited source-fact gap and six project lint IDs"
    );
    assert_eq!(
        unavailable, 1,
        "only the audited async macro input-fact gap"
    );
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}

fn section<'a>(page: &'a str, heading: &str, next: &str) -> &'a str {
    let text = page.split_once(heading).expect("documented section").1;
    if next.is_empty() {
        text
    } else {
        text.split_once(next).expect("next section").0
    }
}

fn files(section: &str) -> Vec<(String, String)> {
    let mut parts = section.split("```");
    let mut inputs = Vec::new();
    while let Some(before) = parts.next() {
        let Some(block) = parts.next() else { break };
        let file = before
            .lines()
            .rev()
            .find_map(|line| {
                line.strip_prefix('`')
                    .and_then(|file| file.strip_suffix('`'))
            })
            .expect("complete example filename");
        let (language, source) = block.split_once('\n').expect("source language");
        match language {
            "ts" | "vue" | "html" => {}
            other => panic!("unsupported documentation source language: {other}"),
        }
        inputs.push((file.into(), source.into()));
    }
    inputs
}
