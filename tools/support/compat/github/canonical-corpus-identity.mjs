import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, readdirSync, realpathSync, statSync } from "node:fs";
import { join, relative, resolve } from "node:path";
import { rustCachePolicy } from "../../../../.github/actions/setup-rust-sticky-cache/cache-policy.mjs";

export const corpusRoot = "tests/_fixtures/_git";
export const artifactRoot = "real-project-davinci-dom-corpus";
export const observers = ["dom", "ssr-pug", "reach"];
export const expectedFiles = 42998;
export const expectedGitlinks = 147;
export const sha256 = (value) => createHash("sha256").update(value).digest("hex");

const git = (cwd, ...args) =>
  execFileSync("git", args, { cwd, encoding: "utf8", maxBuffer: 64 * 1024 * 1024 });

export function parseGitlinks(output) {
  const rows = output
    .split("\0")
    .filter(Boolean)
    .map((row) => {
      const match = /^160000 commit ([0-9a-f]{40})\t(tests\/_fixtures\/_git\/[^\0\r\n]+)$/.exec(
        row,
      );
      assert(match && !/^0+$/.test(match[1]), "Invalid canonical gitlink");
      assert(!match[2].split("/").some((part) => ["", ".", ".."].includes(part)), "Unsafe gitlink");
      return { path: match[2], sha: match[1] };
    });
  rows.sort((left, right) => (left.path < right.path ? -1 : left.path > right.path ? 1 : 0));
  assert.equal(rows.length, expectedGitlinks, "Canonical gitlink count changed");
  assert.equal(new Set(rows.map((row) => row.path)).size, rows.length, "Duplicate gitlink");
  return rows;
}

export function checkoutIdentity(cwd, env = process.env) {
  const sha = git(cwd, "rev-parse", "HEAD").trim();
  assert(/^[0-9a-f]{40}$/.test(env.GITHUB_SHA), "Missing candidate SHA");
  assert.equal(sha, env.GITHUB_SHA, "Canonical checkout differs from candidate");
  assert.equal(env.GITHUB_REPOSITORY, "ubugeeei-prod/vize", "Foreign canonical repository");
  const runId = Number(env.GITHUB_RUN_ID);
  const attempt = Number(env.GITHUB_RUN_ATTEMPT);
  assert(Number.isSafeInteger(runId) && runId > 0, "Missing canonical run ID");
  assert(Number.isSafeInteger(attempt) && attempt > 0, "Missing canonical attempt");
  return {
    schema: "vize.canonical-corpus-identity",
    version: 1,
    repository: env.GITHUB_REPOSITORY,
    runId,
    attempt,
    event: env.GITHUB_EVENT_NAME,
    sha,
    tree: git(cwd, "rev-parse", "HEAD^{tree}").trim(),
  };
}

export function corpusPlan(cwd, env = process.env) {
  const identity = checkoutIdentity(cwd, env);
  const event = JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  const repositoryId = Number(env.GITHUB_REPOSITORY_ID);
  assert(
    Number.isSafeInteger(repositoryId) && repositoryId > 0 && event.repository?.id === repositoryId,
    "Missing canonical base repository identity",
  );
  const providerSha =
    env.GITHUB_EVENT_NAME === "pull_request" ? event.pull_request?.head?.sha : identity.sha;
  assert(
    /^[0-9a-f]{40}$/.test(providerSha) && !/^0+$/.test(providerSha),
    "Missing canonical provider head",
  );
  const providerRepositoryId =
    env.GITHUB_EVENT_NAME === "pull_request" ? event.pull_request?.head?.repo?.id : repositoryId;
  assert(
    Number.isSafeInteger(providerRepositoryId) && providerRepositoryId > 0,
    "Missing canonical head repository identity",
  );
  const gitlinks = parseGitlinks(git(cwd, "ls-tree", "-rz", "HEAD", "--", corpusRoot));
  const indexed = git(cwd, "ls-files", "--stage", "-z", "--", corpusRoot);
  assert.equal(
    indexed,
    gitlinks.map((row) => `160000 ${row.sha} 0\t${row.path}\0`).join(""),
    "Canonical index differs from the committed gitlinks",
  );
  const modulesSha256 = sha256(git(cwd, "show", "HEAD:.gitmodules"));
  assert.equal(
    sha256(readFileSync(join(cwd, ".gitmodules"))),
    modulesSha256,
    "Canonical submodule URLs differ from the committed metadata",
  );
  const gitlinksSha256 = sha256(JSON.stringify(gitlinks));
  const policy = rustCachePolicy(
    {
      eventName: env.GITHUB_EVENT_NAME,
      event,
      ref: env.GITHUB_REF,
      repository: identity.repository,
      sourceSha: identity.sha,
      checkoutSha: identity.sha,
      runnerEnvironment: env.RUNNER_ENVIRONMENT,
      runnerOs: env.RUNNER_OS,
      runnerArch: env.RUNNER_ARCH,
      role: "canonical-corpus-objects",
      suffix: `${env.RUNNER_OS}-${env.RUNNER_ARCH}`,
      targetPath: ".git/modules/tests/_fixtures/_git",
    },
    { cwd, workspace: cwd },
  );
  return {
    ...identity,
    repositoryId,
    providerSha,
    providerRepositoryId,
    gitlinks,
    gitlinksSha256,
    modulesSha256,
    cacheKey: `canonical-corpus-objects-v1-${identity.repository}-${modulesSha256}-${gitlinksSha256}`,
    cacheTrusted: policy.trusted === "true",
  };
}

export function collectFiles(root) {
  const files = [];
  const ancestors = new Set();
  const visit = (directory) => {
    const physical = realpathSync(directory);
    assert(!ancestors.has(physical), "Canonical corpus contains a directory cycle");
    ancestors.add(physical);
    const children = readdirSync(directory).sort();
    for (const name of children) {
      const path = join(directory, name);
      if (statSync(path).isDirectory()) {
        if (!["node_modules", "_git-worktrees"].includes(name)) visit(path);
      } else if (name.endsWith(".vue")) {
        const bytes = readFileSync(path);
        files.push([relative(root, path).split("\\").join("/"), sha256(bytes), bytes.length]);
      }
    }
    ancestors.delete(physical);
  };
  visit(root);
  return files;
}

export function validateFiles(files) {
  assert(Array.isArray(files) && files.length === expectedFiles, "Canonical Vue corpus changed");
  const paths = new Set();
  for (const row of files) {
    assert(Array.isArray(row) && row.length === 3, "Invalid canonical file row");
    const [path, hash, size] = row;
    assert(
      typeof path === "string" &&
        path.endsWith(".vue") &&
        !path.startsWith("/") &&
        !path
          .split("/")
          .some((part) => ["", ".", "..", "node_modules", "_git-worktrees"].includes(part)),
      "Unsafe canonical file path",
    );
    assert(!paths.has(path), "Duplicate canonical file");
    paths.add(path);
    assert(/^[0-9a-f]{64}$/.test(hash), "Missing canonical file digest");
    assert(Number.isSafeInteger(size) && size >= 0, "Invalid canonical file size");
  }
  return sha256(JSON.stringify(files));
}

export function captureCorpus(cwd, env = process.env) {
  const plan = corpusPlan(cwd, env);
  for (const row of plan.gitlinks) {
    assert.equal(
      git(join(cwd, row.path), "rev-parse", "HEAD").trim(),
      row.sha,
      `Canonical submodule drift: ${row.path}`,
    );
    execFileSync("git", ["diff", "--quiet", "HEAD", "--"], { cwd: join(cwd, row.path) });
  }
  const files = collectFiles(resolve(cwd, corpusRoot));
  return { identity: { ...plan, files: files.length, filesSha256: validateFiles(files) }, files };
}

export function sameCorpus(left, right) {
  for (const key of [
    "schema",
    "version",
    "repository",
    "runId",
    "attempt",
    "event",
    "sha",
    "providerSha",
    "repositoryId",
    "providerRepositoryId",
    "tree",
    "gitlinksSha256",
    "modulesSha256",
    "files",
    "filesSha256",
  ])
    assert.deepEqual(left[key], right[key], `Canonical corpus ${key} changed`);
  assert.deepEqual(left.gitlinks, right.gitlinks, "Canonical gitlink ownership changed");
}
