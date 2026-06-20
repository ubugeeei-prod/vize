import { spawnSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { setTimeout as sleep } from "node:timers/promises";

import {
  PROMPT_TEMPLATE_PATH,
  ROOT,
  WORK_DIR,
  derivePrTitle,
  ensureCleanWorktree,
  fetchFixRequest,
  fixBranch,
  isProcessed,
  listOpenFixRequests,
  markProcessed,
  resolveRepository,
  run,
  runJson,
} from "./core.mjs";

function buildPrompt(fixRequest, contextPath) {
  const template = readFileSync(PROMPT_TEMPLATE_PATH, "utf8");
  return `${template}

## Fix Context File

Read this JSON file before editing:

${contextPath}

## Fix Context

\`\`\`json
${JSON.stringify(fixRequest, null, 2)}
\`\`\`
`;
}

function runAgent({ agentCommand, contextPath, fixRequest, prompt, promptPath, resultPath }) {
  const env = {
    AI_FIX_AGENT_CONTEXT_FILE: contextPath,
    AI_FIX_AGENT_FIX_NUMBER: String(fixRequest.number),
    AI_FIX_AGENT_PROMPT_FILE: promptPath,
    AI_FIX_AGENT_RESULT_FILE: resultPath,
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

function writePrBody({ bodyPath, fixRequest, prTitle, resultPath }) {
  const resultText = existsSync(resultPath)
    ? readFileSync(resultPath, "utf8").trim()
    : "No final agent message was written.";

  const body = `## Summary

Implements ${fixRequest.url} using the local AI Fix Agent.

Closes #${fixRequest.number}

## Change Class

- [ ] Parser or AST
- [ ] Compiler and codegen
- [ ] Semantic analysis, lint, and cross-file analysis
- [ ] Virtual TypeScript and type checking
- [ ] Formatter and LSP
- [ ] Runtime packaging, release, or docs
- [ ] Not language-facing

## Behavior Reference

${fixRequest.url}

## Verification Evidence

Agent reported:

${resultText}

The local AI Fix Agent waits for PR checks after opening or updating this PR.

## Risk

AI-generated draft PR. Review the diff, verification evidence, and CI before merging.

<!-- local-ai-fix-agent title: ${prTitle} -->
`;

  writeFileSync(bodyPath, body);
}

function createOrUpdatePr({ baseBranch, branch, fixRequest, prTitle, repo, resultPath }) {
  const workDir = join(WORK_DIR, `fix-${fixRequest.number}`);
  const bodyPath = join(workDir, "pr-body.md");
  writePrBody({ bodyPath, fixRequest, prTitle, resultPath });

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

export function processFixRequest(fixNumber, options) {
  const { baseBranch, repo } = resolveRepository(options);
  const remote = options.remote ?? "origin";
  const fixRequest = fetchFixRequest(repo, fixNumber);
  if (fixRequest.state !== "OPEN") {
    console.log(`Fix request #${fixRequest.number} is ${fixRequest.state}; skipping.`);
    return null;
  }

  const branch = fixBranch(fixRequest.number);
  const prTitle = derivePrTitle(fixRequest);
  const workDir = join(WORK_DIR, `fix-${fixRequest.number}`);
  const contextPath = join(workDir, "context.json");
  const promptPath = join(workDir, "prompt.md");
  const resultPath = join(workDir, "result.md");

  ensureCleanWorktree();
  rmSync(workDir, { force: true, recursive: true });
  mkdirSync(workDir, { recursive: true });
  writeFileSync(contextPath, `${JSON.stringify(fixRequest, null, 2)}\n`);

  run("git", ["fetch", "--no-tags", remote, baseBranch]);
  run("git", ["switch", "-C", branch, `${remote}/${baseBranch}`]);

  const prompt = buildPrompt(fixRequest, contextPath);
  writeFileSync(promptPath, prompt);
  runAgent({
    agentCommand: options.agent_command ?? process.env.AI_FIX_AGENT_COMMAND,
    contextPath,
    fixRequest,
    prompt,
    promptPath,
    resultPath,
  });

  const status = run("git", ["status", "--porcelain"]);
  if (status.trim() === "") {
    console.log(`Agent made no changes for fix request #${fixRequest.number}.`);
    markProcessed(fixRequest.number, { branch, prUrl: null, result: "no-changes" });
    run("git", ["switch", baseBranch]);
    return null;
  }

  run("git", ["add", "-A"]);
  run("git", ["commit", "-m", prTitle]);
  run("git", ["push", "--force-with-lease", remote, `HEAD:${branch}`]);

  const commit = run("git", ["rev-parse", "HEAD"]).trim();
  const pr = createOrUpdatePr({ baseBranch, branch, fixRequest, prTitle, repo, resultPath });
  console.log(`PR: ${pr.url}`);

  if (options.waitCi !== false) {
    waitForCi(repo, pr.number);
  }

  markProcessed(fixRequest.number, {
    branch,
    commit,
    prNumber: pr.number,
    prUrl: pr.url,
    result: "pr-created",
  });

  run("git", ["switch", baseBranch]);
  return pr;
}

export function processOpenFixRequests(options, startedAt) {
  const { repo } = resolveRepository(options);
  const fixRequests = listOpenFixRequests(repo, Number(options.limit ?? 50)).sort(
    (a, b) => new Date(a.createdAt) - new Date(b.createdAt),
  );

  for (const fixRequest of fixRequests) {
    if (!options.includeExisting && new Date(fixRequest.createdAt) < startedAt) {
      continue;
    }
    if (isProcessed(fixRequest.number)) {
      continue;
    }
    processFixRequest(fixRequest.number, options);
  }
}

export async function watch(options) {
  const startedAt = new Date();
  const interval = Number(options.interval ?? 300);
  console.log(`Watching fix requests every ${interval}s from ${startedAt.toISOString()}`);

  for (;;) {
    processOpenFixRequests(options, startedAt);
    await sleep(interval * 1000);
  }
}
