use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::Command;

use vize_s0::{String, ToCompactString, cstr};

use super::{CANONICAL_CORPUS_ROOT, CorpusPreflightError, HydrationReport};

pub(super) fn assert_canonical_hydrated(workspace: &Path) -> Result<(), CorpusPreflightError> {
    let gitlinks = read_indexed_gitlinks(workspace)?;
    let statuses = read_submodule_statuses(workspace, gitlinks.keys())?;
    let report = build_report(workspace, &gitlinks, &statuses);
    if report.is_clean() {
        Ok(())
    } else {
        Err(CorpusPreflightError::Hydration(Box::new(report)))
    }
}

fn read_indexed_gitlinks(
    workspace: &Path,
) -> Result<BTreeMap<String, String>, CorpusPreflightError> {
    let output = git(
        workspace,
        &["ls-files", "--stage", "--", CANONICAL_CORPUS_ROOT],
    )?;
    let mut gitlinks = BTreeMap::new();
    for line in output.lines() {
        let Some((meta, path)) = line.split_once('\t') else {
            continue;
        };
        let mut fields = meta.split_whitespace();
        let (Some(mode), Some(sha), Some(stage)) = (fields.next(), fields.next(), fields.next())
        else {
            continue;
        };
        if mode == "160000" {
            gitlinks.insert(path.into(), cstr!("{sha}:{stage}"));
        }
    }
    Ok(gitlinks)
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SubmoduleStatus {
    marker: char,
    path: String,
}

fn read_submodule_statuses<'a>(
    workspace: &Path,
    indexed_paths: impl Iterator<Item = &'a String>,
) -> Result<BTreeMap<String, SubmoduleStatus>, CorpusPreflightError> {
    let paths: Vec<&str> = indexed_paths.map(String::as_str).collect();
    let output = git(
        workspace,
        &["submodule", "status", "--", CANONICAL_CORPUS_ROOT],
    )?;
    let mut statuses = BTreeMap::new();
    for line in output.lines() {
        if let Some(status) = parse_status_line(line, &paths) {
            statuses.insert(status.path.clone(), status);
        }
    }
    Ok(statuses)
}

fn parse_status_line(line: &str, indexed_paths: &[&str]) -> Option<SubmoduleStatus> {
    let marker = line.chars().next()?;
    let rest = line.get(marker.len_utf8()..)?.trim_start();
    let (_sha, tail) = rest.split_once(' ')?;
    let mut candidates: Vec<&str> = indexed_paths.to_vec();
    candidates.sort_by_key(|path| core::cmp::Reverse(path.len()));
    let path = candidates.into_iter().find(|path| {
        tail == *path
            || tail
                .strip_prefix(*path)
                .is_some_and(|tail| tail.starts_with(' '))
    })?;
    Some(SubmoduleStatus {
        marker,
        path: path.into(),
    })
}

fn build_report(
    workspace: &Path,
    gitlinks: &BTreeMap<String, String>,
    statuses: &BTreeMap<String, SubmoduleStatus>,
) -> HydrationReport {
    let mut report = HydrationReport {
        root: CANONICAL_CORPUS_ROOT.into(),
        indexed: gitlinks.len(),
        clean: 0,
        missing: Vec::new(),
        drifted: Vec::new(),
        conflicted: Vec::new(),
        invalid: Vec::new(),
        inventory_mismatch: Vec::new(),
    };
    if gitlinks.is_empty() {
        report
            .inventory_mismatch
            .push(cstr!("no indexed gitlinks under {CANONICAL_CORPUS_ROOT}"));
    }
    for (path, expected_stage) in gitlinks {
        classify_gitlink(
            workspace,
            &mut report,
            path,
            expected_stage,
            statuses.get(path),
        );
    }
    for path in statuses.keys() {
        if !gitlinks.contains_key(path) {
            report
                .inventory_mismatch
                .push(cstr!("{path}: submodule status has no indexed gitlink"));
        }
    }
    report
}

fn classify_gitlink(
    workspace: &Path,
    report: &mut HydrationReport,
    path: &str,
    expected_stage: &str,
    status: Option<&SubmoduleStatus>,
) {
    let Some((expected, stage)) = expected_stage.split_once(':') else {
        report.invalid.push(cstr!("{path}: malformed index entry"));
        return;
    };
    if stage != "0" {
        report
            .conflicted
            .push(cstr!("{path}: unmerged gitlink stage {stage}"));
        return;
    }
    match status.map(|status| status.marker) {
        // A real unhydrated submodule reports `-` and then fails the checkout
        // probe below. Synthetic test repos can carry a standalone checkout at
        // the path; in that case the indexed SHA remains the source of truth.
        Some('-' | ' ') => {}
        Some('+') => {
            report
                .drifted
                .push(cstr!("{path}: checked out at a different sha"));
            return;
        }
        Some('U') => {
            report.conflicted.push(cstr!("{path}: has merge conflicts"));
            return;
        }
        Some(marker) => {
            report
                .invalid
                .push(cstr!("{path}: invalid submodule status marker {marker:?}"));
            return;
        }
        None => {
            report
                .inventory_mismatch
                .push(cstr!("{path}: absent from git submodule status"));
            return;
        }
    }
    let checkout = workspace.join(path);
    if !is_non_empty_directory(&checkout) {
        report.missing.push(cstr!("{path}: not hydrated or empty"));
        return;
    }
    match read_head(&checkout) {
        Ok(actual) if actual == expected => report.clean += 1,
        Ok(_) => report
            .drifted
            .push(cstr!("{path}: HEAD does not match the indexed sha")),
        Err(detail) => report.invalid.push(cstr!("{path}: {detail}")),
    }
}

fn is_non_empty_directory(directory: &Path) -> bool {
    fs::read_dir(directory)
        .map(|mut entries| entries.next().is_some())
        .unwrap_or(false)
}

fn read_head(directory: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .arg("-C")
        .arg(directory)
        .arg("rev-parse")
        .arg("HEAD")
        .output()
        .map_err(|error| cstr!("{error}"))?;
    if output.status.success() {
        Ok(output_text(&output.stdout))
    } else {
        let detail = output_text(&output.stderr);
        Err(if detail.is_empty() {
            "not a git checkout".into()
        } else {
            detail
        })
    }
}

fn git(workspace: &Path, args: &[&str]) -> Result<String, CorpusPreflightError> {
    let output = Command::new("git")
        .args(args)
        .current_dir(workspace)
        .output()
        .map_err(|error| CorpusPreflightError::Git {
            command: git_command(args),
            detail: cstr!("{error}"),
        })?;
    if output.status.success() {
        Ok(output_text(&output.stdout))
    } else {
        let stderr = output_text(&output.stderr);
        let stdout = output_text(&output.stdout);
        Err(CorpusPreflightError::Git {
            command: git_command(args),
            detail: if stderr.is_empty() { stdout } else { stderr },
        })
    }
}

fn output_text(bytes: &[u8]) -> String {
    match core::str::from_utf8(bytes) {
        Ok(text) => text.trim().to_compact_string(),
        Err(_) => "<non-utf8 output>".into(),
    }
}

fn git_command(args: &[&str]) -> String {
    let mut command = String::from("git");
    for arg in args {
        command.push(' ');
        command.push_str(arg);
    }
    command
}

#[test]
fn status_parser_handles_paths_with_spaces() {
    let paths = ["tests/_fixtures/_git/path with spaces"];
    let status = parse_status_line(
        " 0123456789012345678901234567890123456789 tests/_fixtures/_git/path with spaces",
        &paths,
    )
    .expect("status line");
    assert_eq!(status.marker, ' ');
    assert_eq!(status.path, "tests/_fixtures/_git/path with spaces");
}
