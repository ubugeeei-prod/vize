import assert from "node:assert/strict";
import test from "node:test";

import { hasCiBlockingVrtResult, isArtFileInput, createVrtOptions } from "./commands.ts";
import { parseArgs } from "./index.ts";
import type { VrtSummary } from "../vrt.ts";

function summary(overrides: Partial<VrtSummary>): VrtSummary {
  return {
    total: 1,
    passed: 1,
    failed: 0,
    new: 0,
    skipped: 0,
    duration: 25,
    ...overrides,
  };
}

void test("VRT CI blocks on visual diffs", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ failed: 1, passed: 0 })), true);
});

void test("VRT CI blocks on capture errors", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ skipped: 1, passed: 0 })), true);
});

void test("VRT CI allows clean and newly-created baselines", () => {
  assert.equal(hasCiBlockingVrtResult(summary({})), false);
  assert.equal(hasCiBlockingVrtResult(summary({ passed: 0, new: 1 })), false);
});

void test("generate rejects existing art-file inputs", () => {
  assert.equal(isArtFileInput("src/components/Button.art.vue"), true);
  assert.equal(isArtFileInput("src/components/Button.vue"), false);
});

void test("CLI threshold accepts zero as an explicit threshold", () => {
  const options = parseArgs(["-t", "0"]);

  assert.equal(options.threshold, 0);
  assert.equal(options.thresholdProvided, true);
});

void test("CLI VRT options preserve config threshold when CLI threshold is omitted", () => {
  const options = parseArgs([]);
  options.vrt = {
    threshold: 0,
    capture: { settleTime: 250 },
    comparison: { antiAliasing: false },
    viewports: [{ width: 320, height: 240, name: "tiny" }],
  };

  assert.deepEqual(createVrtOptions(options), {
    snapshotDir: ".vize/snapshots",
    threshold: 0,
    capture: { settleTime: 250 },
    comparison: { antiAliasing: false },
    viewports: [{ width: 320, height: 240, name: "tiny" }],
  });
});

void test("CLI threshold overrides configured VRT threshold", () => {
  const options = parseArgs(["--threshold", "0"]);
  options.vrt = {
    threshold: 10,
    comparison: { antiAliasing: false },
  };

  assert.deepEqual(createVrtOptions(options), {
    snapshotDir: ".vize/snapshots",
    threshold: 0,
    comparison: { antiAliasing: false },
  });
});
