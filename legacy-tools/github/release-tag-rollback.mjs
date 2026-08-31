import { spawnSync } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { parseReleaseVersion } from "./release-platforms.mjs";

const DEFAULT_GIT_TIMEOUT_MS = 30_000;
const rollbackResults = new Set(["failure", "cancelled"]);

function runGit(args, cwd = process.cwd(), timeoutMs = DEFAULT_GIT_TIMEOUT_MS) {
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
  if (result.status !== 0) {
    const detail = [result.stdout, result.stderr].filter(Boolean).join("\n").trim();
    throw new Error(
      [`git ${args.join(" ")} failed with exit ${result.status}`, detail]
        .filter(Boolean)
        .join("\n"),
    );
  }
  return result;
}

export function assertRollbackContext(env) {
  const tag = env.GITHUB_REF_NAME ?? "";
  const sha = env.GITHUB_SHA ?? "";
  const repository = env.GITHUB_REPOSITORY ?? "";
  const preflightResult = env.RELEASE_PREFLIGHT_RESULT ?? "";
  const token = env.GITHUB_TOKEN ?? "";

  if (env.GITHUB_REF_TYPE !== "tag") {
    throw new Error(
      `Release rollback requires a tag event, got ${env.GITHUB_REF_TYPE ?? "unknown"}.`,
    );
  }
  parseReleaseVersion(tag);
  if (!/^v\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/.test(tag)) {
    throw new Error(`Release rollback requires a v-prefixed release tag, got ${tag}.`);
  }
  if (!/^[0-9a-f]{40}$/.test(sha)) {
    throw new Error(`Release rollback requires a full event SHA, got ${sha}.`);
  }
  if (!/^[A-Za-z0-9_.-]+\/[A-Za-z0-9_.-]+$/.test(repository)) {
    throw new Error(`Release rollback requires an owner/repository identity, got ${repository}.`);
  }
  if (!rollbackResults.has(preflightResult)) {
    throw new Error(
      `Refusing to roll back ${tag}: release preflight concluded ${preflightResult || "unknown"}.`,
    );
  }
  if (token === "") {
    throw new Error("Release rollback requires GITHUB_TOKEN to check for an existing release.");
  }
  return { preflightResult, repository, sha, tag, token };
}

export function remoteTagState(output, tag) {
  const refs = new Map(
    output
      .trim()
      .split(/\r?\n/)
      .filter(Boolean)
      .map((line) => line.trim().split(/\s+/, 2).reverse()),
  );
  const objectSha = refs.get(`refs/tags/${tag}`);
  if (objectSha == null) return undefined;
  const commitSha = refs.get(`refs/tags/${tag}^{}`) ?? objectSha;
  if (!/^[0-9a-f]{40}$/.test(objectSha) || !/^[0-9a-f]{40}$/.test(commitSha)) {
    throw new Error(`Remote tag ${tag} returned malformed object IDs.`);
  }
  return { commitSha, objectSha };
}

async function assertNoGithubRelease({ apiUrl, fetchImpl, repository, tag, token }) {
  const response = await fetchImpl(
    `${apiUrl}/repos/${repository}/releases/tags/${encodeURIComponent(tag)}`,
    {
      headers: {
        Accept: "application/vnd.github+json",
        Authorization: `Bearer ${token}`,
        "X-GitHub-Api-Version": "2022-11-28",
      },
    },
  );
  if (response.status === 404) return;
  if (response.status === 200) {
    throw new Error(`Refusing to delete ${tag}: a GitHub Release already exists.`);
  }
  const detail = (await response.text()).trim();
  throw new Error(
    `Could not prove that ${tag} is unpublished: GitHub Releases API returned ${response.status}${detail ? `: ${detail}` : ""}.`,
  );
}

export async function rollbackUnpublishedTag({
  env = process.env,
  cwd = process.cwd(),
  fetchImpl = globalThis.fetch,
  git = (args) => runGit(args, cwd),
} = {}) {
  const context = assertRollbackContext(env);
  const tagRef = `refs/tags/${context.tag}`;
  const readRemote = () =>
    remoteTagState(
      git(["ls-remote", "--tags", "origin", tagRef, `${tagRef}^{}`]).stdout,
      context.tag,
    );
  const remote = readRemote();
  if (remote == null) {
    return { deleted: false, reason: "already-absent", tag: context.tag };
  }
  if (remote.commitSha !== context.sha) {
    throw new Error(
      `Refusing to delete ${context.tag}: remote tag resolves to ${remote.commitSha}, not event SHA ${context.sha}.`,
    );
  }

  await assertNoGithubRelease({
    apiUrl: (env.GITHUB_API_URL ?? "https://api.github.com").replace(/\/$/, ""),
    fetchImpl,
    ...context,
  });

  git(["fetch", "--quiet", "--force", "origin", `${tagRef}:${tagRef}`]);
  const localObjectSha = git(["rev-parse", tagRef]).stdout.trim();
  const localCommitSha = git(["rev-parse", `${tagRef}^{}`]).stdout.trim();
  if (localObjectSha !== remote.objectSha || localCommitSha !== context.sha) {
    throw new Error(
      `Refusing to delete ${context.tag}: fetched tag identity does not match the audited remote tag.`,
    );
  }

  // Recheck after fetching so a Release created during the tag audit cannot
  // slip through immediately before the destructive operation.
  await assertNoGithubRelease({
    apiUrl: (env.GITHUB_API_URL ?? "https://api.github.com").replace(/\/$/, ""),
    fetchImpl,
    ...context,
  });

  git(["push", `--force-with-lease=${tagRef}:${remote.objectSha}`, "origin", `:${tagRef}`]);
  return { deleted: true, tag: context.tag };
}

const entrypoint = process.argv[1]
  ? fileURLToPath(import.meta.url) === path.resolve(process.argv[1])
  : false;
if (entrypoint) {
  rollbackUnpublishedTag()
    .then((result) => {
      console.log(
        result.deleted
          ? `Rolled back unpublished release tag ${result.tag}.`
          : `Release tag ${result.tag} was already absent.`,
      );
    })
    .catch((error) => {
      console.error(error instanceof Error ? error.message : String(error));
      process.exit(1);
    });
}
