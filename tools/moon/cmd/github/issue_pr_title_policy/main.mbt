/// Enforces repository issue and pull request title metadata policy.
/// The workflow runs this from trusted base-branch code. It reads the GitHub
/// event payload, normalizes repository area names in the Conventional Commits
/// scope of issue / PR titles, assigns opened and reopened issues / PRs to the
/// maintainer, and exits non-zero when a normalized PR title is not
/// Conventional Commits-shaped. Subject prose is never rewritten.

let default_assignee = "ubugeeei"

struct PolicySubject {
  kind : String
  number : String
  title : String
  assignees : Json?
}

fn has_assignee(assignees : Json?) -> Bool {
  guard assignees is Some(Array(items)) else { return false }
  for item in items {
    guard item is Object(obj) else { continue }
    guard obj.get("login") is Some(String(login)) else { continue }
    if login.to_lower() == default_assignee {
      return true
    }
  }
  false
}

fn policy_subject(event_name : String, payload : Json) -> PolicySubject? {
  guard payload is Object(root) else { return None }
  if event_name == "issues" {
    guard root.get("issue") is Some(Object(issue)) else { return None }
    if issue.get("pull_request") is Some(_) {
      return None
    }
    guard issue.get("number") is Some(Number(number, ..)) else { return None }
    guard issue.get("title") is Some(String(title)) else { return None }
    return Some(PolicySubject::{
      kind: "issue",
      number: "\{number}",
      title,
      assignees: issue.get("assignees"),
    })
  }
  if event_name == "pull_request" || event_name == "pull_request_target" {
    guard root.get("pull_request") is Some(Object(pr)) else { return None }
    guard pr.get("number") is Some(Number(number, ..)) else { return None }
    guard pr.get("title") is Some(String(title)) else { return None }
    return Some(PolicySubject::{
      kind: "pull_request",
      number: "\{number}",
      title,
      assignees: pr.get("assignees"),
    })
  }
  None
}

fn should_assign(payload : Json) -> Bool {
  guard payload is Object(root) else { return false }
  match root.get("action") {
    Some(String("opened")) | Some(String("reopened")) => true
    _ => false
  }
}

fn safe_log_value(value : String) -> String {
  let parts : Array[String] = []
  for i in 0..<value.length() {
    if value[i] == 10 || value[i] == 13 {
      parts.push(" ")
    } else {
      parts.push(value[i:i + 1].to_owned())
    }
  }
  parts.join("")
}

async fn gh_api(http_method : String, path : String, field_args : Array[String]) -> Int {
  let args = [
    "api",
    "--method",
    http_method,
    "-H",
    "X-GitHub-Api-Version: 2022-11-28",
    "--silent",
    path,
  ]
  for arg in field_args {
    args.push(arg)
  }
  @process.run("gh", args)
}

async fn apply_policy(event_name : String, repository : String, payload : Json) -> Int {
  let subject = match policy_subject(event_name, payload) {
    Some(value) => value
    None => {
      println("Skipping unsupported \{event_name} event")
      return 0
    }
  }
  let issue_path = "/repos/\{repository}/issues/\{subject.number}"
  let normalized_title = normalize_title(subject.title)

  if normalized_title != subject.title {
    let exit_code = gh_api("PATCH", issue_path, ["-f", "title=\{normalized_title}"])
    if exit_code != 0 {
      return exit_code
    }
    println(
      "Normalized \{subject.kind} #\{subject.number}: \"\{safe_log_value(subject.title)}\" -> \"\{safe_log_value(normalized_title)}\"",
    )
  }

  if should_assign(payload) && !has_assignee(subject.assignees) {
    let exit_code = gh_api(
      "POST",
      issue_path + "/assignees",
      ["-F", "assignees[]=\{default_assignee}"],
    )
    if exit_code != 0 {
      return exit_code
    }
    println("Assigned \{subject.kind} #\{subject.number} to \{default_assignee}")
  }

  if subject.kind == "pull_request" && !is_conventional_title(normalized_title) {
    println(
      "::error title=Invalid PR title::Use Conventional Commits format: type(scope): summary",
    )
    return 1
  }
  0
}

fn read_payload(path : String) -> Json? {
  try {
    Some(@json.parse(@xfs.read_file_to_string(path)))
  } catch {
    _ => None
  }
}

async fn run_app() -> Int {
  let event_path = match @env.get_env_var("GITHUB_EVENT_PATH") {
    Some(value) if value != "" => value
    _ => {
      println("GITHUB_EVENT_PATH is required")
      return 1
    }
  }
  let event_name = match @env.get_env_var("GITHUB_EVENT_NAME") {
    Some(value) if value != "" => value
    _ => {
      println("GITHUB_EVENT_NAME is required")
      return 1
    }
  }
  let repository = match @env.get_env_var("GITHUB_REPOSITORY") {
    Some(value) if value.contains("/") => value
    _ => {
      println("GITHUB_REPOSITORY must be owner/name")
      return 1
    }
  }
  let payload = match read_payload(event_path) {
    Some(value) => value
    None => {
      println("Failed to read or parse \{event_path}")
      return 1
    }
  }
  apply_policy(event_name, repository, payload)
}

async fn main {
  @sys.exit(run_app())
}
