import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
const root = fileURLToPath(new URL("../../", import.meta.url));

test("capture never replaces the actual diagnostic assertion and the full tier stays unfiltered", () => {
  const rust = fs.readFileSync(
    path.join(root, "crates/vize_canon/tests/fix_history_diagnostics.rs"),
    "utf8",
  );
  const assertion = rust.indexOf("assert_eq!(\n            actual, case.diagnostics,");
  const capture = rust.indexOf("capture.record(");
  assert(
    assertion >= 0 && capture > assertion,
    "expected data must never substitute for actual diagnostics",
  );
  const workflow = fs.readFileSync(
    path.join(root, ".github/workflows/pr-rust-products.yml"),
    "utf8",
  );
  // Queue cohorts run the same assertions in place; only full qualification captures.
  assert(!workflow.includes("VIZE_TEST_FIX_HISTORY_CAPTURE_DIR"));
  const config = fs.readFileSync(path.join(root, ".config/nextest.toml"), "utf8");
  const full = config.slice(config.indexOf("[profile.full]"));
  assert(!full.includes("default-filter"));
  assert(full.includes("retries = 0"));
});
