import assert from "node:assert/strict";
import fs from "node:fs";
import test from "node:test";

test("scheduler carrier follows each direct source and allocator dependency", () => {
  const workflow = fs.readFileSync(
    new URL("../../.github/workflows/lsp-scheduler-cost-carrier.yml", import.meta.url),
    "utf8",
  );
  const paths = [...workflow.matchAll(/^ {6}- "([^"]+)"$/gm)].map((match) => match[1]);

  assert.deepEqual(paths, [
    ".github/workflows/lsp-scheduler-cost-carrier.yml",
    "Cargo.toml",
    "Cargo.lock",
    "crates/vize_maestro/Cargo.toml",
    "crates/vize_maestro/src/server.rs",
    "crates/vize_maestro/src/server/**",
    "davinci/vize_l0/Cargo.toml",
    "davinci/vize_l0/src/profiler.rs",
    "davinci/vize_l0/src/profiler/**",
    "tools/benchmarks/crates/davinci_harness/**",
    "tests/tooling/lsp-scheduler-cost-carrier.test.ts",
  ]);
});
