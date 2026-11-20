// Tooling selection follows what a test reads, not what shared data names (#6830).
import assert from "node:assert/strict";
import { matchesGlob } from "node:path";
import test from "node:test";
import { cacheInputs } from "../../tools/config/vite-plus/task-inputs.ts";
import { toolingTestScopes } from "../../tools/config/vite-plus/tooling-test-scopes.ts";
import {
  memberReferences,
  planToolingTests,
  SHARED_HELPER_IMPORTERS,
} from "../../tools/support/compat/github/plan-tooling-tests.mjs";

const source = [
  `const glob = "tools/**";`,
  `const top = "crates";`,
  `const nested = "tools/support/release";`,
  `const exact = "tools/support/release/manifest.ts";`,
].join("\n");
const refs = (member: string, importers = 1) =>
  (memberReferences(member, source, importers) as string[]).toSorted();

test("third-party sources never select their importers", () => {
  assert.deepEqual(refs("npm/cli/node_modules/typescript/lib/typescript.js"), []);
  assert.deepEqual(refs("tests/node_modules/yaml/dist/index.js", 50), []);
});

test("task configuration modules select only by exact file names", () => {
  assert.deepEqual(refs("tools/config/vite-plus/task-inputs.ts"), [
    "tools/support/release/manifest.ts",
  ]);
});

test("shared helpers drop globs and top-level names; narrow modules keep every reference", () => {
  assert.deepEqual(refs("tests/tooling/support/shared.ts", SHARED_HELPER_IMPORTERS + 1), [
    "tools/support/release",
    "tools/support/release/manifest.ts",
  ]);
  assert.deepEqual(refs("tests/tooling/support/narrow.ts", SHARED_HELPER_IMPORTERS), [
    "crates",
    "tools/**",
    "tools/support/release",
    "tools/support/release/manifest.ts",
  ]);
});

const scope = (test: string) =>
  toolingTestScopes.find((entry) => entry.tests.some((glob) => matchesGlob(test, glob)));

test("audited release contracts keep crate manifests but not Rust sources as inputs", () => {
  const contract = scope("tests/tooling/release/release-workflow.test.ts");
  assert.equal(contract?.input, "toolingReleaseSurfaces");
  const inputs = cacheInputs.toolingReleaseSurfaces;
  const matches = (path: string) => inputs.some((glob) => matchesGlob(path, glob));
  assert.equal(matches("crates/vize_canon/src/lib.rs"), false);
  assert.equal(matches("davinci/vize_l2/src/lib.rs"), false);
  assert.equal(matches("docs/content/guide/index.md"), false);
  for (const path of ["crates/vize/Cargo.toml", "Cargo.lock", ".github/workflows/release.yml"])
    assert.equal(matches(path), true, path);
  // Release tests that execute the source CLI or native addon keep product sources.
  assert.equal(scope("tests/tooling/release/computed-event-release.test.ts")?.input, "tooling");
});

test("nightly Full qualification (merge tier) still runs every release contract", () => {
  const merge = planToolingTests(["crates/vize_canon/src/lib.rs"], { tier: "merge" })
    .tests as string[];
  const all = planToolingTests([], { tier: "merge" }).tests as string[];
  assert.deepEqual(merge, all);
  assert.ok(merge.some((file) => file.startsWith("tests/tooling/release/")));
});
