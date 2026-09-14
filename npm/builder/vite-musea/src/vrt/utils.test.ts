import assert from "node:assert/strict";
import test from "node:test";

import { buildSnapshotName, computeSummary } from "./utils.ts";

void test("snapshot names percent-encode path separators in variant names", () => {
  const snapshotName = buildSnapshotName("src/components/MyChip.art.vue", "Outlined / squared", {
    width: 1280,
    height: 720,
    name: "desktop",
  });

  assert.equal(snapshotName, "MyChip--Outlined%20%2F%20squared--desktop.png");
  assert.equal(snapshotName.includes("/"), false);
  assert.equal(snapshotName.includes("\\"), false);
});

void test("VRT summary reports runner errors separately from skipped variants", () => {
  const summary = computeSummary(
    [
      {
        artPath: "MyChip.art.vue",
        variantName: "Outlined / squared",
        viewport: { width: 1280, height: 720, name: "desktop" },
        passed: false,
        snapshotPath: ".vize/snapshots/MyChip--Outlined%20%2F%20squared--desktop.png",
        error: "ENOENT: no such file or directory",
      },
    ],
    Date.now(),
  );

  assert.equal(summary.errors, 1);
  assert.equal(summary.skipped, 0);
  assert.equal(summary.failed, 0);
});
