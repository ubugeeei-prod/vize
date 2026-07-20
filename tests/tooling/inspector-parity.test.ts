import assert from "node:assert/strict";
import path from "node:path";
import { test } from "node:test";

import { inspectorReportDirectory } from "../_helpers/inspector-parity.ts";

test("inspector reports use the shared artifact directory by default", () => {
  const outputDirectory = inspectorReportDirectory("Example / Application");

  assert.match(
    outputDirectory,
    new RegExp(
      `${escapeRegExp(path.join(".vize", "artifacts", "inspect", "Example-Application"))}$`,
    ),
  );
});

test("inspector reports preserve an explicit output directory", () => {
  assert.equal(
    inspectorReportDirectory("Example", "/tmp/inspection-output"),
    "/tmp/inspection-output",
  );
});

function escapeRegExp(value: string): string {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}
