#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";
import { fileURLToPath } from "node:url";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const AGENT_DIR = join(ROOT, "tools", "ai-issue-agent");
const PROMPT_TEMPLATE_PATH = join(AGENT_DIR, "prompt.md");
const STATE_PATH = join(ROOT, ".git", "ai-issue-agent-state.json");
const WORK_DIR = join(ROOT, ".git", "ai-issue-agent");
const CONVENTIONAL_TITLE =
  /^(build|chore|ci|docs|feat|fix|perf|refactor|style|test|revert)(\([A-Za-z0-9._-]+\))?!?:\s.+/;

function usage() {
  console.log(`Usage:
  node tools/ai-issue-agent.mjs run --issue <number> [options]
  node tools/ai-issue-agent.mjs once [options]
  node tools/ai-issue-agent.mjs watch [options]

Options:
  --repo <owner/name>          GitHub repository. Defaults to gh repo view.
  --base <branch>             PR base branch. Defaults to repository default branch.
  --remote <name>             Git remote to push branches to. Default: origin.
  --issue <number>            Issue number for run.
  --interval <seconds>        Watch polling interval. Default: 300.
  --limit <number>            Open issue scan limit. Default: 50.
  --include-existing          Watch mode also processes issues opened before startup.
  --agent-command <command>   Shell command to run instead of the default Codex CLI command.
  --no-wait-ci                Create/update PR without waiting for checks.
  --help                      Show this help.

The default agent command is:
  codex exec --full-auto --cd <repo-root> -o <result-file> -

Custom commands receive these environment variables:
  AI_ISSUE_AGENT_CONTEXT_FILE
  AI_ISSUE_AGENT_PROMPT_FILE
  AI_ISSUE_AGENT_RESULT_FILE
  AI_ISSUE_AGENT_ISSUE_NUMBER
`);
}

function parseArgs(argv) {
  const args = { _: [] };

  for (let i = 0; i < argv.length; i += 1) {
    const arg = argv[i];
    if (!arg.startsWith("--")) {
      args._.push(arg);
      continue;
    }

    if (arg === "--include-existing") {
      args.includeExisting = true;
      continue;
    }
    if (arg === "--no-wait-ci") {
      args.waitCi = false;
      continue;
    }
    if (arg === "--help") {
      args.help = true;
      continue;
    }

    const key = arg.slice(2).replaceAll("-", "_");
    const value = argv[i + 1];
    if (value == null || value.startsWith("--")) {
      throw new Error(`${arg} requires a value`);
    }
    args[key] = value;
    i += 1;
  }

  return args;
}

function run(bin, args, options = {}) {
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

function runJson(bin, args, options = {}) {
  const output = run(bin, args, options);
  return JSON.parse(output);
}

function ensureTool(bin) {
  run("sh", ["-lc", `command -v ${bin}`]);
}

function ensureCleanWorktree() {
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

function markProcessed(issueNumber, data) {
  const state = readState();
  state.processed[String(issueNumber)] = {
    ...data,
    processedAt: new Date().toISOString(),
  };
  writeState(state);
}

function isProcessed(issueNumber) {
  return readState().processed[String(issueNumber)] != null;
}

function resolveRepository(options) {
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

function fetchIssue(repo, issueNumber) {
  return runJson("gh", [
    "issue",
    "view",
    String(issueNumber),
    "--repo",
    repo,
    "--json",
    "author,authorAssociation,body,createdAt,labels,number,state,title,updatedAt,url",
  ]);
}

function listOpenIssues(repo, limit) {
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

function derivePrTitle(issue) {
  const labels = new Set((issue.labels ?? []).map((label) => label.name));
  let title = issue.title.replace(/^(\s*\[[Cc][Oo][Dd][Ee][Xx]\]\s*)+/, "").trim();

  if (title === "") {
    title = `implement issue #${issue.number}`;
  }

  if (CONVENTIONAL_TITLE.test(title)) {
    return title;
  }

  let type = "chore";
  if (labels.has("bug")) {
    type = "fix";
  } else if (labels.has("enhancement")) {
    type = "feat";
  } else if (labels.has("documentation") || labels.has("docs")) {
    type = "docs";
  }

  return `${type}: ${title}`;
}

function issueBranch(issueNumber) {
  return `ai-agent/issue-${issueNumber}`;
}

function buildPrompt(issue, contextPath) {
  const template = readFileSync(PROMPT_TEMPLATE_PATH, "utf8");
  return `${template}

## Issue Context File

Read this JSON file before editing:

${contextPath}

## Issue Context

\`\`\`json
${JSON.stringify(issue, null, 2)}
\`\`\`
`;
}

function runAgent({ agentCommand, contextPath, issue, prompt, promptPath, resultPath }) {
  const env = {
    AI_ISSUE_AGENT_CONTEXT_FILE: contextPath,
    AI_ISSUE_AGENT_ISSUE_NUMBER: String(issue.number),
    AI_ISSUE_AGENT_PROMPT_FILE: promptPath,
    AI_ISSUE_AGENT_RESULT_FILE: resultPath,
  };

  if (agentCommand != null) {
    console.log(`Running agent command: ${agentCommand}`);
    const result = spawnSync("sh", ["-lc", agentCommand], {
      cwd: ROOT,
      encoding: "utf8",
      env: { ...process.env, ...env },
      input: prompt,
      stdio: ["pipe", "inherit", "inherit"],
    });

    if (result.error) {
      throw result.error;
    }
    if (result.status !== 0) {
      throw new Error(`agent command exited with ${result.status}`);
    }
    return;
  }

  console.log("Running Codex CLI agent...");
  const result = spawnSync("codex", ["exec", "--full-auto", "--cd", ROOT, "-o", resultPath, "-"], {
    cwd: ROOT,
    encoding: "utf8",
    env: { ...process.env, ...env },
    input: prompt,
    stdio: ["pipe", "inherit", "inherit"],
  });

  if (result.error) {
    throw result.error;
  }
  if (result.status !== 0) {
    throw new Error(`codex exec exited with ${result.status}`);
  }
}

function writePrBody({ bodyPath, issue, prTitle, resultPath }) {
  const resultText = existsSync(resultPath)
    ? readFileSync(resultPath, "utf8").trim()
    : "No final agent message was written.";

  const body = `## Summary

Implements ${issue.url} using the local AI Issue Agent.

Closes #${issue.number}

## Change Class

- [ ] Parser or AST
- [ ] Compiler and codegen
- [ ] Semantic analysis, lint, and cross-file analysis
- [ ] Virtual TypeScript and type checking
- [ ] Formatter and LSP
- [ ] Runtime packaging, release, or docs
- [ ] Not language-facing

## Behavior Reference

${issue.url}

## Verification Evidence

Agent reported:

${resultText}

The local AI Issue Agent waits for PR checks after opening or updating this PR.

## Risk

AI-generated draft PR. Review the diff, verification evidence, and CI before merging.

<!-- local-ai-issue-agent title: ${prTitle} -->
`;

  writeFileSync(bodyPath, body);
}

function createOrUpdatePr({ baseBranch, branch, issue, prTitle, repo, resultPath }) {
  const workDir = join(WORK_DIR, `issue-${issue.number}`);
  const bodyPath = join(workDir, "pr-body.md");
  writePrBody({ bodyPath, issue, prTitle, resultPath });

  const existing = runJson("gh", [
    "pr",
    "list",
    "--repo",
    repo,
    "--head",
    branch,
    "--state",
    "open",
    "--json",
    "number,url",
  ]);

  if (existing.length > 0) {
    const pr = existing[0];
    run("gh", [
      "pr",
      "edit",
      String(pr.number),
      "--repo",
      repo,
      "--title",
      prTitle,
      "--body-file",
      bodyPath,
    ]);
    return pr;
  }

  const url = run("gh", [
    "pr",
    "create",
    "--repo",
    repo,
    "--draft",
    "--base",
    baseBranch,
    "--head",
    branch,
    "--title",
    prTitle,
    "--body-file",
    bodyPath,
  ]).trim();

  return runJson("gh", ["pr", "view", url, "--repo", repo, "--json", "number,url"]);
}

function waitForCi(repo, prNumber) {
  run(
    "gh",
    [
      "pr",
      "checks",
      String(prNumber),
      "--repo",
      repo,
      "--watch",
      "--fail-fast",
      "--interval",
      "30",
    ],
    { inherit: true },
  );
}

function processIssue(issueNumber, options) {
  const { baseBranch, repo } = resolveRepository(options);
  const remote = options.remote ?? "origin";
  const issue = fetchIssue(repo, issueNumber);
  if (issue.state !== "OPEN") {
    console.log(`Issue #${issue.number} is ${issue.state}; skipping.`);
    return null;
  }

  const branch = issueBranch(issue.number);
  const prTitle = derivePrTitle(issue);
  const workDir = join(WORK_DIR, `issue-${issue.number}`);
  const contextPath = join(workDir, "context.json");
  const promptPath = join(workDir, "prompt.md");
  const resultPath = join(workDir, "result.md");

  ensureCleanWorktree();
  rmSync(workDir, { force: true, recursive: true });
  mkdirSync(workDir, { recursive: true });
  writeFileSync(contextPath, `${JSON.stringify(issue, null, 2)}\n`);

  run("git", ["fetch", "--no-tags", remote, baseBranch]);
  run("git", ["switch", "-C", branch, `${remote}/${baseBranch}`]);

  const prompt = buildPrompt(issue, contextPath);
  writeFileSync(promptPath, prompt);
  runAgent({
    agentCommand: options.agent_command ?? process.env.AI_ISSUE_AGENT_COMMAND,
    contextPath,
    issue,
    prompt,
    promptPath,
    resultPath,
  });

  const status = run("git", ["status", "--porcelain"]);
  if (status.trim() === "") {
    console.log(`Agent made no changes for issue #${issue.number}.`);
    markProcessed(issue.number, { branch, prUrl: null, result: "no-changes" });
    run("git", ["switch", baseBranch]);
    return null;
  }

  run("git", ["add", "-A"]);
  run("git", ["commit", "-m", prTitle]);
  run("git", ["push", "--force-with-lease", remote, `HEAD:${branch}`]);

  const commit = run("git", ["rev-parse", "HEAD"]).trim();
  const pr = createOrUpdatePr({ baseBranch, branch, issue, prTitle, repo, resultPath });
  console.log(`PR: ${pr.url}`);

  if (options.waitCi !== false) {
    waitForCi(repo, pr.number);
  }

  markProcessed(issue.number, {
    branch,
    commit,
    prNumber: pr.number,
    prUrl: pr.url,
    result: "pr-created",
  });

  run("git", ["switch", baseBranch]);
  return pr;
}

function processOpenIssues(options, startedAt) {
  const { repo } = resolveRepository(options);
  const issues = listOpenIssues(repo, Number(options.limit ?? 50)).sort(
    (a, b) => new Date(a.createdAt) - new Date(b.createdAt),
  );

  for (const issue of issues) {
    if (!options.includeExisting && new Date(issue.createdAt) < startedAt) {
      continue;
    }
    if (isProcessed(issue.number)) {
      continue;
    }
    processIssue(issue.number, options);
  }
}

async function watch(options) {
  const startedAt = new Date();
  const interval = Number(options.interval ?? 300);
  console.log(`Watching issues every ${interval}s from ${startedAt.toISOString()}`);

  for (;;) {
    processOpenIssues(options, startedAt);
    await sleep(interval * 1000);
  }
}

async function main() {
  const options = parseArgs(process.argv.slice(2));
  const command = options._[0];

  if (options.help || command == null) {
    usage();
    return;
  }

  ensureTool("git");
  ensureTool("gh");
  if ((options.agent_command ?? process.env.AI_ISSUE_AGENT_COMMAND) == null) {
    ensureTool("codex");
  }

  if (command === "run") {
    if (options.issue == null) {
      throw new Error("run requires --issue <number>");
    }
    processIssue(options.issue, options);
  } else if (command === "once") {
    processOpenIssues({ ...options, includeExisting: true }, new Date(0));
  } else if (command === "watch") {
    await watch(options);
  } else {
    throw new Error(`Unknown command: ${command}`);
  }
}

main().catch((error) => {
  console.error(error instanceof Error ? error.message : error);
  process.exit(1);
});
