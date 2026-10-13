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

void test("SVG title parser controls register the complete native and Armature corpus", () => {
  const name = "vapor-svg-title-8624.manifest.json";
  const rows = JSON.parse(read("manifest.json")).runtimePacks.filter((row) => row.path === name);
  assert.equal(rows.length, 1);
  const bytes = read(name);
  assert.equal(hash(bytes), rows[0].sha256);
  const pack = JSON.parse(bytes);
  assert.equal(pack.cases.length, 1);
  const entry = pack.cases[0];
  assert.equal(entry.id, "compiler/parser/vapor-svg-title-8624");
  assert.equal(entry.inputs.files.length, 1);
  const corpus = read(entry.inputs.files[0].path);
  assert.equal(hash(corpus), entry.inputs.files[0].sha256);
  assert.equal(entry.reference.path, entry.inputs.files[0].path);
  assert.equal(hash(corpus), entry.reference.sha256);
  const controls = corpus
    .toString()
    .trimEnd()
    .split("\n")
    .map((row) => row.split("\t"));
  assert.equal(controls.length, 12);
  assert.equal(new Set(controls.map(([name]) => name)).size, 12);
  for (const fields of controls) assert.equal(fields.length, 3);
  assert.deepEqual(entry.runtime.rustTargets, [
    "davinci/vize_l1/tests/native_svg_title.rs",
    "crates/vize_armature/src/parser/svg_title_tests.rs",
  ]);
  assert.equal(entry.runtime.phase, "parser-source-controls");
  assert.equal(entry.runtime.caseCount, controls.length);
  assert.equal(entry.runtime.nativeHandled, 0);
  assert.equal(entry.runtime.publicationQualification, false);
});

void test("SVG title scope controls keep pinned Vue 3.6 authority separate from 3.5 diagnostics", () => {
  const name = "vapor-svg-title-scope-8624.manifest.json";
  const rows = JSON.parse(read("manifest.json")).runtimePacks.filter((row) => row.path === name);
  assert.equal(rows.length, 1);
  const bytes = read(name);
  assert.equal(hash(bytes), rows[0].sha256);
  const pack = JSON.parse(bytes);
  assert.equal(pack.cases.length, 1);
  const entry = pack.cases[0];
  for (const input of entry.inputs.files) assert.equal(hash(read(input.path)), input.sha256);
  const controls = read(entry.reference.path)
    .toString()
    .trimEnd()
    .split("\n")
    .map((row) => row.split("\t"));
  assert.equal(hash(read(entry.reference.path)), entry.reference.sha256);
  assert.equal(controls.length, 11);
  assert.equal(new Set(controls.map(([name]) => name)).size, 11);
  for (const fields of controls) assert.equal(fields.length, 3);
  const references = [
    ["rc9", "3.6.0-rc.9", "pinned-vapor-parser-reference"],
    ["rc10", "3.6.0-rc.10", "pinned-vapor-parser-reference"],
    ["3.5.35-diagnostic", "3.5.35", "initial-diagnostic-only"],
  ].map(([label, version, role]) => {
    const [header, ...packets] = read(`vapor-svg-title-scope-8624/vue-${label}.json.txt`)
      .toString()
      .trimEnd()
      .split("\n")
      .map((line) => JSON.parse(line));
    assert.equal(header.version, version);
    assert.equal(header.role, role);
    assert.equal(header.archiveAuthenticated, false);
    assert.equal(header.caseCount, 11);
    assert.deepEqual(
      header.packages.map(({ name }) => name),
      ["@vue/compiler-dom", "@vue/compiler-core", "@vue/shared"],
    );
    for (const pkg of header.packages) {
      assert.equal(pkg.version, version);
      assert.match(pkg.manifestSha256, /^[a-f0-9]{64}$/);
      assert.ok(pkg.loadedFiles.some(({ path }) => path.startsWith("dist/")));
      for (const file of pkg.loadedFiles) assert.match(file.sha256, /^[a-f0-9]{64}$/);
    }
    assert.equal(packets.length, 11);
    for (const packet of packets) {
      assert.ok(Array.isArray(packet.parse.children));
      assert.ok(Array.isArray(packet.compile.ast.children));
      assert.ok(packet.compile.code.length > 0);
      assert.ok(Array.isArray(packet.warnings));
    }
    return packets.map(({ name, source, shape }) => [name, source, shape.join("|")]);
  });
  assert.deepEqual(references[0], controls);
  assert.deepEqual(references[1], controls);
  assert.notDeepEqual(
    references[2],
    controls,
    "the initial 3.5 comparison cannot qualify a 3.6 regression",
  );
  assert.equal(entry.runtime.caseCount, 11);
  assert.equal(entry.runtime.nativeHandled, 0);
  assert.equal(entry.runtime.productionChange, false);
  assert.equal(entry.runtime.publicationQualification, false);
  assert.equal(
    hash(read("vapor-svg-title-8624.tsv")),
    "34b19ba8c4363dc8d2989ce59c7acf39d89ab8375c4e54ed84d04b56de6a0162",
  );
});
