#!/usr/bin/env node

import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { parseReleaseVersion } from "./release-platforms.mjs";

const DEFAULT_GIT_TIMEOUT_MS = 30_000;

function runGit(
  args,
  acceptedExitCodes = [0],
  cwd = process.cwd(),
  timeoutMs = DEFAULT_GIT_TIMEOUT_MS,
) {
  const result = spawnSync("git", args, {
    cwd,
    encoding: "utf8",
    stdio: ["ignore", "pipe", "pipe"],
    timeout: timeoutMs,
  });
  if (result.error?.code === "ETIMEDOUT") {
    throw new Error(`git ${args.join(" ")} timed out after ${timeoutMs}ms.`);
  }
  if (result.error != null) throw result.error;
  if (result.signal != null) {
    throw new Error(`git ${args.join(" ")} was terminated by ${result.signal}.`);
  }
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

export function verifyLocalReleaseGuard(
  tag,
  cwd = process.cwd(),
  timeoutMs = DEFAULT_GIT_TIMEOUT_MS,
) {
  const git = (args, acceptedExitCodes = [0]) => runGit(args, acceptedExitCodes, cwd, timeoutMs);
  parseReleaseVersion(tag);
  if (git(["branch", "--show-current"]).stdout.trim() !== "main") {
    throw new Error("Releases must be prepared from the local main branch.");
  }
  if (git(["status", "--porcelain"]).stdout.trim() !== "") {
    throw new Error("There are uncommitted changes. Please commit or stash them first.");
  }

  git(["fetch", "--quiet", "--no-tags", "origin", "+refs/heads/main:refs/remotes/origin/main"]);
  // A local commit or divergence is a different failure from a merely stale,
  // behind checkout, which the exact SHA comparison below diagnoses.
  if (
    git(["merge-base", "--is-ancestor", "HEAD", "refs/remotes/origin/main"], [0, 1]).status !== 0
  ) {
    throw new Error("HEAD is not reachable from the current origin/main.");
  }
  const head = git(["rev-parse", "HEAD"]).stdout.trim();
  const remoteMain = git(["rev-parse", "refs/remotes/origin/main"]).stdout.trim();
  if (head !== remoteMain) {
    throw new Error("HEAD must exactly match the current origin/main before preparing a release.");
  }

  const tagRef = `refs/tags/${tag}`;
  if (git(["rev-parse", "--verify", "--quiet", tagRef], [0, 1]).status === 0) {
    throw new Error(`Tag ${tag} already exists locally.`);
  }
  if (git(["ls-remote", "--exit-code", "--tags", "origin", tagRef], [0, 2]).status === 0) {
    throw new Error(`Remote tag ${tag} already exists and release tags are immutable.`);
  }
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  const [tag, extra] = process.argv.slice(2);
  Promise.resolve()
    .then(() => {
      if (tag == null || extra != null) {
        throw new Error("Usage: node tools/github/release-local-guard.mjs <release-tag>");
      }
      verifyLocalReleaseGuard(tag);
      console.log(`Local release guard passed for ${tag}.`);
    })
    .catch((error) => {
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    });
}
