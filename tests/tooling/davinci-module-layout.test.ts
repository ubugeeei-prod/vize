import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const repoRoot = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const davinciRoots = [
  "crates/vize_davinci",
  "crates/vize_disegno",
  "crates/vize_ricalco",
  "crates/vize_sinopia",
  "benchmarks/davinci_harness",
];
const pathAttribute = /^\s*#\s*\[\s*path\s*=/mu;

function rustFiles(root: string): string[] {
  const files: string[] = [];
  for (const entry of fs.readdirSync(root, { withFileTypes: true })) {
    const absolute = path.join(root, entry.name);
    if (entry.isDirectory()) {
      files.push(...rustFiles(absolute));
    } else if (entry.isFile() && entry.name.endsWith(".rs")) {
      files.push(absolute);
    }
  }
  return files;
}

test("Davinci-owned Rust modules use ordinary module discovery", () => {
  const violations = davinciRoots.flatMap((root) =>
    rustFiles(path.join(repoRoot, root))
      .filter((file) => pathAttribute.test(fs.readFileSync(file, "utf8")))
      .map((file) => path.relative(repoRoot, file)),
  );
  assert.deepEqual(
    violations,
    [],
    `replace path-attributed modules with ordinary mod declarations:\n${violations.join("\n")}`,
  );
});

test("the Davinci module-layout gate recognizes a path attribute", () => {
  assert.equal(pathAttribute.test('  #[path = "nested/file.rs"]\nmod file;'), true);
});
