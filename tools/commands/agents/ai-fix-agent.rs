#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! chrono = { version = "0.4", default-features = false, features = ["clock"] }
//! regex = "1"
//! serde = { version = "1", features = ["derive"] }
//! serde_json = "1"
//!
//! [package]
//! edition = "2024"
//! ```

#[path = "../../rust/common.rs"]
mod common;

use regex::Regex;
use serde_json::{Value, json};
use std::{
    collections::BTreeMap,
    env, fs,
    path::{Path, PathBuf},
    process::{Command, ExitCode, Stdio},
    thread,
    time::Duration,
};

fn main() -> ExitCode {
    common::main_result(run())
}

#[derive(Default)]
struct Options {
    positional: Vec<String>,
    values: BTreeMap<String, String>,
    include_existing: bool,
    wait_ci: bool,
    help: bool,
}

fn run() -> Result<(), String> {
    let options = parse_args(env::args().skip(1))?;
    let command = options.positional.first().map(String::as_str);
    if options.help || command.is_none() {
        usage();
        return Ok(());
    }

    ensure_tool("git")?;
    ensure_tool("gh")?;
    if options.value("agent_command").is_none() && env::var_os("AI_FIX_AGENT_COMMAND").is_none() {
        ensure_tool("codex")?;
    }

    match command.unwrap() {
        "run" => {
            let fix = options
                .value("fix")
                .ok_or_else(|| "run requires --fix <number>".to_string())?;
            process_fix_request(fix, &options)?;
        }
        "once" => process_open_fix_requests(
            &Options {
                include_existing: true,
                ..options
            },
            "1970-01-01T00:00:00Z",
        )?,
        "watch" => watch(&options)?,
        command => return Err(format!("Unknown command: {command}")),
    }
    Ok(())
}

impl Options {
    fn value(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }
}

fn usage() {
    println!(
        r#"Usage:
  rust-script tools/commands/agents/ai-fix-agent.rs run --fix <number> [options]
  rust-script tools/commands/agents/ai-fix-agent.rs once [options]
  rust-script tools/commands/agents/ai-fix-agent.rs watch [options]

Options:
  --repo <owner/name>          GitHub repository. Defaults to gh repo view.
  --base <branch>             PR base branch. Defaults to repository default branch.
  --remote <name>             Git remote to push branches to. Default: origin.
  --fix <number>              Fix request number for run.
  --interval <seconds>        Watch polling interval. Default: 300.
  --limit <number>            Open fix request scan limit. Default: 50.
  --include-existing          Watch mode also processes fix requests opened before startup.
  --agent-command <command>   Program and arguments to run instead of the default Codex CLI command.
                              Shell metacharacters are rejected; quote arguments if they contain spaces.
  --no-wait-ci                Create/update PR without waiting for checks.
  --help                      Show this help.

The default agent command is:
  codex exec --full-auto --cd <repo-root> -o <result-file> -

Custom commands receive these environment variables:
  AI_FIX_AGENT_CONTEXT_FILE
  AI_FIX_AGENT_PROMPT_FILE
  AI_FIX_AGENT_RESULT_FILE
  AI_FIX_AGENT_FIX_NUMBER
"#
    );
}

fn parse_args(args: impl Iterator<Item = String>) -> Result<Options, String> {
    let mut options = Options {
        wait_ci: true,
        ..Options::default()
    };
    let mut args = args.peekable();
    while let Some(arg) = args.next() {
        if !arg.starts_with("--") {
            options.positional.push(arg);
            continue;
        }
        match arg.as_str() {
            "--include-existing" => options.include_existing = true,
            "--no-wait-ci" => options.wait_ci = false,
            "--help" => options.help = true,
            _ => {
                let key = arg.trim_start_matches("--").replace('-', "_");
                let value = args
                    .next()
                    .filter(|value| !value.starts_with("--"))
                    .ok_or_else(|| format!("{arg} requires a value"))?;
                options.values.insert(key, value);
            }
        }
    }
    Ok(options)
}

fn repo_root() -> Result<PathBuf, String> {
    common::repo_root()
}

fn agent_dir(root: &Path) -> PathBuf {
    root.join("tools/ai-fix-agent")
}

fn state_path(root: &Path) -> PathBuf {
    root.join(".git/ai-fix-agent-state.json")
}

fn work_dir(root: &Path) -> PathBuf {
    root.join(".git/ai-fix-agent")
}

fn ensure_tool(bin: &str) -> Result<(), String> {
    let tool_name = Regex::new(r"^[A-Za-z0-9._+-]+$").unwrap();
    if bin.contains('/') || bin.contains('\\') || !tool_name.is_match(bin) {
        return Err(format!("invalid tool name: {bin}"));
    }
    let output = Command::new("sh")
        .args(["-lc", &format!("command -v -- {bin}")])
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to check {bin}: {error}"))?;
    if output.status.success() {
        Ok(())
    } else {
        Err(format!("{bin} was not found on PATH"))
    }
}

fn parse_command_arguments(command: &str) -> Result<Vec<String>, String> {
    if command.trim().is_empty() {
        return Err("agent command is empty".to_string());
    }
    if command
        .chars()
        .any(|character| "$`;&|<>(){}\n\r".contains(character))
    {
        return Err(
            "agent command must not contain shell metacharacters; pass a program and arguments only"
                .to_string(),
        );
    }

    let mut args = Vec::new();
    let mut current = String::new();
    let mut quote = None;
    for character in command.chars() {
        if let Some(active_quote) = quote {
            if character == active_quote {
                quote = None;
            } else {
                current.push(character);
            }
            continue;
        }
        if character == '"' || character == '\'' {
            quote = Some(character);
            continue;
        }
        if character.is_whitespace() {
            if !current.is_empty() {
                args.push(std::mem::take(&mut current));
            }
            continue;
        }
        current.push(character);
    }
    if quote.is_some() {
        return Err("unterminated quote in agent command".to_string());
    }
    if !current.is_empty() {
        args.push(current);
    }
    if args.is_empty() {
        Err("agent command is empty".to_string())
    } else {
        Ok(args)
    }
}

fn run_capture(root: &Path, bin: &str, args: &[String]) -> Result<String, String> {
    let output = Command::new(bin)
        .args(args)
        .current_dir(root)
        .stdin(Stdio::null())
        .output()
        .map_err(|error| format!("failed to run {bin}: {error}"))?;
    if !output.status.success() {
        let rendered = format!(
            "{}{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return Err(format!(
            "{} exited with {}\n{}",
            common::command_line(bin, args),
            output.status.code().unwrap_or(1),
            rendered.trim()
        ));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn run_json(root: &Path, bin: &str, args: &[String]) -> Result<Value, String> {
    serde_json::from_str(&run_capture(root, bin, args)?).map_err(|error| error.to_string())
}

fn ensure_clean_worktree(root: &Path) -> Result<(), String> {
    let status = run_capture(root, "git", &["status".into(), "--porcelain".into()])?;
    if status.trim().is_empty() {
        Ok(())
    } else {
        Err(format!("Working tree is not clean:\n{status}"))
    }
}

fn read_state(root: &Path) -> Result<Value, String> {
    let path = state_path(root);
    if !path.exists() {
        return Ok(json!({ "processed": {} }));
    }
    common::read_json(path)
}

fn write_state(root: &Path, state: &Value) -> Result<(), String> {
    common::write_json_pretty(state_path(root), state)
}

fn mark_processed(root: &Path, fix_number: u64, data: Value) -> Result<(), String> {
    let mut state = read_state(root)?;
    let processed = state
        .get_mut("processed")
        .and_then(Value::as_object_mut)
        .ok_or_else(|| "AI fix agent state must contain a processed object".to_string())?;
    let mut data = data
        .as_object()
        .cloned()
        .ok_or_else(|| "processed data must be an object".to_string())?;
    data.insert(
        "processedAt".to_string(),
        Value::String(chrono::Utc::now().to_rfc3339()),
    );
    processed.insert(fix_number.to_string(), Value::Object(data));
    write_state(root, &state)
}

fn is_processed(root: &Path, fix_number: u64) -> Result<bool, String> {
    Ok(read_state(root)?
        .get("processed")
        .and_then(Value::as_object)
        .and_then(|processed| processed.get(&fix_number.to_string()))
        .is_some())
}

fn resolve_repository(root: &Path, options: &Options) -> Result<(String, String), String> {
    let mut args = vec![
        "repo".to_string(),
        "view".to_string(),
        "--json".to_string(),
        "nameWithOwner,defaultBranchRef".to_string(),
    ];
    if let Some(repo) = options.value("repo") {
        args.insert(2, repo.to_string());
    }
    let repo_info = run_json(root, "gh", &args)?;
    let repo = repo_info
        .get("nameWithOwner")
        .and_then(Value::as_str)
        .ok_or_else(|| "gh repo view did not report nameWithOwner".to_string())?
        .to_string();
    let base_branch = options
        .value("base")
        .map(str::to_string)
        .or_else(|| {
            repo_info
                .get("defaultBranchRef")
                .and_then(|branch| branch.get("name"))
                .and_then(Value::as_str)
                .map(str::to_string)
        })
        .ok_or_else(|| "gh repo view did not report defaultBranchRef.name".to_string())?;
    Ok((repo, base_branch))
}

fn fetch_fix_request(root: &Path, repo: &str, fix_number: &str) -> Result<Value, String> {
    run_json(
        root,
        "gh",
        &[
            "issue".into(),
            "view".into(),
            fix_number.into(),
            "--repo".into(),
            repo.into(),
            "--json".into(),
            "author,authorAssociation,body,createdAt,labels,number,state,title,updatedAt,url"
                .into(),
        ],
    )
}

fn list_open_fix_requests(root: &Path, repo: &str, limit: &str) -> Result<Vec<Value>, String> {
    let value = run_json(
        root,
        "gh",
        &[
            "issue".into(),
            "list".into(),
            "--repo".into(),
            repo.into(),
            "--state".into(),
            "open".into(),
            "--limit".into(),
            limit.into(),
            "--json".into(),
            "createdAt,number,title,updatedAt,url".into(),
        ],
    )?;
    value
        .as_array()
        .cloned()
        .ok_or_else(|| "gh issue list did not return an array".to_string())
}

fn derive_pr_title(fix_request: &Value) -> Result<String, String> {
    let labels = fix_request
        .get("labels")
        .and_then(Value::as_array)
        .into_iter()
        .flatten()
        .filter_map(|label| label.get("name").and_then(Value::as_str))
        .collect::<std::collections::BTreeSet<_>>();
    let mut title = fix_request
        .get("title")
        .and_then(Value::as_str)
        .unwrap_or("")
        .to_string();
    let codex_prefix = Regex::new(r"^(\s*\[[Cc][Oo][Dd][Ee][Xx]\]\s*)+").unwrap();
    title = codex_prefix.replace(&title, "").trim().to_string();
    if title.is_empty() {
        title = format!(
            "fix #{}",
            fix_request
                .get("number")
                .and_then(Value::as_u64)
                .ok_or_else(|| "fix request is missing number".to_string())?
        );
    }
    let conventional = Regex::new(
        r"^(build|chore|ci|docs|feat|fix|perf|refactor|style|test|revert)(\([A-Za-z0-9._-]+\))?!?:\s.+",
    )
    .unwrap();
    if conventional.is_match(&title) {
        return Ok(title);
    }
    let kind = if labels.contains("fix") {
        "fix"
    } else if labels.contains("enhancement") {
        "feat"
    } else if labels.contains("documentation") || labels.contains("docs") {
        "docs"
    } else {
        "chore"
    };
    Ok(format!("{kind}: {title}"))
}

fn fix_branch(fix_number: u64) -> String {
    format!("ai-agent/fix-{fix_number}")
}

fn build_prompt(root: &Path, fix_request: &Value, context_path: &Path) -> Result<String, String> {
    let template = common::read_text(agent_dir(root).join("prompt.md"))?;
    Ok(format!(
        "{template}\n\n## Fix Context File\n\nRead this JSON file before editing:\n\n{}\n\n## Fix Context\n\n```json\n{}\n```\n",
        context_path.display(),
        serde_json::to_string_pretty(fix_request).map_err(|error| error.to_string())?
    ))
}

fn run_agent(
    root: &Path,
    agent_command: Option<&str>,
    fix_request: &Value,
    context_path: &Path,
    prompt: &str,
    prompt_path: &Path,
    result_path: &Path,
) -> Result<(), String> {
    let fix_number = fix_request
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| "fix request is missing number".to_string())?;
    let command;
    let args;
    if let Some(agent_command) = agent_command {
        let parsed = parse_command_arguments(agent_command)?;
        command = parsed[0].clone();
        args = parsed[1..].to_vec();
        println!("Running agent command: {} {}", command, args.join(" "));
    } else {
        command = "codex".to_string();
        args = vec![
            "exec".to_string(),
            "--full-auto".to_string(),
            "--cd".to_string(),
            root.to_string_lossy().into_owned(),
            "-o".to_string(),
            result_path.to_string_lossy().into_owned(),
            "-".to_string(),
        ];
        println!("Running Codex CLI agent...");
    }

    let mut child = Command::new(&command)
        .args(&args)
        .current_dir(root)
        .env("AI_FIX_AGENT_CONTEXT_FILE", context_path)
        .env("AI_FIX_AGENT_FIX_NUMBER", fix_number.to_string())
        .env("AI_FIX_AGENT_PROMPT_FILE", prompt_path)
        .env("AI_FIX_AGENT_RESULT_FILE", result_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|error| format!("failed to run agent command: {error}"))?;
    use std::io::Write;
    child
        .stdin
        .as_mut()
        .ok_or_else(|| "agent stdin is unavailable".to_string())?
        .write_all(prompt.as_bytes())
        .map_err(|error| format!("failed to write prompt: {error}"))?;
    let status = child
        .wait()
        .map_err(|error| format!("failed to wait for agent: {error}"))?;
    if status.success() {
        Ok(())
    } else if command == "codex" {
        Err(format!(
            "codex exec exited with {}",
            status.code().unwrap_or(1)
        ))
    } else {
        Err(format!(
            "agent command exited with {}",
            status.code().unwrap_or(1)
        ))
    }
}

fn write_pr_body(
    body_path: &Path,
    fix_request: &Value,
    pr_title: &str,
    result_path: &Path,
) -> Result<(), String> {
    let result_text = fs::read_to_string(result_path)
        .unwrap_or_else(|_| "No final agent message was written.".to_string())
        .trim()
        .to_string();
    let url = fix_request
        .get("url")
        .and_then(Value::as_str)
        .unwrap_or("<unknown>");
    let number = fix_request
        .get("number")
        .and_then(Value::as_u64)
        .unwrap_or_default();
    common::write_text(
        body_path,
        &format!(
            r#"## Summary

Implements {url} using the local AI Fix Agent.

Closes #{number}

## Change Class

- [ ] Parser or AST
- [ ] Compiler and codegen
- [ ] Semantic analysis, lint, and cross-file analysis
- [ ] Virtual TypeScript and type checking
- [ ] Formatter and LSP
- [ ] Runtime packaging, release, or docs
- [ ] Not language-facing

## Behavior Reference

{url}

## Verification Evidence

Agent reported:

{result_text}

The local AI Fix Agent waits for PR checks after opening or updating this PR.

## Risk

AI-generated draft PR. Review the diff, verification evidence, and CI before merging.

<!-- local-ai-fix-agent title: {pr_title} -->
"#
        ),
    )
}

fn create_or_update_pr(
    root: &Path,
    base_branch: &str,
    branch: &str,
    fix_request: &Value,
    pr_title: &str,
    repo: &str,
    result_path: &Path,
) -> Result<Value, String> {
    let fix_number = fix_request
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| "fix request is missing number".to_string())?;
    let work_dir = work_dir(root).join(format!("fix-{fix_number}"));
    let body_path = work_dir.join("pr-body.md");
    write_pr_body(&body_path, fix_request, pr_title, result_path)?;

    let existing = run_json(
        root,
        "gh",
        &[
            "pr".into(),
            "list".into(),
            "--repo".into(),
            repo.into(),
            "--head".into(),
            branch.into(),
            "--state".into(),
            "open".into(),
            "--json".into(),
            "number,url".into(),
        ],
    )?;
    if let Some(pr) = existing.as_array().and_then(|prs| prs.first()) {
        let number = pr
            .get("number")
            .and_then(Value::as_u64)
            .ok_or_else(|| "existing PR is missing number".to_string())?;
        run_capture(
            root,
            "gh",
            &[
                "pr".into(),
                "edit".into(),
                number.to_string(),
                "--repo".into(),
                repo.into(),
                "--title".into(),
                pr_title.into(),
                "--body-file".into(),
                body_path.to_string_lossy().into_owned(),
            ],
        )?;
        return Ok(pr.clone());
    }

    let url = run_capture(
        root,
        "gh",
        &[
            "pr".into(),
            "create".into(),
            "--repo".into(),
            repo.into(),
            "--draft".into(),
            "--base".into(),
            base_branch.into(),
            "--head".into(),
            branch.into(),
            "--title".into(),
            pr_title.into(),
            "--body-file".into(),
            body_path.to_string_lossy().into_owned(),
        ],
    )?
    .trim()
    .to_string();
    run_json(
        root,
        "gh",
        &[
            "pr".into(),
            "view".into(),
            url,
            "--repo".into(),
            repo.into(),
            "--json".into(),
            "number,url".into(),
        ],
    )
}

fn wait_for_ci(root: &Path, repo: &str, pr_number: u64) -> Result<(), String> {
    let status = Command::new("gh")
        .args([
            "pr",
            "checks",
            &pr_number.to_string(),
            "--repo",
            repo,
            "--watch",
            "--fail-fast",
            "--interval",
            "30",
        ])
        .current_dir(root)
        .stdin(Stdio::inherit())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .status()
        .map_err(|error| format!("failed to run gh pr checks: {error}"))?;
    if status.success() {
        Ok(())
    } else {
        Err(format!(
            "gh pr checks exited with {}",
            status.code().unwrap_or(1)
        ))
    }
}

fn process_fix_request(fix_number: &str, options: &Options) -> Result<Option<Value>, String> {
    let root = repo_root()?;
    let (repo, base_branch) = resolve_repository(&root, options)?;
    let remote = options.value("remote").unwrap_or("origin");
    let fix_request = fetch_fix_request(&root, &repo, fix_number)?;
    if fix_request.get("state").and_then(Value::as_str) != Some("OPEN") {
        println!(
            "Fix request #{} is {}; skipping.",
            fix_request
                .get("number")
                .and_then(Value::as_u64)
                .unwrap_or_default(),
            fix_request
                .get("state")
                .and_then(Value::as_str)
                .unwrap_or("<unknown>")
        );
        return Ok(None);
    }

    let fix_number = fix_request
        .get("number")
        .and_then(Value::as_u64)
        .ok_or_else(|| "fix request is missing number".to_string())?;
    let branch = fix_branch(fix_number);
    let pr_title = derive_pr_title(&fix_request)?;
    let work_dir = work_dir(&root).join(format!("fix-{fix_number}"));
    let context_path = work_dir.join("context.json");
    let prompt_path = work_dir.join("prompt.md");
    let result_path = work_dir.join("result.md");

    ensure_clean_worktree(&root)?;
    if work_dir.exists() {
        fs::remove_dir_all(&work_dir)
            .map_err(|error| format!("cannot reset {}: {error}", work_dir.display()))?;
    }
    fs::create_dir_all(&work_dir)
        .map_err(|error| format!("cannot create {}: {error}", work_dir.display()))?;
    common::write_json_pretty(&context_path, &fix_request)?;

    run_capture(
        &root,
        "git",
        &[
            "fetch".into(),
            "--no-tags".into(),
            remote.into(),
            base_branch.clone(),
        ],
    )?;
    run_capture(
        &root,
        "git",
        &[
            "switch".into(),
            "-C".into(),
            branch.clone(),
            format!("{remote}/{base_branch}"),
        ],
    )?;

    let prompt = build_prompt(&root, &fix_request, &context_path)?;
    common::write_text(&prompt_path, &prompt)?;
    let agent_command = options
        .value("agent_command")
        .map(str::to_string)
        .or_else(|| env::var("AI_FIX_AGENT_COMMAND").ok());
    run_agent(
        &root,
        agent_command.as_deref(),
        &fix_request,
        &context_path,
        &prompt,
        &prompt_path,
        &result_path,
    )?;

    let status = run_capture(&root, "git", &["status".into(), "--porcelain".into()])?;
    if status.trim().is_empty() {
        println!("Agent made no changes for fix request #{fix_number}.");
        mark_processed(
            &root,
            fix_number,
            json!({ "branch": branch, "prUrl": null, "result": "no-changes" }),
        )?;
        run_capture(&root, "git", &["switch".into(), base_branch])?;
        return Ok(None);
    }

    run_capture(&root, "git", &["add".into(), "-A".into()])?;
    run_capture(
        &root,
        "git",
        &["commit".into(), "-m".into(), pr_title.clone()],
    )?;
    run_capture(
        &root,
        "git",
        &[
            "push".into(),
            "--force-with-lease".into(),
            remote.into(),
            format!("HEAD:{branch}"),
        ],
    )?;
    let commit = run_capture(&root, "git", &["rev-parse".into(), "HEAD".into()])?
        .trim()
        .to_string();
    let pr = create_or_update_pr(
        &root,
        &base_branch,
        &branch,
        &fix_request,
        &pr_title,
        &repo,
        &result_path,
    )?;
    println!(
        "PR: {}",
        pr.get("url").and_then(Value::as_str).unwrap_or("")
    );
    if options.wait_ci {
        let number = pr
            .get("number")
            .and_then(Value::as_u64)
            .ok_or_else(|| "created PR is missing number".to_string())?;
        wait_for_ci(&root, &repo, number)?;
    }
    mark_processed(
        &root,
        fix_number,
        json!({
            "branch": branch,
            "commit": commit,
            "prNumber": pr.get("number").and_then(Value::as_u64),
            "prUrl": pr.get("url").and_then(Value::as_str),
            "result": "pr-created"
        }),
    )?;
    run_capture(&root, "git", &["switch".into(), base_branch])?;
    Ok(Some(pr))
}

fn process_open_fix_requests(options: &Options, started_at: &str) -> Result<(), String> {
    let root = repo_root()?;
    let (repo, _) = resolve_repository(&root, options)?;
    let limit = options.value("limit").unwrap_or("50");
    let mut fix_requests = list_open_fix_requests(&root, &repo, limit)?;
    fix_requests.sort_by(|left, right| {
        left.get("createdAt")
            .and_then(Value::as_str)
            .cmp(&right.get("createdAt").and_then(Value::as_str))
    });
    for fix_request in fix_requests {
        let Some(number) = fix_request.get("number").and_then(Value::as_u64) else {
            continue;
        };
        let created_at = fix_request
            .get("createdAt")
            .and_then(Value::as_str)
            .unwrap_or("");
        if !options.include_existing && created_at < started_at {
            continue;
        }
        if is_processed(&root, number)? {
            continue;
        }
        process_fix_request(&number.to_string(), options)?;
    }
    Ok(())
}

fn watch(options: &Options) -> Result<(), String> {
    let started_at = chrono::Utc::now().to_rfc3339();
    let interval = options
        .value("interval")
        .unwrap_or("300")
        .parse::<u64>()
        .map_err(|error| format!("invalid --interval: {error}"))?;
    println!("Watching fix requests every {interval}s from {started_at}");
    loop {
        process_open_fix_requests(options, &started_at)?;
        thread::sleep(Duration::from_secs(interval));
    }
}
