import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import {
  type BurstSample,
  type SerialSample,
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
const serial = (outputSha = "s"): SerialSample => ({ latencies: [1, 2, 3, 4], outputSha });

const cell = (
  head: { bursts: BurstSample[]; serials: SerialSample[] },
  allowOutputChange = false,
) =>
  summarizeCell({
    label: "linux-ext4",
    sfcs: 1000,
    serialFiles: 4,
    builds: ["base", "head"],
    bursts: new Map([
      ["base", [burst(200), burst(220), burst(210)]],
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

test("identical answers publish medians and signed head deltas", () => {
  const result = cell({
    bursts: [burst(100), burst(110), burst(105)],
    serials: [serial(), serial()],
  });
  assert.equal(result.failure, undefined);
  assert.equal(result.outputsIdentical, true);
  assert.equal(result.builds[1]?.burstWallMs, 105);
  const report = markdown("linux-ext4", [result]);
  assert.match(report, /\| head \| 105 \(-50\.0%\) \(100–110\)/);
  assert.match(report, /\| base \| 210 \(200–220\)/);
});

test("a build that answers differently publishes no timing unless allowed", () => {
  const changed = {
    bursts: [burst(100, "b"), burst(110, "b"), burst(105, "b")],
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
    { bursts: [burst(100), burst(110, "other"), burst(105)], serials: [serial(), serial()] },
    true,
  );
  assert.equal(flaky.failure, "head answered runs differently");
});

test("the workflow measures every filesystem from one build per platform", () => {
  const workflow = readFileSync(
    new URL("../../.github/workflows/content-mapper-pair.yml", import.meta.url),
    "utf8",
  );
  for (const label of ["linux-ext4", "wsl2-ext4", "wsl2-windows-mount", "windows-ntfs"]) {
    assert.match(workflow, new RegExp(label));
  }
  assert.match(
    workflow,
    /WSLENV: SIZES\/u:RUNS\/u:ALLOW\/u:REFERENCE\/u:WSL_MOUNT_ROOT\/u:GITHUB_STEP_SUMMARY\/u/,
  );
  assert.match(workflow, /cargo build --profile ci-opt -p vize --bin vize/);
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
