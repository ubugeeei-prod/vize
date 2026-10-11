import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import { test } from "node:test";

const base = new URL("../_fixtures/differential/compiler/", import.meta.url);
const read = (name) => readFileSync(new URL(name, base));
const hash = (bytes) => createHash("sha256").update(bytes).digest("hex");

void test("inherited SVG bindings retain a separate complete corpus and no release credit", () => {
  const name = "vapor-svg-namespace-8624.manifest.json";
  const registration = JSON.parse(read("manifest.json")).runtimePacks.find(
    (row) => row.path === name,
  );
  assert.ok(registration);
  const bytes = read(name);
  assert.equal(hash(bytes), registration.sha256);
  const pack = JSON.parse(bytes);
  assert.equal(pack.cases.length, 1);
  const entry = pack.cases[0];
  for (const input of entry.inputs.files)
    assert.equal(hash(read(`${entry.inputs.root}/${input.path}`)), input.sha256, input.path);
  assert.equal(hash(read(entry.reference.path)), entry.reference.sha256);
  const controls = JSON.parse(read(entry.reference.path));
  assert.equal(controls.length, 1, "separate authored namespace control");
  assert.equal(controls[0].expectedTrees.length, controls[0].steps.length + 2);
  assert.deepEqual(controls[0].expectedTrees.at(-1), []);
  assert.deepEqual(controls[0].expectedNamespacePhases.at(-1), []);
  assert.equal(entry.runtime.sourceProvider, "fresh-source-napi");
  assert.equal(entry.runtime.oracleProvider, "independent-stock-vdom");
  assert.equal(entry.runtime.sourceBrowser, "physical Chromium");
  for (const target of [entry.runtime.browserTarget, entry.runtime.helper])
    assert.ok(readFileSync(new URL(`../../${target}`, import.meta.url)).length > 0);
  assert.equal(entry.runtime.nativeHandled, 0);
  assert.equal(entry.runtime.lockedVue, "3.6.0-rc.9");
  assert.equal(entry.runtime.reportedVueExecution, "unexecuted");
});
