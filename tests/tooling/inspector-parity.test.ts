import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { inspectorReportDirectory } from "../_helpers/inspector-parity.ts";

test("inspector reports use the shared artifact directory by default", () => {
  const outputDirectory = inspectorReportDirectory("Example / Application");

  const expectedSuffix = path.join(".vize", "artifacts", "inspect", "Example-Application");
  assert.ok(
    outputDirectory.endsWith(expectedSuffix),
    `Expected path to end with "${expectedSuffix}", but got "${outputDirectory}"`,
  );
});

test("inspector reports preserve an explicit output directory", () => {
  assert.equal(
    inspectorReportDirectory("Example", "/tmp/inspection-output"),
    "/tmp/inspection-output",
  );
});
