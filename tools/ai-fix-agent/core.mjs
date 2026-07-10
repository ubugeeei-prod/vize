import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

export const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..", "..");
export const AGENT_DIR = join(ROOT, "tools", "ai-fix-agent");
export const PROMPT_TEMPLATE_PATH = join(AGENT_DIR, "prompt.md");
export const STATE_PATH = join(ROOT, ".git", "ai-fix-agent-state.json");
export const WORK_DIR = join(ROOT, ".git", "ai-fix-agent");

const CONVENTIONAL_TITLE =
  /^(build|chore|ci|docs|feat|fix|perf|refactor|style|test|revert)(\([A-Za-z0-9._-]+\))?!?:\s.+/;

export function run(bin, args, options = {}) {
  const result = spawnSync(bin, args, {
    cwd: options.cwd ?? ROOT,
    encoding: "utf8",
    env: { ...process.env, ...options.env },
    input: options.input,
    stdio: options.inherit ? ["inherit", "inherit", "inherit"] : ["pipe", "pipe", "pipe"],
  });

  if (result.error) {
    throw result.error;
  }

  const okStatuses = options.okStatuses ?? [0];
  if (!okStatuses.includes(result.status)) {
    const command = [bin, ...args].join(" ");
    const output = [result.stdout, result.stderr].filter(Boolean).join("\n");
    throw new Error(`${command} exited with ${result.status}\n${output}`);
  }

  return result.stdout ?? "";
}

export function runJson(bin, args, options = {}) {
  const output = run(bin, args, options);
  return JSON.parse(output);
}

export function ensureTool(bin) {
  run("sh", ["-lc", `command -v ${bin}`]);
}

export function ensureCleanWorktree() {
  const status = run("git", ["status", "--porcelain"]);
  if (status.trim() !== "") {
    throw new Error(`Working tree is not clean:\n${status}`);
  }
}

function readState() {
  if (!existsSync(STATE_PATH)) {
    return { processed: {} };
  }
  return JSON.parse(readFileSync(STATE_PATH, "utf8"));
}

function writeState(state) {
  mkdirSync(dirname(STATE_PATH), { recursive: true });
  writeFileSync(STATE_PATH, `${JSON.stringify(state, null, 2)}\n`);
}

export function markProcessed(fixNumber, data) {
  const state = readState();
  state.processed[String(fixNumber)] = {
    ...data,
    processedAt: new Date().toISOString(),
  };
  writeState(state);
}

export function isProcessed(fixNumber) {
  return readState().processed[String(fixNumber)] != null;
}

export function resolveRepository(options) {
  const repoInfo = runJson(
    "gh",
    ["repo", "view", options.repo ?? "", "--json", "nameWithOwner,defaultBranchRef"].filter(
      Boolean,
    ),
  );

  return {
    baseBranch: options.base ?? repoInfo.defaultBranchRef.name,
    repo: repoInfo.nameWithOwner,
  };
}

export function fetchFixRequest(repo, fixNumber) {
  return runJson("gh", [
    "issue",
    "view",
    String(fixNumber),
    "--repo",
    repo,
    "--json",
    "author,authorAssociation,body,createdAt,labels,number,state,title,updatedAt,url",
  ]);
}

export function listOpenFixRequests(repo, limit) {
  return runJson("gh", [
    "issue",
    "list",
    "--repo",
    repo,
    "--state",
    "open",
    "--limit",
    String(limit),
    "--json",
    "createdAt,number,title,updatedAt,url",
  ]);
}

export function derivePrTitle(fixRequest) {
  const labels = new Set((fixRequest.labels ?? []).map((label) => label.name));
  let title = fixRequest.title.replace(/^(\s*\[[Cc][Oo][Dd][Ee][Xx]\]\s*)+/, "").trim();

  if (title === "") {
    title = `fix #${fixRequest.number}`;
  }

  if (CONVENTIONAL_TITLE.test(title)) {
    return title;
  }

  let type = "chore";
  if (labels.has("fix")) {
    type = "fix";
  } else if (labels.has("enhancement")) {
    type = "feat";
  } else if (labels.has("documentation") || labels.has("docs")) {
    type = "docs";
  }

  return `${type}: ${title}`;
}

export function fixBranch(fixNumber) {
  return `ai-agent/fix-${fixNumber}`;
}
