import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import path from "node:path";
import { test } from "node:test";

test("the actual Rust reporter preserves rejected findings without hiding valid findings", () => {
  const result = spawnSync(
    "rust-script",
    ["--test", "tools/commands/fixtures/lint-divergence-report.rs"],
    {
      cwd: path.resolve(import.meta.dirname, "..", ".."),
      encoding: "utf8",
      timeout: 120_000,
      env: { ...process.env, LANG: "C", LC_ALL: "C" },
    },
  );
  assert.equal(result.status, 0, `${result.stdout}\n${result.stderr}`);
  assert.match(result.stdout, /baseline_zero_column_is_counted_as_invalid_range \.\.\. ok/u);
  assert.match(result.stdout, /baseline_invalid_range_does_not_hide_valid_findings \.\.\. ok/u);
});
