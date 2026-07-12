#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import {
  bootstrapRequiredWorkflowRuns,
  createReleaseGateDispatchPlans,
  releaseGateRunQualifiers,
} from "./release-preflight-bootstrap.mjs";
import {
  assertReleaseCommitIsCurrentMain,
  assertReleaseMetadata,
  findReleaseBlockers,
  remoteTagCommit,
} from "./release-preflight-core.mjs";
import {
  assertRequiredWorkflowJobs,
  requiredReleaseWorkflows,
  selectRequiredWorkflowRuns,
  workflowRequiresJobEvidence,
} from "./release-preflight-evidence.mjs";
import { githubApiPages, githubApiRequest } from "./release-preflight-github.mjs";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const releasePackageRoots = ["editors", "npm"];

function runGit(args, acceptedExitCodes = [0], cwd = root) {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: 30_000,
  });
  if (result.error?.code === "ETIMEDOUT") {
    throw new Error(`git ${args.join(" ")} timed out after 30000ms`);
  }
  if (result.error != null) throw result.error;
  if (!acceptedExitCodes.includes(result.status)) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`git ${args.join(" ")} failed with exit ${result.status}`, detail]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result;
}

function verifyGitReleaseTarget(tag, sha) {
  const head = runGit(["rev-parse", "HEAD"]).stdout.trim();
  if (head !== sha) {
    throw new Error(`Checked out HEAD ${head} does not match release event SHA ${sha}`);
  }
  runGit(["fetch", "--quiet", "--no-tags", "origin", "+refs/heads/main:refs/remotes/origin/main"]);
  const mainSha = runGit(["rev-parse", "refs/remotes/origin/main"]).stdout.trim();
  assertReleaseCommitIsCurrentMain(sha, mainSha);

  const remoteTag = runGit([
    "ls-remote",
    "--exit-code",
    "--tags",
    "origin",
    `refs/tags/${tag}`,
    `refs/tags/${tag}^{}`,
  ]).stdout;
  const target = remoteTagCommit(remoteTag, tag);
  if (target !== sha) {
    const targetDescription = typeof target === "string" ? target : "nothing";
    throw new Error(`Remote tag ${tag} points to ${targetDescription}, expected ${sha}`);
  }
}

function releaseParentSha(sha) {
  const revision = runGit(["rev-list", "--parents", "-n", "1", sha]).stdout.trim().split(/\s+/);
  if (revision.length !== 2 || revision[0] !== sha) {
    throw new Error(`Release commit ${sha} must have exactly one parent for benchmark comparison`);
  }
  const baseSha = revision[1];
  const ancestry = runGit(["merge-base", "--is-ancestor", baseSha, sha], [0, 1]);
  if (ancestry.status !== 0) {
    throw new Error(`Benchmark base ${baseSha} is not an ancestor of release commit ${sha}`);
  }
  return baseSha;
}

export function readPackageManifests() {
  const manifestPaths = runGit(["ls-files", "-z", "--", ...releasePackageRoots])
    .stdout.split("\0")
    .filter((relativePath) => relativePath.endsWith("/package.json"));
  const manifests = [];
  for (const relativePath of manifestPaths) {
    const content = fs.readFileSync(path.join(root, relativePath), "utf8");
    let packageJson;
    try {
      packageJson = JSON.parse(content);
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      throw new Error(`Failed to parse tracked package manifest ${relativePath}: ${detail}`, {
        cause: error,
      });
    }
    if (packageJson.private === true) continue;
    manifests.push({ path: relativePath, content });
  }
  return manifests.sort((left, right) => left.path.localeCompare(right.path));
}

export function verifyReleaseTarget(env = process.env) {
  const tag = env.GITHUB_REF_NAME ?? "";
  const sha = env.GITHUB_SHA ?? "";
  if (env.GITHUB_REF_TYPE !== "tag") {
    throw new Error(
      `Release preflight requires a tag event, got ${env.GITHUB_REF_TYPE ?? "unknown"}`,
    );
  }
  const version = assertReleaseMetadata({
    tag,
    sha,
    cargoToml: fs.readFileSync(path.join(root, "Cargo.toml"), "utf8"),
    packageManifests: readPackageManifests(),
  });
  verifyGitReleaseTarget(tag, sha);
  return { tag, sha, version, baseSha: releaseParentSha(sha) };
}

export async function verifyReleasePreflight(env = process.env, { bootstrap = true } = {}) {
  const { tag, sha, version, baseSha } = verifyReleaseTarget(env);
  const repository = env.GITHUB_REPOSITORY ?? "";
  const token = env.GITHUB_TOKEN ?? "";
  const apiUrl = env.GITHUB_API_URL ?? "https://api.github.com";
  if (repository === "" || token === "") {
    throw new Error("GITHUB_REPOSITORY and GITHUB_TOKEN are required");
  }

  const listRuns = () =>
    githubApiPages({
      apiUrl,
      repository,
      token,
      resource: "actions/runs",
      collection: "workflow_runs",
      query: { head_sha: sha },
    });
  const dispatchPlans = createReleaseGateDispatchPlans({ ref: tag, headSha: sha, baseSha });
  let selectedRuns;
  if (bootstrap) {
    let previousWaitState = "";
    selectedRuns = await bootstrapRequiredWorkflowRuns({
      sha,
      dispatchPlans,
      listRuns,
      dispatchWorkflow: async (plan) => {
        console.log(`Dispatching ${plan.workflowName} on ${plan.ref} for ${sha}.`);
        await githubApiRequest({
          apiUrl,
          repository,
          token,
          resource: `actions/workflows/${encodeURIComponent(plan.workflowId)}/dispatches`,
          method: "POST",
          body: { ref: plan.ref, inputs: plan.inputs },
        });
      },
      onWait: ({ missing, pending }) => {
        const state = JSON.stringify({ missing, pending });
        if (state === previousWaitState) return;
        previousWaitState = state;
        console.log(
          `Waiting for exact-SHA release gates (missing: ${missing.join(", ") || "none"}; pending: ${pending.join(", ") || "none"}).`,
        );
      },
    });
  } else {
    selectedRuns = selectRequiredWorkflowRuns(
      await listRuns(),
      sha,
      requiredReleaseWorkflows,
      releaseGateRunQualifiers(dispatchPlans),
    );
  }

  const [issues] = await Promise.all([
    githubApiPages({
      apiUrl,
      repository,
      token,
      resource: "issues",
      collection: null,
      query: { state: "open" },
    }),
    ...[...selectedRuns]
      .filter(([workflowName]) => workflowRequiresJobEvidence(workflowName))
      .map(async ([workflowName, run]) => {
        const jobs = await githubApiPages({
          apiUrl,
          repository,
          token,
          resource: `actions/runs/${run.id}/jobs`,
          collection: "jobs",
        });
        assertRequiredWorkflowJobs(workflowName, jobs);
      }),
  ]);
  const blockers = findReleaseBlockers(issues);
  if (blockers.length > 0) {
    throw new Error(
      `Release-blocking issues remain open:\n${blockers
        .map((issue) => `- #${issue.number} ${issue.title}`)
        .join("\n")}`,
    );
  }
  verifyGitReleaseTarget(tag, sha);
  console.log(`Release preflight passed for ${tag} (${sha}) at workspace version ${version}.`);
  console.log(`Required workflows: ${requiredReleaseWorkflows.join(", ")}`);
}

export function parseReleasePreflightMode(args) {
  if (args.length === 0) return "bootstrap";
  if (args.length === 1 && args[0] === "--verify-only") return "verify-only";
  if (args.length === 1 && args[0] === "--target-only") return "target-only";
  throw new Error("Usage: node tools/github/release-preflight.mjs [--verify-only|--target-only]");
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const command = Promise.resolve().then(() => {
    const mode = parseReleasePreflightMode(process.argv.slice(2));
    if (mode === "target-only") return verifyReleaseTarget();
    return verifyReleasePreflight(process.env, { bootstrap: mode === "bootstrap" });
  });
  command.catch((error) => {
    console.error(error instanceof Error ? error.message : String(error));
    process.exit(1);
  });
}
