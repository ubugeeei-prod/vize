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
import { buildMatchedBinaries } from "./typechecker-bulk-build-custody.mjs";
import { prepareSourceCustody } from "./typechecker-bulk-source-custody.mjs";

// The actual parent-only main retains the original route and all incoming fixes.
const baseline = "e8be174060515b5f2a97087f5dc8004dcbcae54e";
const root = process.cwd();
const git = (...args) => execFileSync("git", args, { cwd: root, encoding: "utf8" }).trim();
const source = git("rev-parse", "HEAD");
const digest = (bytes) => createHash("sha256").update(bytes).digest("hex");
const compareText = (left, right) => (left < right ? -1 : left > right ? 1 : 0);
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
  "crates/vize_canon/tests/support/tier_l_fixture.rs",
  "crates/vize_canon/tests/support/tier_l_incremental_artifact.rs",
  "crates/vize_canon/tests/support/tier_l_incremental_budget.rs",
  "crates/vize_canon/tests/support/tier_l_incremental_failure.rs",
  "crates/vize_canon/tests/support/tier_l_incremental_failure_tests.rs",
];
assert.deepEqual(
  readdirSync(join(root, "crates/vize_canon/tests/support"))
    .filter((name) => name.startsWith("tier_l_") && name.endsWith(".rs"))
    .sort(compareText),
  driverPaths
    .slice(1)
    .map((path) => path.split("/").at(-1))
    .sort(compareText),
);
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

let primaryFailure;
let cleanupFailure;
try {
  const custody = prepareSourceCustody({
    fixture,
    capture: resolve(root, process.env.VIZE_TIER_L_BULK_CAPTURE_DIR),
    output,
    driver: readFileSync(join(root, driverPaths[1]), "utf8"),
    authority: receipt.fixtureAuthority,
  });
  receipt.sourceCustody = {
    catalogPath: custody.catalogPath,
    catalogSha256: custody.catalogSha256,
    scope:
      "Complete original 500 Vue +181 TS catalog, Git-pinned bodies/configs, authored patches and restored absence; outside timers.",
  };
  persist();
  custody.verify("before-builds");
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
  const binaries = buildMatchedBinaries({
    root,
    parent,
    source,
    baseline,
    temporary,
    output,
    receipt,
    persist,
  });
  // Three authored pairs alternate execution order on this one runner.
  for (let pair = 0; pair < 3; pair++) {
    const order = pair % 2 === 0 ? ["original", "bulk"] : ["bulk", "original"];
    const packet = { pair, order, arms: {} };
    packets.push(packet);
    for (const arm of order) {
      const dir = join(output, `pair-${pair}`, arm);
      mkdirSync(dir, { recursive: true });
      const before = nativeProcesses();
      custody.verify("pair-" + pair + "/" + arm + "/before");
      const nativeShaBefore = digest(readFileSync(native));
      packet.arms[arm] = { nativeProcessesBefore: before, nativeShaBefore };
      persist();
      assert.deepEqual(before, [], "matched arm starts with a retained exact-native process");
      assert.equal(nativeShaBefore, receipt.nativeSha256, "native binary changed before arm");
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
        nativeShaBefore,
        nativeShaAfter: digest(readFileSync(native)),
        newNativeProcessesAfterExit: live,
      };
      persist();
      custody.verify("pair-" + pair + "/" + arm + "/after");
      assert.equal(
        packet.arms[arm].nativeShaAfter,
        receipt.nativeSha256,
        "native binary changed during arm",
      );
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
  receipt.cancellationScope =
    "Existing 30-minute job limit contains hangs; job cancellation cannot guarantee final PID census, cleanup or artifact upload. No execution-time fit claim.";
  receipt.scope =
    "Matched BatchTypeChecker integration timings only; whole681 diagnostic custody is the separate strict law. No CLI/LSP gain, 10x or default claim.";
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
  primaryFailure = error;
  persist();
} finally {
  persist();
  if (existsSync(parent)) {
    try {
      git("worktree", "remove", "--force", parent);
    } catch (error) {
      receipt.cleanupError = { message: error.message, stack: error.stack };
      persist();
      cleanupFailure = error;
    }
  }
}

if (primaryFailure) throw primaryFailure;
if (cleanupFailure) throw cleanupFailure;
