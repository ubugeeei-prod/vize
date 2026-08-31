#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! serde = "1"
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../../support/common.rs"]
mod common;

use serde_json::Value;
use std::{env, ffi::OsString, fs, process::ExitCode};

const DEFAULT_ASSIGNEE: &str = "ubugeeei";
const API_VERSION_HEADER: &str = "X-GitHub-Api-Version: 2022-11-28";

#[derive(Debug, PartialEq)]
struct PolicySubject {
    kind: SubjectKind,
    number: u64,
    title: String,
    assignees: Option<Value>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SubjectKind {
    Issue,
    PullRequest,
}

impl SubjectKind {
    fn label(self) -> &'static str {
        match self {
            SubjectKind::Issue => "issue",
            SubjectKind::PullRequest => "pull_request",
        }
    }
}

fn main() -> ExitCode {
    ExitCode::from(run_app() as u8)
}

fn run_app() -> i32 {
    let Some(event_path) = non_empty_env("GITHUB_EVENT_PATH") else {
        println!("GITHUB_EVENT_PATH is required");
        return 1;
    };
    let Some(event_name) = non_empty_env("GITHUB_EVENT_NAME") else {
        println!("GITHUB_EVENT_NAME is required");
        return 1;
    };
    let Some(repository) = non_empty_env("GITHUB_REPOSITORY").filter(|value| value.contains('/'))
    else {
        println!("GITHUB_REPOSITORY must be owner/name");
        return 1;
    };
    let Some(payload) = read_payload(&event_path) else {
        println!("Failed to read or parse {event_path}");
        return 1;
    };

    apply_policy(&event_name, &repository, &payload)
}

fn non_empty_env(name: &str) -> Option<String> {
    env::var(name).ok().filter(|value| !value.is_empty())
}

fn read_payload(path: &str) -> Option<Value> {
    let text = fs::read_to_string(path).ok()?;
    serde_json::from_str(&text).ok()
}

fn apply_policy(event_name: &str, repository: &str, payload: &Value) -> i32 {
    let Some(subject) = policy_subject(event_name, payload) else {
        println!("Skipping unsupported {event_name} event");
        return 0;
    };
    let issue_path = format!("/repos/{repository}/issues/{}", subject.number);
    let normalized_title = normalize_title(&subject.title);

    if normalized_title != subject.title {
        let exit_code = gh_api(
            "PATCH",
            &issue_path,
            [
                OsString::from("-f"),
                OsString::from(format!("title={normalized_title}")),
            ],
        );
        if exit_code != 0 {
            return exit_code;
        }
        println!(
            "Normalized {} #{}: \"{}\" -> \"{}\"",
            subject.kind.label(),
            subject.number,
            safe_log_value(&subject.title),
            safe_log_value(&normalized_title)
        );
    }

    if should_assign(payload) && !has_assignee(subject.assignees.as_ref()) {
        let exit_code = gh_api(
            "POST",
            &format!("{issue_path}/assignees"),
            [
                OsString::from("-F"),
                OsString::from(format!("assignees[]={DEFAULT_ASSIGNEE}")),
            ],
        );
        if exit_code != 0 {
            return exit_code;
        }
        println!(
            "Assigned {} #{} to {DEFAULT_ASSIGNEE}",
            subject.kind.label(),
            subject.number
        );
    }

    if subject.kind == SubjectKind::PullRequest && !is_conventional_title(&normalized_title) {
        println!(
            "::error title=Invalid PR title::Use Conventional Commits format: type(scope): summary"
        );
        return 1;
    }

    0
}

fn gh_api<I>(http_method: &str, path: &str, field_args: I) -> i32
where
    I: IntoIterator<Item = OsString>,
{
    let mut args: Vec<OsString> = vec![
        "api".into(),
        "--method".into(),
        http_method.into(),
        "-H".into(),
        API_VERSION_HEADER.into(),
        "--silent".into(),
        path.into(),
    ];
    args.extend(field_args);

    match common::run_status("gh", &args) {
        Ok(status) => status,
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn policy_subject(event_name: &str, payload: &Value) -> Option<PolicySubject> {
    let root = payload.as_object()?;
    match event_name {
        "issues" => {
            let issue = root.get("issue")?.as_object()?;
            if issue.contains_key("pull_request") {
                return None;
            }
            Some(PolicySubject {
                kind: SubjectKind::Issue,
                number: issue.get("number")?.as_u64()?,
                title: issue.get("title")?.as_str()?.to_string(),
                assignees: issue.get("assignees").cloned(),
            })
        }
        "pull_request" | "pull_request_target" => {
            let pr = root.get("pull_request")?.as_object()?;
            Some(PolicySubject {
                kind: SubjectKind::PullRequest,
                number: pr.get("number")?.as_u64()?,
                title: pr.get("title")?.as_str()?.to_string(),
                assignees: pr.get("assignees").cloned(),
            })
        }
        _ => None,
    }
}

fn should_assign(payload: &Value) -> bool {
    payload
        .get("action")
        .and_then(Value::as_str)
        .is_some_and(|action| action == "opened" || action == "reopened")
}

fn has_assignee(assignees: Option<&Value>) -> bool {
    assignees.and_then(Value::as_array).is_some_and(|items| {
        items.iter().any(|item| {
            item.get("login")
                .and_then(Value::as_str)
                .is_some_and(|login| login.eq_ignore_ascii_case(DEFAULT_ASSIGNEE))
        })
    })
}

fn safe_log_value(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            '\n' | '\r' => ' ',
            _ => character,
        })
        .collect()
}

fn title_replacement(word: &str) -> Option<&'static str> {
    match word.to_ascii_lowercase().as_str() {
        "check" => Some("canon"),
        "compiler" => Some("atelier"),
        "lint" | "linter" => Some("patina"),
        "story" => Some("musea"),
        "format" | "fmt" => Some("glyph"),
        _ => None,
    }
}

fn normalize_scope(scope: &str) -> String {
    scope
        .split('/')
        .map(|segment| title_replacement(segment).unwrap_or(segment))
        .collect::<Vec<_>>()
        .join("/")
}

fn normalize_title(title: &str) -> String {
    let Some(separator) = colon_space_index(title) else {
        return title.to_string();
    };
    let prefix = &title[..separator];
    if !is_conventional_prefix(prefix) {
        return title.to_string();
    }
    let Some((scope_start, scope_end)) = scope_bounds(prefix) else {
        return title.to_string();
    };
    let scope = &prefix[scope_start..scope_end];
    let normalized_scope = normalize_scope(scope);
    if normalized_scope == scope {
        return title.to_string();
    }
    format!(
        "{}{}{}",
        &title[..scope_start],
        normalized_scope,
        &title[scope_end..]
    )
}

fn colon_space_index(value: &str) -> Option<usize> {
    value.as_bytes().windows(2).position(|pair| pair == b": ")
}

fn scope_bounds(prefix: &str) -> Option<(usize, usize)> {
    let open = prefix.as_bytes().iter().position(|byte| *byte == b'(')?;
    let close = prefix.as_bytes()[open + 1..]
        .iter()
        .position(|byte| *byte == b')')?
        + open
        + 1;
    Some((open + 1, close))
}

fn is_conventional_title(title: &str) -> bool {
    if title.is_empty()
        || title
            .chars()
            .any(|character| matches!(character, '\n' | '\r'))
    {
        return false;
    }
    let Some(separator) = colon_space_index(title) else {
        return false;
    };
    if separator + 2 >= title.len() || title.as_bytes()[separator + 2] == b' ' {
        return false;
    }
    is_conventional_prefix(&title[..separator])
}

fn is_conventional_prefix(prefix: &str) -> bool {
    let bytes = prefix.as_bytes();
    if bytes.is_empty() || !is_lower_ascii(bytes[0]) {
        return false;
    }

    let mut index = 1;
    while index < bytes.len() && is_type_char(bytes[index]) {
        index += 1;
    }

    if index < bytes.len() && bytes[index] == b'(' {
        index += 1;
        let scope_start = index;
        while index < bytes.len() && is_scope_char(bytes[index]) {
            index += 1;
        }
        if index == scope_start || index >= bytes.len() || bytes[index] != b')' {
            return false;
        }
        index += 1;
    }

    if index < bytes.len() && bytes[index] == b'!' {
        index += 1;
    }

    index == bytes.len()
}

fn is_lower_ascii(byte: u8) -> bool {
    byte.is_ascii_lowercase()
}

fn is_type_char(byte: u8) -> bool {
    is_lower_ascii(byte) || byte.is_ascii_digit() || byte == b'-'
}

fn is_scope_char(byte: u8) -> bool {
    is_lower_ascii(byte) || byte.is_ascii_digit() || matches!(byte, b'-' | b'.' | b'/' | b'_')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_scope_segments_only() {
        assert_eq!(
            normalize_title("fix(check/lint)!: keep type-check prose"),
            "fix(canon/patina)!: keep type-check prose"
        );
        assert_eq!(
            normalize_title("bench(tools): rank type-check engine classes separately"),
            "bench(tools): rank type-check engine classes separately"
        );
        assert_eq!(
            normalize_title("docs: format the output"),
            "docs: format the output"
        );
    }

    #[test]
    fn validates_conventional_titles() {
        assert!(is_conventional_title(
            "fix(cli): preserve failed compiler artifacts"
        ));
        assert!(is_conventional_title("feat(canon)!: drop legacy flag"));
        assert!(!is_conventional_title("fix lint issue"));
        assert!(!is_conventional_title("fix(Check): resolve alias imports"));
        assert!(!is_conventional_title("fix(cli):  double-spaced subject"));
    }

    #[test]
    fn extracts_policy_subjects() {
        let payload = serde_json::json!({
            "issue": {
                "number": 12,
                "title": "fix(check): normalize title",
                "assignees": []
            }
        });
        assert_eq!(
            policy_subject("issues", &payload),
            Some(PolicySubject {
                kind: SubjectKind::Issue,
                number: 12,
                title: "fix(check): normalize title".to_string(),
                assignees: Some(serde_json::json!([])),
            })
        );
    }
}
