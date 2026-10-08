/** Authenticate immutable version cuts before selecting their whole original native step. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { readFileSync, realpathSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import {
  metadataCandidate,
  rewriteVersionMetadata,
} from "./typechecker-native-release-metadata.mjs";

export const WORKFLOW = ".github/workflows/typechecker-native-phases.yml";
export const STEP =
  "Qualify declared template emits and config-scoped Vue helpers in the source CLI and editor";
const REPO = "ubugeeei-prod/vize";
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const sha = (value) => {
  assert.match(value ?? "", /^[0-9a-f]{40}$/u);
  return value;
};
function git(root, ...args) {
  const result = spawnSync("git", args, {
    cwd: root,
    encoding: "utf8",
    timeout: 30_000,
    maxBuffer: 16 * 1024 * 1024,
  });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
const object = (root, revision, path) => git(root, "show", `${revision}:${path}`);
const tree = (root, revision) => git(root, "rev-parse", `${revision}^{tree}`).trim();
function marker(body, key) {
  const prefix = `<!-- ${key}: `;
  const values = body
    .split("\n")
    .filter((line) => line.startsWith(prefix) && line.endsWith(" -->"));
  assert.equal(values.length, 1, `Missing/ambiguous ${key}`);
  return values[0].slice(prefix.length, -4);
}
function version(content) {
  const section = content.split("[workspace.package]")[1]?.split("\n[")[0];
  const values = [
    ...(section ?? "").matchAll(/^version = "([0-9]+\.[0-9]+\.[0-9]+(?:-[0-9A-Za-z.-]+)?)"$/gmu),
  ];
  assert.equal(values.length, 1, "Exact workspace version is required");
  return values[0][1];
}

export function verifyVersionCut(root, cut, source, old, next) {
  assert.equal(version(object(root, cut, "Cargo.toml")), old);
  assert.equal(version(object(root, source, "Cargo.toml")), next);
  assert.notEqual(old, next);
  const entries = git(root, "diff", "--raw", "--no-renames", "--no-abbrev", cut, source)
    .trim()
    .split("\n");
  for (const row of entries) {
    const [metadata, path] = row.split("\t");
    assert.match(
      metadata,
      /^:100644 100644 [0-9a-f]{40} [0-9a-f]{40} M$/u,
      "Only existing metadata file bytes may change",
    );
    const before = object(root, cut, path);
    const expected = rewriteVersionMetadata(path, before, old, next);
    assert.notEqual(expected, before, `Non-version edit: ${path}`);
    assert.equal(object(root, source, path), expected, `Altered generated metadata: ${path}`);
  }
  for (const path of git(root, "ls-tree", "-r", "--name-only", "-z", cut)
    .split("\0")
    .filter(metadataCandidate)) {
    const before = object(root, cut, path),
      expected = rewriteVersionMetadata(path, before, old, next);
    if (before !== expected)
      assert.equal(object(root, source, path), expected, `Omitted generated metadata: ${path}`);
  }
  return entries;
}

export function selectSourceRecipe({ root, env, event, permission, pin, parse }) {
  const source = sha(env.SOURCE_SHA),
    workflow = sha(env.GITHUB_WORKFLOW_SHA);
  assert.equal(git(root, "rev-parse", "HEAD").trim(), source);
  assert.equal(env.DRIVER_SHA, source);
  assert.equal(env.GITHUB_EVENT_NAME, "pull_request");
  assert.match(
    env.GITHUB_WORKFLOW_REF ?? "",
    new RegExp(`^${REPO}/${WORKFLOW}@refs/pull/[0-9]+/merge$`, "u"),
  );
  const pr = event.pull_request;
  assert.equal(pr?.head?.sha, source);
  const ordinary = {
    schema: 1,
    mode: "current-inline",
    source,
    sourceTree: tree(root, source),
    workflow,
    recipe: null,
  };
  if (!pr.head.ref.startsWith("release/v") && !pr.body?.includes("<!-- vize-release-pin:"))
    return ordinary;
  assert.equal(env.GITHUB_REPOSITORY, REPO);
  for (const side of [pr.head, pr.base]) assert.equal(side.repo.full_name, REPO);
  assert.equal(pr.base.ref, "main");
  assert.equal(pr.state, "open");
  assert.equal(pr.draft, true);
  assert.equal(pr.merged, false);
  assert.ok(
    ["maintain", "admin"].includes(permission?.role_name),
    "Release author requires maintain/admin permission",
  );
  assert.equal(marker(pr.body, "vize-release-pin"), "immutable-v1");
  assert.equal(marker(pr.body, "vize-release-pin-head"), source);
  const cut = sha(marker(pr.body, "vize-release-pin-cut"));
  const old = marker(pr.body, "vize-release-base-version");
  const next = version(object(root, source, "Cargo.toml"));
  assert.equal(pr.head.ref, `release/v${next}`);
  const integration = Number(marker(pr.body, "vize-release-integration"));
  assert.ok(Number.isSafeInteger(integration) && integration > 0 && integration !== pr.number);
  assert.equal(git(root, "rev-list", "--parents", "-n", "1", source).trim(), `${source} ${cut}`);
  assert.equal(env.MAIN_SOURCE_SHA, cut);
  git(root, "merge-base", "--is-ancestor", cut, sha(env.MAIN_HEAD_SHA));
  assert.equal(git(root, "rev-list", "--parents", "-n", "1", sha(pin)).trim(), `${pin} ${source}`);
  assert.equal(tree(root, pin), tree(root, source));
  const expectedPin = `Vize immutable release cut\n\nSource-PR: #${pr.number}\nSource-head: ${source}\nSource-cut: ${cut}\nTag: v${next}\nBase-version: ${old}\nIntegration-PR: #${integration}`;
  assert.equal(
    git(root, "cat-file", "-p", pin).split("\n\n").slice(1).join("\n\n").trimEnd(),
    expectedPin,
  );
  const changes = verifyVersionCut(root, cut, source, old, next);
  const bytes = object(root, source, WORKFLOW);
  assert.equal(
    bytes,
    object(root, cut, WORKFLOW),
    "Source native workflow must equal its genuine cut",
  );
  const steps = parse(bytes).jobs["native-phases"].steps.filter((step) => step.name === STEP);
  assert.equal(steps.length, 1);
  const step = steps[0];
  assert.equal(step.if, "github.event_name == 'pull_request'");
  assert.equal(step.env.VIZE_TEST_REQUIRE_TSGO, "1");
  assert.ok(
    step.run.startsWith(
      'set -euo pipefail\nunset VIZE_TEST_DISABLE_TSGO\ncd "$NATIVE_PHASE_SOURCE_ROOT"\n',
    ),
  );
  return {
    ...ordinary,
    mode: "immutable-source-step",
    cut,
    cutTree: tree(root, cut),
    pin,
    pinTree: tree(root, pin),
    changes,
    sourceWorkflowSha256: hash(bytes),
    recipeSha256: hash(step.run),
    recipe: step.run,
    originalEnvironment: step.env,
    authentication: { pullRequest: pr, permission, pinCommit: git(root, "cat-file", "-p", pin) },
    unavailableCurrentCapture: "tsconfig-types-extends",
  };
}

async function main() {
  const env = process.env,
    root = realpathSync(env.NATIVE_PHASE_SOURCE_ROOT);
  const require = createRequire(join(root, "package.json"));
  const parse = require("yaml").parse;
  const helperPath = "tools/benchmarks/scripts/typechecker-native-source-recipe.mjs";
  const metadataPath = "tools/benchmarks/scripts/typechecker-native-release-metadata.mjs";
  const workflow = sha(env.GITHUB_WORKFLOW_SHA);
  assert.equal(
    readFileSync(fileURLToPath(import.meta.url), "utf8"),
    object(root, workflow, helperPath),
  );
  assert.equal(
    readFileSync(new URL("./typechecker-native-release-metadata.mjs", import.meta.url), "utf8"),
    object(root, workflow, metadataPath),
  );
  const event = JSON.parse(readFileSync(env.GITHUB_EVENT_PATH, "utf8"));
  let permission, pin;
  if (
    event.pull_request?.head.ref.startsWith("release/v") ||
    event.pull_request?.body?.includes("<!-- vize-release-pin:")
  ) {
    const author = event.pull_request.user.login;
    assert.match(author, /^[A-Za-z0-9-]+$/u);
    const response = await fetch(
      `https://api.github.com/repos/${REPO}/collaborators/${author}/permission`,
      {
        headers: { Authorization: `Bearer ${env.GH_TOKEN}`, Accept: "application/vnd.github+json" },
        signal: AbortSignal.timeout(30_000),
      },
    );
    assert.equal(response.status, 200, "Release maintainer permission could not be authenticated");
    permission = await response.json();
    git(
      root,
      "fetch",
      "--no-tags",
      "origin",
      `refs/heads/release-pin/v${version(object(root, env.SOURCE_SHA, "Cargo.toml"))}`,
    );
    pin = git(root, "rev-parse", "FETCH_HEAD").trim();
    const remote = git(
      root,
      "ls-remote",
      "--heads",
      "origin",
      `refs/heads/${event.pull_request.head.ref}`,
    ).split(/\s+/u)[0];
    assert.equal(remote, env.SOURCE_SHA, "Immutable source branch changed");
  }
  const selected = selectSourceRecipe({ root, env, event, permission, pin, parse });
  if (selected.recipe !== null) {
    for (const [key, value] of Object.entries(selected.originalEnvironment)) {
      assert.equal(typeof value, "string", `Unrecognized original environment ${key}`);
      const expected = value.replaceAll("${{ runner.temp }}", env.RUNNER_TEMP);
      assert.ok(!expected.includes("${{"), `Unresolved original environment ${key}`);
      assert.equal(env[key], expected, `Original native environment changed: ${key}`);
    }
  }
  const recipePath =
    selected.recipe === null ? "" : join(env.RUNNER_TEMP, "native-source-recipe.sh");
  if (recipePath) writeFileSync(recipePath, selected.recipe, { flag: "wx" });
  writeFileSync(
    join(env.RUNNER_TEMP, "native-source-recipe.json"),
    JSON.stringify(selected, null, 2) + "\n",
  );
  process.stdout.write(recipePath);
}
if (
  process.argv[1] &&
  realpathSync(process.argv[1]) === realpathSync(fileURLToPath(import.meta.url))
)
  await main();
