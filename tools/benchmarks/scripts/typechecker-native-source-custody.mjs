/** Pin distinct benchmark-driver and production-source trees before building. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  closeSync,
  constants,
  fstatSync,
  lstatSync,
  openSync,
  readFileSync,
  readlinkSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const WORKFLOW = ".github/workflows/typechecker-native-phases.yml";
const ORIGINAL_INFRASTRUCTURE = [
  WORKFLOW,
  "crates/vize_canon/examples/native_phase_projection.rs",
  "docs/davinci/decisions/2026-09-27-level-restructure.md",
  "docs/davinci/decisions/2026-10-04-typechecker-native-phases.md",
  ...[
    "typechecker-native-phases.mjs",
    "typechecker-native-phase-capture.mjs",
    "typechecker-native-phase-report.mjs",
    "typechecker-native-phase-runner.mjs",
    "typechecker-native-phase-forwarding.test.mjs",
    "type-snapshot-cli-corpus.mjs",
    "type-snapshot-cli-leaf-corpus.mjs",
    "type-snapshot-cli-protocol.mjs",
  ].map((file) => "tools/benchmarks/scripts/" + file),
];
assert.equal(ORIGINAL_INFRASTRUCTURE.length, 12);
export const INFRASTRUCTURE_PATHS = new Set([
  ...ORIGINAL_INFRASTRUCTURE,
  "docs/davinci/decisions/2026-10-04-typechecker-cold-native-profiles.md",
  "docs/davinci/decisions/2026-10-04-shared-leaf-native-trivia.md",
  "tools/benchmarks/scripts/typechecker-native-profile-corpus-fixture.mjs",
  "tools/benchmarks/scripts/typechecker-native-dependency-link.test.mjs",
  ...[
    "typechecker-native-graph-archive",
    "typechecker-native-profile-replay",
    "typechecker-native-profile-corpus",
    "typechecker-native-source-custody",
  ]
    .flatMap((file) => [file + ".mjs", file + ".test.mjs"])
    .map((file) => "tools/benchmarks/scripts/" + file),
]);
const BRIDGE_FILES = [
  "Cargo.toml",
  "Cargo.lock",
  "package.json",
  "pnpm-lock.yaml",
  ...["corpus", "leaf-corpus", "protocol"].map(
    (name) => "tools/benchmarks/scripts/type-snapshot-cli-" + name + ".mjs",
  ),
];
const sha256 = (bytes) => createHash("sha256").update(bytes).digest("hex");
function git(args, cwd) {
  const result = spawnSync("git", args, { cwd, encoding: "utf8", timeout: 60_000 });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout;
}
function commit(sha, root) {
  assert.match(sha ?? "", /^[0-9a-f]{40}$/u, "commit must be an immutable lowercase SHA");
  assert.equal(git(["rev-parse", "--verify", sha + "^{commit}"], root).trim(), sha);
  return sha;
}
function tree(sha, root) {
  return git(["rev-parse", sha + "^{tree}"], root).trim();
}
function changes(from, to, root) {
  const raw = git(
    ["diff", "--raw", "-z", "--no-renames", "--abbrev=40", from, to, "--"],
    root,
  ).split("\0");
  assert.equal(raw.pop(), "");
  const entries = [];
  for (let index = 0; index < raw.length; index += 2) {
    const match = raw[index].match(
      /^:([0-7]{6}) ([0-7]{6}) ([0-9a-f]{40}) ([0-9a-f]{40}) ([ADMTUX])$/u,
    );
    assert(match && raw[index + 1], "unsupported raw source delta");
    entries.push({
      path: raw[index + 1],
      status: match[5],
      beforeMode: match[1],
      afterMode: match[2],
      beforeBlob: match[3],
      afterBlob: match[4],
    });
  }
  return entries;
}
function infrastructureOnly(entries) {
  return entries.every(
    (entry) =>
      INFRASTRUCTURE_PATHS.has(entry.path) &&
      [entry.beforeMode, entry.afterMode].every((mode) =>
        ["000000", "100644", "100755"].includes(mode),
      ),
  );
}
const revision = (stat, fields) =>
  Object.fromEntries(fields.map((field) => [field, stat[field].toString()]));
const LINK_REVISION = ["dev", "ino", "mode", "size", "mtimeNs", "ctimeNs"];
const DIRECTORY_IDENTITY = ["dev", "ino", "mode"];
function dependencyLink(sourceRoot, driverRoot) {
  const path = join(sourceRoot, "node_modules");
  const target = join(driverRoot, "node_modules");
  let targetBefore;
  try {
    targetBefore = lstatSync(target, { bigint: true });
    assert(targetBefore.isDirectory(), "driver dependency target must be a physical directory");
  } catch (error) {
    if (error.code !== "ENOENT") throw error;
  }
  let before;
  try {
    before = lstatSync(path, { bigint: true });
  } catch (error) {
    if (error.code === "ENOENT") return null;
    throw error;
  }
  // Keep the existing ignored physical-directory contract. Only the workflow's
  // untracked manual symlink receives the narrowly verified exception below.
  if (!before.isSymbolicLink()) return null;
  assert.equal(readlinkSync(path), target, "source dependency link target mismatch");
  assert(targetBefore, "driver dependency target is absent");
  const descriptor = openSync(
    target,
    constants.O_RDONLY | constants.O_DIRECTORY | constants.O_NOFOLLOW,
  );
  try {
    const opened = fstatSync(descriptor, { bigint: true });
    assert(opened.isDirectory(), "opened driver dependency target is not a directory");
    assert.deepEqual(
      revision(opened, DIRECTORY_IDENTITY),
      revision(targetBefore, DIRECTORY_IDENTITY),
    );
    assert.equal(realpathSync(path), target, "source dependency link canonical target mismatch");
    assert.deepEqual(
      revision(lstatSync(target, { bigint: true }), DIRECTORY_IDENTITY),
      revision(opened, DIRECTORY_IDENTITY),
      "driver dependency directory changed during custody",
    );
    assert.equal(readlinkSync(path), target, "source dependency link retargeted during custody");
    const after = lstatSync(path, { bigint: true });
    assert(after.isSymbolicLink(), "source dependency link replaced during custody");
    assert.deepEqual(
      revision(after, LINK_REVISION),
      revision(before, LINK_REVISION),
      "source dependency link changed during custody",
    );
    return {
      path,
      target,
      canonicalTarget: target,
      revision: revision(before, LINK_REVISION),
      driverDirectory: { path: target, ...revision(opened, DIRECTORY_IDENTITY) },
    };
  } finally {
    closeSync(descriptor);
  }
}
export function captureSourceCustody({ driverRoot, sourceRoot = driverRoot, env = process.env }) {
  driverRoot = realpathSync(driverRoot);
  sourceRoot = realpathSync(sourceRoot);
  const event = env.GITHUB_EVENT_NAME;
  assert(["pull_request", "workflow_dispatch"].includes(event), "unsupported source event");
  const driverSha = commit(env.DRIVER_SHA ?? env.SOURCE_SHA, driverRoot);
  const sourceSha = commit(env.SOURCE_SHA, driverRoot);
  const mainHeadSha = commit(env.MAIN_HEAD_SHA, driverRoot);
  assert.equal(
    git(["rev-parse", "refs/remotes/origin/main"], driverRoot).trim(),
    mainHeadSha,
    "main environment does not match fetched origin/main",
  );
  assert.equal(git(["rev-parse", "HEAD"], driverRoot).trim(), driverSha, "driver HEAD mismatch");
  assert.equal(git(["rev-parse", "HEAD"], sourceRoot).trim(), sourceSha, "source HEAD mismatch");
  for (const root of new Set([driverRoot, sourceRoot])) {
    assert.equal(git(["diff", "--name-only", "HEAD"], root).trim(), "", "dirty tracked source");
  }
  const manual = event === "workflow_dispatch";
  let baselineSha;
  let prBaseSha = null;
  if (manual) {
    assert.notEqual(sourceRoot, driverRoot, "manual source must have a separate checkout");
    assert.equal(env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
    assert.equal(env.GITHUB_REF, "refs/heads/main", "manual driver must run on protected main");
    assert.equal(
      commit(env.GITHUB_WORKFLOW_SHA, driverRoot),
      driverSha,
      "workflow/driver mismatch",
    );
    for (const value of [env.GITHUB_RUN_ID, env.GITHUB_RUN_ATTEMPT])
      assert.match(value ?? "", /^[1-9][0-9]*$/u, "manual run identity missing");
    git(["merge-base", "--is-ancestor", driverSha, mainHeadSha], driverRoot);
    git(["merge-base", "--is-ancestor", sourceSha, mainHeadSha], driverRoot);
    baselineSha = sourceSha;
  } else {
    assert.equal(driverSha, sourceSha, "PR driver must use the exact PR head");
    assert.equal(sourceRoot, driverRoot, "PR source checkout contract changed");
    prBaseSha = commit(env.PR_BASE_SHA, driverRoot);
    baselineSha = git(["merge-base", mainHeadSha, sourceSha], driverRoot).trim();
  }
  assert.equal(env.MAIN_SOURCE_SHA, baselineSha, "source baseline mismatch");
  const driverDelta = changes(sourceSha, driverSha, driverRoot);
  const sourceDelta = changes(baselineSha, sourceSha, driverRoot);
  if (manual) assert(infrastructureOnly(driverDelta), "manual driver differs in production inputs");
  const bridgeFiles = {};
  for (const file of BRIDGE_FILES) {
    const source = join(sourceRoot, file);
    const driver = join(driverRoot, file);
    assert(lstatSync(source).isFile() && lstatSync(driver).isFile(), "bridge input is not regular");
    const sourceHash = sha256(readFileSync(source));
    const driverHash = sha256(readFileSync(driver));
    if (manual)
      assert.equal(sourceHash, driverHash, "driver/source dependency or corpus drift: " + file);
    bridgeFiles[file] = { sourceSha256: sourceHash, driverSha256: driverHash };
  }
  const sourceDependencyLink = manual ? dependencyLink(sourceRoot, driverRoot) : null;
  for (const root of new Set([driverRoot, sourceRoot])) {
    const untracked = git(["ls-files", "--others", "--exclude-standard", "-z"], root);
    const paths = untracked ? untracked.split("\0") : [];
    if (paths.length) assert.equal(paths.pop(), "", "invalid untracked-input framing");
    assert.deepEqual(
      paths.filter(
        (path) => !(root === sourceRoot && sourceDependencyLink && path === "node_modules"),
      ),
      [],
      "untracked source inputs",
    );
  }
  return {
    schemaVersion: 1,
    repository: env.GITHUB_REPOSITORY ?? null,
    event,
    runId: env.GITHUB_RUN_ID ?? null,
    runAttempt: env.GITHUB_RUN_ATTEMPT ?? null,
    workflow: {
      ref: env.GITHUB_WORKFLOW_REF ?? null,
      sha: env.GITHUB_WORKFLOW_SHA ?? null,
      driverFileSha256: sha256(readFileSync(join(driverRoot, WORKFLOW))),
      trust: manual
        ? "exact protected-main workflow and ancestry"
        : "existing exact PR-head driver",
    },
    driver: {
      root: driverRoot,
      sha: driverSha,
      tree: tree(driverSha, driverRoot),
      delta: driverDelta,
    },
    source: { root: sourceRoot, sha: sourceSha, tree: tree(sourceSha, sourceRoot) },
    main: { sha: mainHeadSha, tree: tree(mainHeadSha, driverRoot) },
    baseline: { sha: baselineSha, tree: tree(baselineSha, driverRoot), prBaseSha },
    productionMatchesBaseline: infrastructureOnly(manual ? driverDelta : sourceDelta),
    changedPaths: (manual ? driverDelta : sourceDelta).map((entry) => entry.path),
    changedEntries: manual ? driverDelta : sourceDelta,
    bridgeFiles,
    sourceDependencyLink,
    nativeProfile: manual ? "pprof-untimed-duplicate-corpus" : "none",
    originalInfrastructurePaths: ORIGINAL_INFRASTRUCTURE,
    allowedInfrastructurePaths: [...INFRASTRUCTURE_PATHS],
    build: {
      root: sourceRoot,
      projectionExample: {
        path: "crates/vize_canon/examples/native_phase_projection.rs",
        sourceSha256: sha256(
          readFileSync(join(sourceRoot, "crates/vize_canon/examples/native_phase_projection.rs")),
        ),
        driverSha256: sha256(
          readFileSync(join(driverRoot, "crates/vize_canon/examples/native_phase_projection.rs")),
        ),
      },
      cargoProfile: "ci-opt",
      inherits: "release",
      lto: "thin",
      codegenUnits: 16,
      stripVize: "symbols",
      incremental: false,
      crates: ["vize", "vize_canon --example native_phase_projection"],
    },
  };
}
if (process.argv[1] && resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  assert.equal(process.argv.length, 3, "usage: source-custody.mjs OUTPUT");
  const receipt = captureSourceCustody({
    driverRoot: fileURLToPath(new URL("../../..", import.meta.url)),
    sourceRoot: process.env.NATIVE_PHASE_SOURCE_ROOT,
  });
  writeFileSync(process.argv[2], JSON.stringify(receipt, null, 2) + "\n");
}
