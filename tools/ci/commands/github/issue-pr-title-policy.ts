import fs from "node:fs";
import { spawnSync } from "node:child_process";

const DEFAULT_ASSIGNEE = "ubugeeei";
const API_VERSION_HEADER = "X-GitHub-Api-Version: 2022-11-28";

type JsonObject = Record<string, unknown>;
type SubjectKind = "issue" | "pull_request";

type PolicySubject = {
  kind: SubjectKind;
  number: number;
  title: string;
  assignees: unknown;
};

function isObject(value: unknown): value is JsonObject {
  return typeof value === "object" && value !== null && !Array.isArray(value);
}

function requiredEnv(name: string): string | undefined {
  const value = process.env[name];
  return value === undefined || value.length === 0 ? undefined : value;
}

function readPayload(path: string): unknown {
  try {
    return JSON.parse(fs.readFileSync(path, "utf8"));
  } catch {
    return undefined;
  }
}

function policySubject(eventName: string, payload: unknown): PolicySubject | undefined {
  if (!isObject(payload)) return undefined;

  if (eventName === "issues") {
    const issue = payload.issue;
    if (!isObject(issue) || Object.hasOwn(issue, "pull_request")) return undefined;
    if (typeof issue.number !== "number" || typeof issue.title !== "string") return undefined;
    return {
      kind: "issue",
      number: issue.number,
      title: issue.title,
      assignees: issue.assignees,
    };
  }

  if (eventName === "pull_request" || eventName === "pull_request_target") {
    const pullRequest = payload.pull_request;
    if (!isObject(pullRequest)) return undefined;
    if (typeof pullRequest.number !== "number" || typeof pullRequest.title !== "string") {
      return undefined;
    }
    return {
      kind: "pull_request",
      number: pullRequest.number,
      title: pullRequest.title,
      assignees: pullRequest.assignees,
    };
  }

  return undefined;
}

function shouldAssign(payload: unknown): boolean {
  return isObject(payload) && (payload.action === "opened" || payload.action === "reopened");
}

function hasDefaultAssignee(assignees: unknown): boolean {
  return (
    Array.isArray(assignees) &&
    assignees.some(
      (assignee) =>
        isObject(assignee) &&
        typeof assignee.login === "string" &&
        assignee.login.toLowerCase() === DEFAULT_ASSIGNEE,
    )
  );
}

function safeLogValue(value: string): string {
  return value.replace(/[\r\n]/g, " ");
}

function scopeReplacement(segment: string): string {
  switch (segment.toLowerCase()) {
    case "check":
      return "canon";
    case "compiler":
      return "atelier";
    case "lint":
    case "linter":
      return "patina";
    case "story":
      return "musea";
    case "format":
    case "fmt":
      return "glyph";
    default:
      return segment;
  }
}

function isConventionalPrefix(prefix: string): boolean {
  return /^[a-z][a-z0-9-]*(?:\([a-z0-9._/-]+\))?!?$/.test(prefix);
}

function normalizeTitle(title: string): string {
  const separator = title.indexOf(": ");
  if (separator < 0) return title;

  const prefix = title.slice(0, separator);
  if (!isConventionalPrefix(prefix)) return title;

  const scope = prefix.match(/\(([^)]+)\)/)?.[1];
  if (scope === undefined) return title;

  const normalizedScope = scope.split("/").map(scopeReplacement).join("/");
  if (normalizedScope === scope) return title;

  const scopeStart = prefix.indexOf(scope);
  return `${title.slice(0, scopeStart)}${normalizedScope}${title.slice(scopeStart + scope.length)}`;
}

function isConventionalTitle(title: string): boolean {
  if (title.length === 0 || /[\r\n]/.test(title)) return false;

  const separator = title.indexOf(": ");
  if (separator < 0) return false;

  const subject = title.slice(separator + 2);
  return (
    subject.length > 0 &&
    !subject.startsWith(" ") &&
    isConventionalPrefix(title.slice(0, separator))
  );
}

function ghApi(httpMethod: "PATCH" | "POST", path: string, fieldArgs: string[]): number {
  const result = spawnSync(
    "gh",
    ["api", "--method", httpMethod, "-H", API_VERSION_HEADER, "--silent", path, ...fieldArgs],
    { stdio: "inherit" },
  );
  if (result.error !== undefined) {
    console.error(result.error.message);
    return 1;
  }
  return result.status ?? 1;
}

function applyPolicy(eventName: string, repository: string, payload: unknown): number {
  const subject = policySubject(eventName, payload);
  if (subject === undefined) {
    console.log(`Skipping unsupported ${eventName} event`);
    return 0;
  }

  const issuePath = `/repos/${repository}/issues/${subject.number}`;
  const normalizedTitle = normalizeTitle(subject.title);
  if (normalizedTitle !== subject.title) {
    const exitCode = ghApi("PATCH", issuePath, ["-f", `title=${normalizedTitle}`]);
    if (exitCode !== 0) return exitCode;
    console.log(
      `Normalized ${subject.kind} #${subject.number}: "${safeLogValue(subject.title)}" -> "${safeLogValue(normalizedTitle)}"`,
    );
  }

  if (shouldAssign(payload) && !hasDefaultAssignee(subject.assignees)) {
    const exitCode = ghApi("POST", `${issuePath}/assignees`, [
      "-F",
      `assignees[]=${DEFAULT_ASSIGNEE}`,
    ]);
    if (exitCode !== 0) return exitCode;
    console.log(`Assigned ${subject.kind} #${subject.number} to ${DEFAULT_ASSIGNEE}`);
  }

  if (subject.kind === "pull_request" && !isConventionalTitle(normalizedTitle)) {
    console.log(
      "::error title=Invalid PR title::Use Conventional Commits format: type(scope): summary",
    );
    return 1;
  }

  return 0;
}

function runApp(): number {
  const eventPath = requiredEnv("GITHUB_EVENT_PATH");
  if (eventPath === undefined) {
    console.log("GITHUB_EVENT_PATH is required");
    return 1;
  }
  const eventName = requiredEnv("GITHUB_EVENT_NAME");
  if (eventName === undefined) {
    console.log("GITHUB_EVENT_NAME is required");
    return 1;
  }
  const repository = requiredEnv("GITHUB_REPOSITORY");
  if (repository === undefined || !repository.includes("/")) {
    console.log("GITHUB_REPOSITORY must be owner/name");
    return 1;
  }
  const payload = readPayload(eventPath);
  if (payload === undefined) {
    console.log(`Failed to read or parse ${eventPath}`);
    return 1;
  }

  return applyPolicy(eventName, repository, payload);
}

process.exitCode = runApp();
