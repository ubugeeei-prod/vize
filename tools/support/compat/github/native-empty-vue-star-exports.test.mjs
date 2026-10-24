/** Receipt refusal controls only; no compiled or native acceptance credit. */
import assert from "node:assert/strict";
import { mkdtempSync, mkdirSync, rmSync, writeFileSync } from "node:fs";
import os from "node:os";
import { join } from "node:path";
import { test } from "node:test";
import { validateArtifact, validateRun } from "./native-empty-vue-star-exports.mjs";

function fixture(run) {
  const root = mkdtempSync(join(os.tmpdir(), "empty-star-receipt-refusal-"));
  const source = join(root, "crates/vize_canon/tests/empty_vue_star_exports/main.rs");
  mkdirSync(join(root, "crates/vize_canon/tests"), { recursive: true });
  writeFileSync(source, "// test-only receipt model; never compiled\n");
  const artifact = {
    target: { name: "empty_vue_star_exports", kind: ["test"], src_path: source },
    features: ["native"],
    profile: { test: true, opt_level: "3" },
  };
  try {
    run(root, artifact);
  } finally {
    rmSync(root, { recursive: true, force: true });
  }
}
void test("receipt admission refuses missing native feature, wrong target and wrong profile", () => {
  fixture((root, artifact) => {
    assert.throws(() => validateArtifact(root, { ...artifact, features: [] }));
    assert.throws(() =>
      validateArtifact(root, { ...artifact, target: { ...artifact.target, kind: ["bin"] } }),
    );
    assert.throws(() =>
      validateArtifact(root, { ...artifact, target: { ...artifact.target, name: "other" } }),
    );
    assert.throws(() =>
      validateArtifact(root, { ...artifact, profile: { test: false, opt_level: "3" } }),
    );
    assert.throws(() =>
      validateArtifact(root, { ...artifact, profile: { test: true, opt_level: "1" } }),
    );
  });
});
void test("an executable from a foreign source file cannot lend source identity", () => {
  fixture((root, artifact) => {
    const other = join(root, "crates/vize_canon/tests/foreign.rs");
    writeFileSync(other, "// distinct source\n");
    assert.throws(() =>
      validateArtifact(root, { ...artifact, target: { ...artifact.target, src_path: other } }),
    );
  });
});
void test("failed, killed, empty or filtered test output cannot supply actual three-law credit", () => {
  fixture((root, artifact) => {
    const result = { error: undefined, signal: null, status: 0, stdout: Buffer.alloc(0) };
    assert.throws(() => validateRun(root, root, artifact, { ...result, status: 1 }));
    assert.throws(() => validateRun(root, root, artifact, { ...result, signal: "SIGTERM" }));
    assert.throws(() => validateRun(root, root, artifact, { ...result, error: Error("timeout") }));
    assert.throws(() => validateRun(root, root, artifact, result));
    assert.throws(() =>
      validateRun(root, root, artifact, {
        ...result,
        stdout: Buffer.from(
          "test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out\n",
        ),
      }),
    );
  });
});
