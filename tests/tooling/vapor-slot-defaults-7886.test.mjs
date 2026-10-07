import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const base = new URL("../_fixtures/differential/compiler/", import.meta.url);
const read = (path) => readFileSync(new URL(path, base));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

void test("#7886 preserves every original source byte and registers real runtime controls", () => {
  const registry = JSON.parse(read("manifest.json"));
  const row = registry.runtimePacks.find(
    (pack) => pack.path === "vapor-slot-defaults-7886.manifest.json",
  );
  assert(row);
  const manifest = read(row.path);
  assert.equal(hash(manifest), row.sha256);
  const pack = JSON.parse(manifest).cases[0];
  for (const file of pack.inputs.files)
    assert.equal(hash(read(`${pack.inputs.root}/${file.path}`)), file.sha256, file.path);
  const original = read(`${pack.inputs.root}/original-issue.md`).toString();
  const shell = [...original.matchAll(/```[^\n]*\n([\s\S]*?)```/gu)][0][1];
  assert.equal(read(`${pack.inputs.root}/repro.sh.txt`).toString(), shell);
  for (const name of ["App", "Child"]) {
    const source =
      shell.match(new RegExp(`cat > ${name}\\.vue <<'VUE'\\n([\\s\\S]*?)\\nVUE`))[1] + "\n";
    assert.equal(read(`${pack.inputs.root}/${name}.vue.txt`).toString(), source);
  }
  assert.equal(pack.runtime.nativeHandled, 0);
  assert.equal(JSON.parse(read(`${pack.inputs.root}/controls.json`)).length, 7);
});
