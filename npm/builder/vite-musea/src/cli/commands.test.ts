import assert from "node:assert/strict";
import path from "node:path";
import test from "node:test";

import type { A11yOptions } from "../types/index.ts";
import { MuseaA11yRunner } from "../a11y/index.ts";
import {
  hasCiBlockingVrtResult,
  isArtFileInput,
  createA11yRunner,
  createVrtOptions,
} from "./commands.ts";
import { parseArgs } from "./index.ts";
import type { VrtSummary } from "../vrt.ts";

function summary(overrides: Partial<VrtSummary>): VrtSummary {
  return {
    total: 1,
    passed: 1,
    failed: 0,
    new: 0,
    skipped: 0,
    errors: 0,
    duration: 25,
    ...overrides,
  };
}

void test("VRT CI blocks on visual diffs", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ failed: 1, passed: 0 })), true);
});

void test("VRT CI allows skipped variants", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ skipped: 1, passed: 0 })), false);
});

void test("VRT CI blocks on runner errors", () => {
  assert.equal(hasCiBlockingVrtResult(summary({ errors: 1, passed: 0 })), true);
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

void test("CLI workers accepts positive concurrency and clamps invalid values", () => {
  const concurrent = parseArgs(["--workers", "8"]);
  assert.equal(concurrent.workers, 8);
  assert.equal(concurrent.workersProvided, true);

  const invalid = parseArgs(["-w", "0"]);
  assert.equal(invalid.workers, 1);
  assert.equal(invalid.workersProvided, true);
});

void test("CLI VRT options preserve config threshold when CLI threshold is omitted", () => {
  const options = parseArgs([]);
  options.vrt = {
    threshold: 0,
    capture: { waitForPreviewReady: true, settleTime: 250 },
    comparison: { antiAliasing: false },
    viewports: [{ width: 320, height: 240, name: "tiny" }],
    workers: 4,
  };

  assert.deepEqual(createVrtOptions(options), {
    snapshotDir: ".vize/snapshots",
    threshold: 0,
    capture: { waitForPreviewReady: true, settleTime: 250 },
    comparison: { antiAliasing: false },
    viewports: [{ width: 320, height: 240, name: "tiny" }],
    workers: 4,
  });
});

void test("CLI threshold overrides configured VRT threshold", () => {
  const options = parseArgs(["--threshold", "0", "--workers", "8"]);
  options.vrt = {
    threshold: 10,
    workers: 2,
    comparison: { antiAliasing: false },
  };

  assert.deepEqual(createVrtOptions(options), {
    snapshotDir: ".vize/snapshots",
    threshold: 0,
    workers: 8,
    capture: { waitForPreviewReady: true },
    comparison: { antiAliasing: false },
  });
});

void test("CLI uses vrt.snapshotDir resolved against the config file directory", () => {
  const options = parseArgs(["-c", path.join("project", "vite.config.ts"), "-o", "reports"]);
  options.vrt = {
    snapshotDir: path.join("vrt", "baseline"),
    threshold: 0,
    viewports: [{ width: 320, height: 200, name: "small" }],
    capture: { waitForPreviewReady: true, settleTime: 250 },
    comparison: { antiAliasing: false },
    workers: 2,
  };

  // run, approve, and clean all go through createVrtOptions.
  assert.deepEqual(createVrtOptions(options), {
    snapshotDir: path.resolve("project", "vrt", "baseline"),
    threshold: 0,
    viewports: [{ width: 320, height: 200, name: "small" }],
    capture: { waitForPreviewReady: true, settleTime: 250 },
    comparison: { antiAliasing: false },
    workers: 2,
  });
});

void test("CLI keeps an absolute vrt.snapshotDir and falls back to output snapshots", () => {
  const configured = parseArgs(["-c", path.join("/tmp", "app", "vite.config.ts"), "-o", "reports"]);
  configured.vrt = { snapshotDir: path.join("/var", "baselines") };
  assert.equal(
    createVrtOptions(configured).snapshotDir,
    path.resolve(path.join("/var", "baselines")),
  );

  const fallback = parseArgs(["-o", "reports"]);
  fallback.vrt = { threshold: 3 };
  assert.equal(createVrtOptions(fallback).snapshotDir, path.join("reports", "snapshots"));
});

void test("vrt.a11y excludeRules is the object passed to MuseaA11yRunner", () => {
  const a11y: A11yOptions = {
    excludeRules: ["color-contrast"],
    includeRules: ["image-alt"],
    level: "AA",
  };
  const options = parseArgs(["--a11y"]);
  options.vrt = { a11y, threshold: 4 };

  const received: Array<A11yOptions | undefined> = [];
  class RecordingA11yRunner extends MuseaA11yRunner {
    constructor(configured?: A11yOptions) {
      super(configured);
      received.push(configured);
    }
  }

  const runner = createA11yRunner(options, RecordingA11yRunner);

  assert.equal(runner instanceof RecordingA11yRunner, true);
  assert.deepEqual(received, [a11y]);
  assert.equal(received[0], a11y);
  assert.deepEqual(received[0]?.excludeRules, ["color-contrast"]);
  assert.equal("a11y" in createVrtOptions(options), false);
  assert.equal(createVrtOptions(options).threshold, 4);
});
