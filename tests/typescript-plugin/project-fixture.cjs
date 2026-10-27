"use strict";

const assert = require("node:assert/strict");
const { createHash } = require("node:crypto");
const fs = require("node:fs");
const path = require("node:path");

const id = "typechecker/language-project/inherited-configuration-10740";
const directory = "typescript-project-configuration-10740";
function readProjectFixture() {
  const root = path.resolve(__dirname, "../_fixtures/differential/typechecker");
  const manifest = JSON.parse(fs.readFileSync(path.join(root, "manifest.json"), "utf8"));
  const row = manifest.languageProjectCases?.find((value) => value.id === id);
  assert(row?.state === "active", "the real project fixture must be registered");
  assert.equal(row.adapter, "typescript-project-configuration-v1");
  const files = new Map();
  for (const input of row.inputs) {
    assert(input.path.startsWith(`${directory}/`) && !input.path.split("/").includes(".."));
    const bytes = fs.readFileSync(path.join(root, input.path));
    assert.equal(createHash("sha256").update(bytes).digest("hex"), input.sha256);
    files.set(input.path.slice(directory.length + 1), bytes.toString("utf8"));
  }
  const pack = JSON.parse(files.get("case.json"));
  assert.equal(pack.schema, "vize.typescriptProjectConfigurationFixture.v1");
  assert.equal(pack.issue, 10740);
  assert.equal(pack.contract, "whole-tsserver-diagnostics-and-project-metadata-v1");
  for (const input of [...pack.inputs, pack.beforeWitness]) {
    const source = files.get(input.path);
    assert.equal(typeof source, "string", `missing pinned fixture ${input.path}`);
    assert.equal(createHash("sha256").update(source).digest("hex"), input.sha256);
  }
  const witness = JSON.parse(files.get("before-witness.json"));
  const originalInputs = JSON.parse(files.get("before-inputs.json"));
  const expected = JSON.parse(files.get("expected.json"));
  assert.deepEqual(witness.before.body, expected.before);
  assert.deepEqual(witness.after.body, [], "preserve the original warm false-clean failure");
  assert.deepEqual(witness.fresh.body, expected.strict);
  assert.deepEqual(witness.controlAfter.body, expected.controlAfter);
  assert.equal(files.get("src/Inherited.vue.txt"), originalInputs.source);
  assert.equal(files.get("src/control.ts.txt"), originalInputs.controlSource);
  assert.equal(files.get("tsconfig-6.0.json.txt"), originalInputs.root);
  assert.deepEqual(JSON.parse(files.get("inherited-before.json.txt")), originalInputs.before);
  assert.deepEqual(JSON.parse(files.get("inherited-after.json.txt")), originalInputs.after);
  return { files, pack, expected };
}

module.exports = { readProjectFixture };
