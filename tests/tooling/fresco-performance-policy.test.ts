import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { test } from "node:test";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..", "..");

function readText(relativePath: string): string {
  return fs.readFileSync(path.join(repoRoot, relativePath), "utf8");
}

function assertMentions(text: string, fragments: readonly string[]) {
  for (const fragment of fragments) {
    assert.ok(text.includes(fragment), `expected policy to mention ${fragment}`);
  }
}

test("Fresco performance policy cites the live measurement surfaces", () => {
  const policy = readText("docs/fresco/performance-policy.md");
  const frescoPackage = JSON.parse(readText("npm/fresco/package.json")) as {
    files?: string[];
    scripts?: Record<string, string>;
  };

  assertMentions(policy, [
    "#3113",
    "cargo bench -p vize_fresco --bench render",
    "cargo bench -p vize_fresco --bench capabilities",
    "vp run --filter './npm/fresco' build",
    "npm pack --json",
    "criterion-ab",
    "build-js-packages",
    "test-js-packages",
  ]);
  assert.equal(frescoPackage.scripts?.build, "vp pack");
  assert.deepEqual(frescoPackage.files, ["dist"]);
  assert.ok(fs.existsSync(path.join(repoRoot, "crates/vize_fresco/benches/render.rs")));
  assert.ok(fs.existsSync(path.join(repoRoot, "crates/vize_fresco/benches/capabilities.rs")));
});

test("Fresco compatibility matrix links the performance policy", () => {
  const matrix = readText("docs/fresco/compatibility.md");
  assert.ok(matrix.includes("performance-policy.md"));
});
