import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const canonical = path.join(root, "crates/vize_l1/src/markup/lex");
const packaged = path.join(root, "crates/vize_armature/src/tokenizer");

function files(directory: string, prefix = ""): string[] {
  return fs.readdirSync(directory, { withFileTypes: true }).flatMap((entry) => {
    const relative = path.join(prefix, entry.name);
    if (entry.isDirectory()) return files(path.join(directory, entry.name), relative);
    assert.ok(entry.isFile(), `unexpected snapshot entry: ${relative}`);
    return [relative];
  });
}

test("packaged tokenizer matches L1 canonical source byte for byte", () => {
  const sourceFiles = files(path.join(canonical, "compat")).sort();
  const snapshotFiles = files(packaged)
    .filter((file) => file !== "mod.rs")
    .sort();
  assert.deepEqual(snapshotFiles, sourceFiles, "tokenizer snapshot file set drifted");
  assert.deepEqual(
    fs.readFileSync(path.join(packaged, "mod.rs")),
    fs.readFileSync(path.join(canonical, "compat.rs")),
    "tokenizer root drifted",
  );
  for (const relative of sourceFiles) {
    assert.deepEqual(
      fs.readFileSync(path.join(packaged, relative)),
      fs.readFileSync(path.join(canonical, "compat", relative)),
      `tokenizer snapshot drifted: ${relative}`,
    );
  }
});
