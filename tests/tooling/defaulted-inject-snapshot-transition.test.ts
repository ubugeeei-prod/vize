import assert from "node:assert/strict";
import fs from "node:fs";
import { test } from "node:test";

test("the optional-inject snapshot changes only the explicitly retired fallback warning", () => {
  const original = fs.readFileSync(
    new URL(
      "../../tools/support/compat/defaulted-inject/historical-partial.snap.txt",
      import.meta.url,
    ),
    "utf8",
  );
  const actual = fs.readFileSync(
    new URL(
      "../../crates/vize_croquis_cf/src/analyzer/tests_snapshots/snapshots/vize_croquis_cf__analyzer__tests_snapshots__provide_inject__snapshot_partial_shared_child_branches.snap",
      import.meta.url,
    ),
    "utf8",
  );
  const retired = original
    .split("\n")
    .filter((line) => line.indexOf("Warning: **Conditionally Defaulted Inject**:") === 0);
  assert.equal(retired.length, 1);
  assert.deepEqual(actual, original.replace(retired[0] + "\n", ""));
});
