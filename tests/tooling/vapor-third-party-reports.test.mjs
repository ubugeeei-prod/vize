import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const base = new URL("../_fixtures/differential/compiler/", import.meta.url);
const read = (path) => readFileSync(new URL(path, base));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

void test("third-party Vapor runtime packs retain the original report and every fixture byte", () => {
  const registry = JSON.parse(read("manifest.json"));
  const packs = registry.runtimePacks.filter((row) => row.path.startsWith("vapor-hebilicious-"));
  assert(packs.length > 0);
  for (const row of packs) {
    const manifest = read(row.path);
    assert.equal(hash(manifest), row.sha256);
    const pack = JSON.parse(manifest).cases[0];
    for (const file of pack.inputs.files)
      assert.equal(hash(read(`${pack.inputs.root}/${file.path}`)), file.sha256, file.path);
    const original = read(`${pack.inputs.root}/original-issue.md`).toString();
    const reported = [...original.matchAll(/```vue\n([\s\S]*?)```/gu)][0][1];
    assert.equal(read(`${pack.inputs.root}/reported.vue.txt`).toString(), reported);
    if (pack.inputs.root === "vapor-slot-destructure-8626") {
      const owner = reported
        .split("<!-- SlotConsumer.vue -->\n")[0]
        .replace("<!-- SlotOwner.vue -->\n", "");
      assert.equal(JSON.parse(read(`${pack.inputs.root}/Child.vue.json`)), owner);
    }
    assert.equal(pack.runtime.nativeHandled, 0);
    assert.equal(pack.runtime.lockedVue, "3.6.0-rc.9");
  }
});
