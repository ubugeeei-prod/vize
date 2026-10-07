import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const corpus = new URL("../_fixtures/differential/compiler/", import.meta.url);
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");
const read = (name) => readFileSync(new URL(name, corpus));
void test("the compiler corpus pins all original #7970 bytes and the real runtime target", () => {
  const registry = JSON.parse(read("manifest.json"));
  const registration = registry.runtimePacks.find(
    (row) => row.path === "component-slot-text-7970.manifest.json",
  );
  assert(registration);
  const manifestBytes = read(registration.path);
  assert.equal(hash(manifestBytes), registration.sha256);
  const manifest = JSON.parse(manifestBytes);
  assert.equal(manifest.cases.length, 1);
  const row = manifest.cases[0];
  for (const input of row.inputs.files)
    assert.equal(hash(read(`${row.inputs.root}/${input.path}`)), input.sha256, input.path);
  assert.equal(hash(read(row.custody.path)), row.custody.sha256);
  assert.equal(hash(read(row.reference.path)), row.reference.sha256);
  const root = `${row.inputs.root}/`;
  const body = read(`${root}original-issue.md`).toString();
  const blocks = [...body.matchAll(/```([^\n]*)\n([\s\S]*?)```/gu)];
  for (const [index, filename] of [
    [0, "cmp.mjs.txt"],
    [2, "MyTitle.vue.txt"],
    [3, "App.vue.txt"],
    [4, "observe.mjs.txt"],
    [5, "reported-dom.txt"],
  ])
    assert.equal(read(root + filename).toString(), blocks[index][2]);
  const cases = JSON.parse(read(root + "cases.json"));
  assert.equal(cases.length, 16);
  assert.deepEqual(cases.slice(0, 3), [
    {
      name: "reported-element",
      template: '<div>\n  <span class="icon"></span>\n  {{ title }}\n</div>',
    },
    {
      name: "reported-component",
      template: '<MyTitle>\n  <span class="icon"></span>\n  {{ title }}\n</MyTitle>',
    },
    { name: "reported-text-only", template: "<MyTitle>Hello {{ name }}!</MyTitle>" },
  ]);
  assert.equal(row.runtime.nativeHandled, 0);
});
