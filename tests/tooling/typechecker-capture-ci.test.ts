import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";
const root = fileURLToPath(new URL("../../", import.meta.url));

test("capture and CI selection retain unchanged diagnostic assertions and the full required tier", () => {
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
  const workflow = fs.readFileSync(path.join(root, ".github/workflows/pr-rust-checks.yml"), "utf8");
  assert(workflow.includes("Prepare required typechecker observations"));
  assert(workflow.includes("Verify actual typechecker fixture observations"));
  assert(workflow.includes("Require all registered typechecker observations"));
  const config = fs.readFileSync(path.join(root, ".config/nextest.toml"), "utf8");
  const full = config.slice(config.indexOf("[profile.full]"));
  assert(!full.includes("default-filter"));
  assert(full.includes("retries = 0"));
});
