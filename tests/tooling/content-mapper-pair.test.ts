import { readWorkflowShell } from "./support/automation-source.ts";
import assert from "node:assert/strict";
import test from "node:test";

import {
  type BurstSample,
  type SerialSample,
  PAIR_ORDER_SEED,
  balancedOrders,
  frame,
  markdown,
  median,
  quantile,
  summarizeCell,
} from "../../tools/benchmarks/scripts/content-mapper-pair-report.ts";

const burst = (wallMs: number, outputSha = "a"): BurstSample => ({
  wallMs,
  cpuSeconds: wallMs / 100,
  peakMiB: 40,
  outputBytes: 1_000_000,
  outputSha,
});
const serial = (outputSha = "s", latencies = [1, 2, 3, 4]): SerialSample => ({
  latencies,
  outputSha,
});

const cell = (
  head: { bursts: BurstSample[]; serials: SerialSample[] },
  allowOutputChange = false,
) =>
  summarizeCell({
    label: "linux-ext4",
    sfcs: 1000,
    serialFiles: 4,
    replicates: 1,
    builds: ["base", "head"],
    bursts: new Map([
      ["base", [burst(200), burst(220)]],
      ["head", head.bursts],
    ]),
    serials: new Map([
      ["base", [serial(), serial()]],
      ["head", head.serials],
    ]),
    allowOutputChange,
  });

test("frames carry the exact UTF-8 body length", () => {
  const bytes = frame({ text: "雪" });
  const text = bytes.toString("utf8");
  const [header, body] = text.split("\r\n\r\n");
  assert.equal(header, `Content-Length: ${Buffer.byteLength(body ?? "")}`);
});

test("medians and quantiles ignore input order", () => {
  assert.equal(median([3, 1, 2]), 2);
  assert.equal(median([4, 1, 3, 2]), 2.5);
  assert.equal(quantile([5, 1, 4, 2, 3], 0.95), 5);
  assert.ok(Number.isNaN(median([])));
});

test("each seeded replicate measures both orders exactly once", () => {
  const builds = ["base", "head"];
  const observed = Array.from({ length: 5 }, (_, replicate) => {
    const [first, inverse] = balancedOrders(builds, replicate);
    assert.deepEqual(inverse, [...first].reverse());
    for (const build of builds) {
      assert.equal([first[0], inverse[0]].filter((candidate) => candidate === build).length, 1);
      assert.equal([first[1], inverse[1]].filter((candidate) => candidate === build).length, 1);
    }
    return first.join("");
  });
  assert.deepEqual(observed, ["basehead", "headbase", "headbase", "basehead", "basehead"]);
  assert.equal(PAIR_ORDER_SEED, 0x13018);
});

test("reference-enabled replicates balance every position and relative order", () => {
  const builds = ["base", "head", "reference"];
  for (let replicate = 0; replicate < 5; replicate++) {
    const orders = balancedOrders(builds, replicate);
    assert.equal(orders.length, 6);
    for (const build of builds) {
      assert.deepEqual(
        orders.map((order) => order.indexOf(build)).sort((left, right) => left - right),
        [0, 0, 1, 1, 2, 2],
      );
    }
    for (const left of builds) {
      for (const right of builds.filter((build) => build !== left)) {
        assert.equal(
          orders.filter((order) => order.indexOf(left) < order.indexOf(right)).length,
          orders.length / 2,
        );
      }
    }
  }
});

test("identical answers publish medians and signed head deltas", () => {
  const result = cell({
    bursts: [burst(100), burst(110)],
    serials: [serial("s", [1, 1, 1, 1]), serial("s", [9, 9, 9, 9])],
  });
  assert.equal(result.failure, undefined);
  assert.equal(result.outputsIdentical, true);
  assert.equal(result.builds[1]?.burstWallMs, 105);
  assert.equal(result.builds[1]?.serialRawSamples, 8);
  assert.equal(result.builds[1]?.serialSessions, 2);
  assert.equal(result.builds[1]?.serialP50Ms, 9);
  assert.equal(result.builds[1]?.serialP95Ms, 9);
  assert.equal(result.builds[1]?.serialSessionP50Ms, 5);
  assert.equal(result.builds[1]?.serialSessionP95Ms, 5);
  const report = markdown("linux-ext4", [result]);
  assert.match(report, /\| head \| 105 \(-50\.0%\) \(100–110\)/);
  assert.match(report, /\| base \| 210 \(200–220\)/);
  assert.match(report, /1 balanced replicates \(AB\+BA, seed 0x13018\)/);
  assert.match(report, /2 serial sessions\/build × 4 transforms = 8 raw latency samples\/build/);
});

test("a build that answers differently publishes no timing unless allowed", () => {
  const changed = {
    bursts: [burst(100, "b"), burst(110, "b")],
    serials: [serial("t"), serial("t")],
  };
  const refused = cell(changed);
  assert.equal(refused.failure, "builds answered differently");
  assert.doesNotMatch(markdown("x", [refused]), /\| head \|/);
  const allowed = cell(changed, true);
  assert.equal(allowed.failure, undefined);
  assert.equal(allowed.outputsIdentical, false);
});

test("runs of one build that disagree always fail the cell", () => {
  const flaky = cell(
    { bursts: [burst(100), burst(110, "other")], serials: [serial(), serial()] },
    true,
  );
  assert.equal(flaky.failure, "head answered runs differently");
});

test("the workflow measures every filesystem from one build per platform", () => {
  const workflow = readWorkflowShell("03-typecheck-compare");
  for (const label of ["linux-ext4", "wsl2-ext4", "wsl2-windows-mount", "windows-ntfs"]) {
    assert.match(workflow, new RegExp(label));
  }
  assert.match(
    workflow,
    /WSLENV: PROJECTS\/u:SIZES\/u:RUNS\/u:ALLOW\/u:REFERENCE\/u:WSL_MOUNT_ROOT\/u:GITHUB_STEP_SUMMARY\/u:VIZE_CI_FILES\/p$/mu,
  );
  assert.match(workflow, /cargo build --profile ci-opt -p vize --bin vize/);
  assert.match(workflow, /AB\+BA replicates/);
});

test("the syscall census reads strace lines and collapses generated spellings", async () => {
  const { parseStraceLine, pathShape } =
    await import("../../tools/benchmarks/scripts/content-mapper-census.ts");
  assert.deepEqual(parseStraceLine('openat(AT_FDCWD, "/p/src/a.ts", O_RDONLY|O_CLOEXEC) = 3'), {
    call: "openat",
    path: "/p/src/a.ts",
  });
  assert.deepEqual(parseStraceLine('[pid  42] readlink("/p/src", 0x7ff, 4095) = -1 EINVAL'), {
    call: "readlink",
    path: "/p/src",
  });
  assert.equal(parseStraceLine("+++ exited with 0 +++"), undefined);
  assert.equal(
    pathShape("/p/src/features/f0024/Widget09.vue", "/p"),
    "<project>/src/features/fNNNN/Name.vue",
  );
});

test("the giant prop workload retains every distinct script value and typed child use", async () => {
  const { giantComponentProps } =
    await import("../../tools/benchmarks/scripts/content-mapper-pair-giant.ts");
  const files = giantComponentProps(1000);
  const app = files.get("src/App.vue") ?? "";
  assert.equal(app.match(/const value\d+ = /g)?.length, 1000);
  assert.equal(app.match(/<Child :count="value\d+" \/>/g)?.length, 1000);
  assert.match(app, /const value999 = 999;/);
  assert.match(files.get("src/Child.vue") ?? "", /count: number/);
  assert.throws(() => giantComponentProps(0), /positive integer/);
  assert.throws(() => giantComponentProps(1.5), /positive integer/);
});

test("mapper giant cells use the existing projects input without replacing ordinary sizes", async () => {
  const { giantBlockSizes } =
    await import("../../tools/benchmarks/scripts/content-mapper-pair-giant.ts");
  assert.deepEqual(giantBlockSizes("vue:1000,monorepo:5000"), []);
  assert.deepEqual(giantBlockSizes("giant:1000,giant:10000"), [1000, 10000]);
  assert.throws(() => giantBlockSizes("giant:0"), /positive integer/);
  assert.throws(() => giantBlockSizes("giant:1.5"), /positive integer/);
});
