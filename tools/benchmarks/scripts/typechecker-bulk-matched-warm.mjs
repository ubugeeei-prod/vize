import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import {
  copyFileSync,
  existsSync,
  mkdirSync,
  mkdtempSync,
  readFileSync,
  readdirSync,
  readlinkSync,
  realpathSync,
  writeFileSync,
} from "node:fs";
import { join, resolve } from "node:path";

// The actual parent-only main retains the original route and all incoming fixes.
const baseline = "58e6a0272b4044e3aaa8b7a9cb3ac62100ccec7c";
const root = process.cwd();
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const source = git("rev-parse", "HEAD");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
assert.equal(source, process.env.GITHUB_SHA);
assert.equal(process.env.GITHUB_REPOSITORY, "ubugeeei-prod/vize");
const ancestry = JSON.parse(
  execFileSync(
    "gh",
    [
      "api",
      `repos/ubugeeei-prod/vize/compare/${baseline}...${source}`,
      "--jq",
      "{status,base:.base_commit.sha,mergeBase:.merge_base_commit.sha}",
    ],
    { encoding: "utf8" },
  ),
);
assert.equal(ancestry.base, baseline);
assert.equal(ancestry.mergeBase, baseline);
assert(["ahead", "identical"].includes(ancestry.status));
// Fetch only this immutable tree, including on a shallow source checkout.
git("fetch", "--no-tags", "--depth=1", "origin", baseline);
const authority = ["Cargo.toml", "Cargo.lock", "crates/vize_canon/Cargo.toml", "pnpm-lock.yaml"];
assert.equal(git("diff", baseline, source, "--", ...authority), "");
const fixture = realpathSync(resolve(root, process.env.VIZE_TIER_L_FIXTURE));
const native = realpathSync(resolve(root, process.env.VIZE_TIER_L_CORSA_BIN));
const runtime = realpathSync(resolve(root, process.env.VIZE_RUNTIME_NODE_MODULES));
const output = resolve(root, process.env.VIZE_TIER_L_BULK_CAPTURE_DIR, "matched-warm");
mkdirSync(output, { recursive: true });
const temporary = mkdtempSync(join(process.env.RUNNER_TEMP, "bulk-matched-"));
const parent = join(temporary, "original-source");
const driverPaths = [
  "crates/vize_canon/tests/tier_l_incremental.rs",
  ...readdirSync(join(root, "crates/vize_canon/tests/support"))
    .filter((name) => name.startsWith("tier_l_") && name.endsWith(".rs"))
    .sort()
    .map((name) => `crates/vize_canon/tests/support/${name}`),
];
const drivers = driverPaths.map((path) => ({
  path,
  sha256: digest(readFileSync(join(root, path))),
}));
const registryPath = "tests/_fixtures/vue-ecosystem-fixtures.json";
const project = (path) =>
  JSON.parse(readFileSync(path, "utf8")).projects.find((item) => item.id === "vue-vben-admin");
const packets = [];
const receipt = {
  schemaVersion: 1,
  source,
  baseline,
  ancestry,
  driverOverlay: drivers,
  authority: authority.map((path) => ({ path, sha256: digest(readFileSync(join(root, path))) })),
  fixtureRoot: fixture,
  fixtureRevision: execFileSync("git", ["rev-parse", "HEAD"], {
    cwd: fixture,
    encoding: "utf8",
  }).trim(),
  fixtureAuthority: project(join(root, registryPath)),
  nativeBinary: native,
  nativeSha256: digest(readFileSync(native)),
  runtimeNodeModules: runtime,
  budgetScale: process.env.VIZE_TIER_L_BUDGET_SCALE,
  pairs: packets,
};
const persist = () =>
  writeFileSync(join(output, "receipt.json"), JSON.stringify(receipt, null, 2) + "\n");
const nativeProcesses = () =>
  readdirSync("/proc")
    .filter((name) => /^\d+$/.test(name))
    .filter((name) => {
      try {
        return readlinkSync(`/proc/${name}/exe`) === native;
      } catch {
        return false;
      }
    });

try {
  git("worktree", "add", "--detach", parent, baseline);
  assert.deepEqual(project(join(parent, registryPath)), receipt.fixtureAuthority);
  // Both production sources receive the same already-qualified timed driver.
  for (const { path, sha256 } of drivers) {
    copyFileSync(join(root, path), join(parent, path));
    assert.equal(digest(readFileSync(join(parent, path))), sha256);
  }
  const productionOverlay = git("-C", parent, "diff", "--name-only").split("\n").filter(Boolean);
  assert(productionOverlay.every((path) => driverPaths.includes(path)));
  receipt.baselineChangedPaths = productionOverlay;
  const binaries = {};
  for (const [arm, cwd] of [
    ["original", parent],
    ["bulk", root],
  ]) {
    const result = spawnSync(
      "cargo",
      [
        "test",
        "--locked",
        "--profile",
        "ci",
        "-p",
        "vize_canon",
        "--test",
        "tier_l_incremental",
        "--no-run",
        "--message-format=json",
      ],
      {
        cwd,
        env: { ...process.env, CARGO_TARGET_DIR: join(root, "target") },
        encoding: "utf8",
        maxBuffer: 128 * 1024 * 1024,
      },
    );
    writeFileSync(join(output, `${arm}-build.jsonl`), result.stdout ?? "");
    writeFileSync(join(output, `${arm}-build.stderr`), result.stderr ?? "");
    assert.equal(
      result.status,
      0,
      `${arm} source build failed: ${result.error ?? result.signal ?? result.status}`,
    );
    const artifacts = result.stdout
      .split("\n")
      .filter(Boolean)
      .map((line) => JSON.parse(line));
    const executable = artifacts.findLast(
      (item) =>
        item.reason === "compiler-artifact" &&
        item.target?.name === "tier_l_incremental" &&
        item.executable,
    )?.executable;
    assert(executable, `${arm} integration binary absent`);
    const pinned = join(temporary, `${arm}-tier-l-test`);
    copyFileSync(executable, pinned);
    binaries[arm] = {
      path: pinned,
      sha256: digest(readFileSync(pinned)),
      productionSource: arm === "original" ? baseline : source,
    };
  }
  receipt.binaries = binaries;
  persist();
  // Three authored pairs alternate execution order on this one runner.
  for (let pair = 0; pair < 3; pair++) {
    const order = pair % 2 === 0 ? ["original", "bulk"] : ["bulk", "original"];
    const packet = { pair, order, arms: {} };
    packets.push(packet);
    for (const arm of order) {
      const dir = join(output, `pair-${pair}`, arm);
      mkdirSync(dir, { recursive: true });
      const before = nativeProcesses();
      const result = spawnSync(
        binaries[arm].path,
        [
          "vben_batch_incremental_session_reuses_exact_materialized_delta",
          "--exact",
          "--ignored",
          "--nocapture",
        ],
        {
          cwd: arm === "original" ? parent : root,
          env: {
            ...process.env,
            VIZE_TIER_L_FIXTURE: fixture,
            VIZE_TIER_L_CORSA_BIN: native,
            VIZE_RUNTIME_NODE_MODULES: runtime,
            VIZE_TIER_L_METRICS_DIR: dir,
          },
          encoding: "utf8",
          maxBuffer: 64 * 1024 * 1024,
        },
      );
      writeFileSync(join(dir, "stdout.log"), result.stdout ?? "");
      writeFileSync(join(dir, "stderr.log"), result.stderr ?? "");
      const live = nativeProcesses().filter((pid) => !before.includes(pid));
      const metrics = existsSync(join(dir, "metrics.json"))
        ? JSON.parse(readFileSync(join(dir, "metrics.json"), "utf8"))
        : null;
      packet.arms[arm] = {
        status: result.status,
        signal: result.signal,
        error: result.error?.message,
        metrics,
        nativeProcessesBefore: before,
        newNativeProcessesAfterExit: live,
      };
      persist();
      assert.equal(
        result.status,
        0,
        `pair ${pair}/${arm} original budget or diagnostic law failed`,
      );
      assert.match(result.stdout, /test result: ok\. 1 passed; 0 failed;/);
      assert.deepEqual(live, [], `pair ${pair}/${arm} retained a native process after exit`);
      assert.equal(metrics.fileCount, 681);
      assert.deepEqual(metrics.fixture, {
        id: "vue-vben-admin",
        revision: receipt.fixtureRevision,
        injectedFile: "apps/web-antd/src/__vize_batch_incremental_oracle__.vue",
      });
    }
    assert.deepEqual(packet.arms.original.metrics.budget, packet.arms.bulk.metrics.budget);
    assert.equal(packet.arms.original.metrics.budgetScale, packet.arms.bulk.metrics.budgetScale);
    assert.deepEqual(
      packet.arms.original.metrics.lanes.map((lane) => lane.metrics),
      packet.arms.bulk.metrics.lanes.map((lane) => lane.metrics),
    );
  }
  const median = (values) => [...values].sort((a, b) => a - b)[1];
  receipt.measurements = ["cold", "brokenWarm", "repairedWarm"].map((lane) => {
    const durations = (arm) =>
      packets.map(
        (packet) => packet.arms[arm].metrics.lanes.find((row) => row.name === lane).durationMs,
      );
    const original = durations("original");
    const bulk = durations("bulk");
    return {
      lane,
      originalMs: original,
      bulkMs: bulk,
      originalMedianMs: median(original),
      bulkMedianMs: median(bulk),
      ratio: median(bulk) / median(original),
    };
  });
  receipt.warmGainObserved = receipt.measurements
    .filter((row) => row.lane !== "cold")
    .every((row) => row.bulkMedianMs < row.originalMedianMs);
  receipt.scope =
    "Matched original500 production timings only; whole681 diagnostic custody is the separate strict law. No 10x/default claim.";
  persist();
  console.log(
    JSON.stringify({
      source,
      baseline,
      measurements: receipt.measurements,
      warmGainObserved: receipt.warmGainObserved,
    }),
  );
} catch (error) {
  receipt.failure = { message: error.message, stack: error.stack };
  persist();
  throw error;
} finally {
  persist();
  if (existsSync(parent)) {
    try {
      git("worktree", "remove", "--force", parent);
    } catch (error) {
      receipt.cleanupError = { message: error.message, stack: error.stack };
      persist();
      if (!receipt.failure) throw error;
    }
  }
}
