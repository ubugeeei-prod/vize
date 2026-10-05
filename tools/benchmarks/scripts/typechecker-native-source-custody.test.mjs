/** Real temporary checkouts exercise source/driver custody before any benchmark builds. */
import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import {
  chmodSync,
  mkdirSync,
  mkdtempSync,
  realpathSync,
  rmSync,
  symlinkSync,
  writeFileSync,
} from "node:fs";
import os from "node:os";
import { dirname, join } from "node:path";
import test from "node:test";
import {
  captureSourceCustody,
  INFRASTRUCTURE_PATHS,
} from "./typechecker-native-source-custody.mjs";

const WORKFLOW = ".github/workflows/typechecker-native-phases.yml";
const PREFIX = "tools/benchmarks/scripts/";
const ORIGINAL = [
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
  ].map((file) => PREFIX + file),
];
const BRIDGES = [
  "Cargo.toml",
  "Cargo.lock",
  "package.json",
  "pnpm-lock.yaml",
  ...["corpus", "leaf-corpus", "protocol"].map((name) => PREFIX + `type-snapshot-cli-${name}.mjs`),
];
const PRODUCTION = "crates/vize_canon/src/lib.rs";

function git(root, ...args) {
  const result = spawnSync("git", args, { cwd: root, encoding: "utf8", timeout: 30_000 });
  assert.equal(result.error, undefined, result.error?.message);
  assert.equal(result.status, 0, result.stderr);
  return result.stdout.trim();
}
function put(root, path, text = "changed fixture\n") {
  mkdirSync(dirname(join(root, path)), { recursive: true });
  writeFileSync(join(root, path), text);
}
function commit(root, message) {
  git(root, "add", "-A");
  git(root, "commit", "-m", message);
  return git(root, "rev-parse", "HEAD");
}
function fixture(run) {
  const temporary = mkdtempSync(join(os.tmpdir(), "native-source-custody-"));
  const root = join(temporary, "driver");
  mkdirSync(root);
  try {
    git(root, "init", "--initial-branch=main");
    git(root, "config", "user.name", "Custody fixture");
    git(root, "config", "user.email", "custody-fixture@example.invalid");
    git(root, "config", "commit.gpgsign", "false");
    for (const path of new Set([...INFRASTRUCTURE_PATHS, ...BRIDGES, PRODUCTION]))
      put(root, path, `original ${path}\n`);
    put(root, ".gitignore", "target/\nnode_modules/\n");
    const base = commit(root, "fixture source");
    git(root, "update-ref", "refs/remotes/origin/main", base);
    run({ root, temporary, base });
  } finally {
    rmSync(temporary, { recursive: true, force: true });
  }
}
function manual(f, { source = f.base, main = git(f.root, "rev-parse", "HEAD"), env = {} } = {}) {
  const sourceRoot = join(f.temporary, "source");
  git(f.root, "worktree", "add", "--detach", sourceRoot, source);
  git(f.root, "update-ref", "refs/remotes/origin/main", main);
  return {
    driverRoot: f.root,
    sourceRoot,
    env: {
      GITHUB_EVENT_NAME: "workflow_dispatch",
      GITHUB_REPOSITORY: "ubugeeei-prod/vize",
      GITHUB_REF: "refs/heads/main",
      GITHUB_WORKFLOW_REF: `ubugeeei-prod/vize/${WORKFLOW}@refs/heads/main`,
      GITHUB_WORKFLOW_SHA: git(f.root, "rev-parse", "HEAD"),
      GITHUB_RUN_ID: "12345",
      GITHUB_RUN_ATTEMPT: "2",
      DRIVER_SHA: git(f.root, "rev-parse", "HEAD"),
      SOURCE_SHA: source,
      MAIN_HEAD_SHA: main,
      MAIN_SOURCE_SHA: source,
      ...env,
    },
  };
}

await test("manual identical main uses separate source and records the complete custody", () => {
  fixture((f) => {
    const options = manual(f);
    const receipt = captureSourceCustody(options);
    assert.equal(receipt.source.sha, f.base);
    assert.equal(receipt.driver.sha, f.base);
    assert.notEqual(receipt.source.root, receipt.driver.root);
    assert.equal(receipt.source.tree, git(f.root, "rev-parse", `${f.base}^{tree}`));
    assert.equal(receipt.driver.tree, receipt.source.tree);
    assert.equal(receipt.main.sha, f.base);
    assert.equal(receipt.baseline.sha, f.base);
    assert.equal(receipt.baseline.prBaseSha, null);
    assert.equal(receipt.productionMatchesBaseline, true);
    assert.deepEqual(receipt.changedEntries, []);
    assert.deepEqual(receipt.originalInfrastructurePaths, ORIGINAL);
    assert.equal(receipt.allowedInfrastructurePaths.length, 24);
    assert.deepEqual(Object.keys(receipt.bridgeFiles), BRIDGES);
    for (const hashes of Object.values(receipt.bridgeFiles))
      assert.equal(hashes.sourceSha256, hashes.driverSha256);
    assert.equal(receipt.workflow.sha, f.base);
    assert.match(receipt.workflow.driverFileSha256, /^[0-9a-f]{64}$/u);
    assert.equal(receipt.runId, "12345");
    assert.equal(receipt.runAttempt, "2");
    assert.equal(receipt.nativeProfile, "pprof-untimed-duplicate-corpus");
    assert.equal(receipt.build.root, realpathSync(options.sourceRoot));
  });
});

await test("manual permits precisely reviewed infrastructure while preserving neutral bridges", () => {
  fixture((f) => {
    const paths = [...INFRASTRUCTURE_PATHS].filter((path) => !BRIDGES.includes(path));
    assert.equal(paths.length, 21);
    for (const path of paths) put(f.root, path);
    const executable = PREFIX + "typechecker-native-phase-capture.mjs";
    chmodSync(join(f.root, executable), 0o755);
    const driver = commit(f.root, "benchmark infrastructure only");
    const receipt = captureSourceCustody(manual(f));
    assert.equal(receipt.source.sha, f.base);
    assert.equal(receipt.driver.sha, driver);
    assert.notEqual(receipt.source.tree, receipt.driver.tree);
    assert.equal(receipt.productionMatchesBaseline, true);
    assert.deepEqual(new Set(receipt.changedPaths), new Set(paths));
    assert.equal(receipt.changedPaths.length, paths.length);
    const entry = receipt.changedEntries.find((change) => change.path === executable);
    assert.equal(entry.beforeMode, "100644");
    assert.equal(entry.afterMode, "100755");
    assert.notEqual(entry.beforeBlob, entry.afterBlob);
  });
});

for (const path of [
  PRODUCTION,
  PREFIX + "typechecker-native-unreviewed.mjs",
  PREFIX + "typechecker-native-source-custody.mjs\nCargo.lock",
  ...BRIDGES,
]) {
  await test(`manual rejects production, dependency or neutral-input change: ${JSON.stringify(path)}`, () => {
    fixture((f) => {
      put(f.root, path);
      commit(f.root, "hostile input change");
      assert.throws(
        () => captureSourceCustody(manual(f)),
        /production inputs|dependency or corpus drift/u,
      );
    });
  });
}

await test("manual rejects a symlink even at an explicitly allowed infrastructure path", () => {
  fixture((f) => {
    const path = PREFIX + "typechecker-native-phase-capture.mjs";
    rmSync(join(f.root, path));
    symlinkSync("../../../" + PRODUCTION, join(f.root, path));
    commit(f.root, "symlink is not infrastructure byte custody");
    assert.throws(() => captureSourceCustody(manual(f)), /production inputs/u);
  });
});

/** @type {[string, string, RegExp][]} */
const invalidContexts = [
  ["SOURCE_SHA", "main", /immutable lowercase SHA/u],
  ["SOURCE_SHA", "abc1234", /immutable lowercase SHA/u],
  ["SOURCE_SHA", "A".repeat(40), /immutable lowercase SHA/u],
  ["SOURCE_SHA", "f".repeat(40), /expected|needed|unknown|ambiguous|bad/iu],
  ["GITHUB_EVENT_NAME", "push", /unsupported source event/u],
  ["GITHUB_EVENT_NAME", "pull_request_target", /unsupported source event/u],
  ["GITHUB_REF", "refs/heads/experiment", /protected main/u],
  ["GITHUB_REF", "refs/tags/main", /protected main/u],
  ["GITHUB_REPOSITORY", "someone/vize", /ubugeeei-prod\/vize/u],
  ["GITHUB_RUN_ID", "", /run identity missing/u],
  ["GITHUB_RUN_ATTEMPT", "0", /run identity missing/u],
  ["MAIN_SOURCE_SHA", "0".repeat(40), /source baseline mismatch/u],
];
for (const [field, value, reason] of invalidContexts) {
  await test(`manual rejects invalid context ${field}=${JSON.stringify(value)}`, () => {
    fixture((f) =>
      assert.throws(() => captureSourceCustody(manual(f, { env: { [field]: value } })), reason),
    );
  });
}

await test("manual rejects source/driver HEAD and workflow/driver mismatches independently", () => {
  fixture((f) => {
    put(f.root, PREFIX + "typechecker-native-phase-report.mjs");
    const driver = commit(f.root, "reviewed driver revision");
    const options = manual(f);
    assert.throws(
      () => captureSourceCustody({ ...options, sourceRoot: f.root }),
      /source HEAD mismatch/u,
    );
    assert.throws(
      () => captureSourceCustody({ ...options, env: { ...options.env, DRIVER_SHA: f.base } }),
      /driver HEAD mismatch/u,
    );
    assert.throws(
      () =>
        captureSourceCustody({ ...options, env: { ...options.env, GITHUB_WORKFLOW_SHA: f.base } }),
      /workflow\/driver mismatch/u,
    );
    assert.equal(options.env.DRIVER_SHA, driver);
  });
});

await test("manual declines a shared checkout even when its source SHA is correct", () => {
  fixture((f) => {
    const options = manual(f);
    assert.throws(
      () => captureSourceCustody({ ...options, sourceRoot: f.root }),
      /separate checkout/u,
    );
  });
});

for (const side of ["sourceRoot", "driverRoot"]) {
  for (const state of ["tracked", "staged", "untracked"]) {
    await test(`manual rejects ${state} build inputs in ${side}`, () => {
      fixture((f) => {
        const options = manual(f);
        const path = state === "untracked" ? "crates/vize/build.rs" : PRODUCTION;
        put(options[side], path, "uncommitted build input\n");
        if (state === "staged") git(options[side], "add", path);
        assert.throws(
          () => captureSourceCustody(options),
          /dirty tracked source|untracked source/u,
        );
      });
    });
  }
}

await test("manual preserves ignored build and dependency output directories", () => {
  fixture((f) => {
    const options = manual(f);
    for (const root of [options.sourceRoot, options.driverRoot])
      for (const path of ["target/build-output", "node_modules/runtime-output"]) put(root, path);
    assert.equal(captureSourceCustody(options).productionMatchesBaseline, true);
  });
});

for (const mode of ["stale-env", "forged-main", "missing-ref"]) {
  await test(`manual binds recorded main to the actual fetched origin/main: ${mode}`, () => {
    fixture((f) => {
      put(f.root, PREFIX + "typechecker-native-phase-report.mjs");
      commit(f.root, "reviewed main driver");
      const options = manual(f);
      if (mode === "stale-env") options.env.MAIN_HEAD_SHA = f.base;
      if (mode === "forged-main") git(f.root, "update-ref", "refs/remotes/origin/main", f.base);
      if (mode === "missing-ref") git(f.root, "update-ref", "-d", "refs/remotes/origin/main");
      assert.throws(
        () => captureSourceCustody(options),
        mode === "missing-ref" ? undefined : /fetched origin\/main/u,
      );
    });
  });
}

await test("manual rejects a driver outside the recorded main history", () => {
  fixture((f) => {
    git(f.root, "checkout", "-b", "unmerged-driver");
    put(f.root, PREFIX + "typechecker-native-phase-report.mjs");
    commit(f.root, "unmerged infrastructure");
    assert.throws(() => captureSourceCustody(manual(f, { main: f.base })));
  });
});

await test("manual rejects an arbitrary source branch even with a protected-main driver", () => {
  fixture((f) => {
    git(f.root, "checkout", "-b", "unmerged-source");
    put(f.root, PRODUCTION);
    const source = commit(f.root, "unmerged source");
    git(f.root, "checkout", "main");
    assert.throws(() => captureSourceCustody(manual(f, { source, main: f.base })));
  });
});

await test("PR keeps origin-main merge-base rather than the immediate stacked PR base", () => {
  fixture((f) => {
    git(f.root, "checkout", "-b", "stack-parent");
    put(f.root, PRODUCTION, "parent production revision\n");
    const parent = commit(f.root, "stack parent");
    put(f.root, PREFIX + "typechecker-native-phase-report.mjs");
    const head = commit(f.root, "stack child");
    git(f.root, "checkout", "main");
    put(f.root, "docs/main-progress.md");
    const main = commit(f.root, "main progressed independently");
    git(f.root, "update-ref", "refs/remotes/origin/main", main);
    git(f.root, "checkout", "--detach", head);
    const options = {
      driverRoot: f.root,
      env: {
        GITHUB_EVENT_NAME: "pull_request",
        SOURCE_SHA: head,
        MAIN_HEAD_SHA: main,
        MAIN_SOURCE_SHA: f.base,
        PR_BASE_SHA: parent,
      },
    };
    const receipt = captureSourceCustody(options);
    assert.equal(receipt.baseline.sha, f.base);
    assert.equal(receipt.baseline.prBaseSha, parent);
    assert.equal(receipt.productionMatchesBaseline, false);
    assert.equal(receipt.nativeProfile, "none");
    assert(receipt.changedPaths.includes(PRODUCTION));
    assert.equal(receipt.driver.sha, head);
    assert.equal(receipt.source.sha, head);
    assert.throws(
      () => captureSourceCustody({ ...options, env: { ...options.env, MAIN_SOURCE_SHA: parent } }),
      /source baseline mismatch/u,
    );
    assert.throws(
      () => captureSourceCustody({ ...options, env: { ...options.env, DRIVER_SHA: parent } }),
      /driver HEAD mismatch/u,
    );
  });
});
