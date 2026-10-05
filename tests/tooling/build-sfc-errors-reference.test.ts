import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";

const root = new URL("../_fixtures/differential/compiler/build-sfc-errors/", import.meta.url);
const pack = JSON.parse(fs.readFileSync(new URL("case.json", root), "utf8"));
const ui = createRequire(new URL("../../npm/ui/package.json", import.meta.url));
const vue = createRequire(ui.resolve("vue/package.json"));
const compiler = vue("@vue/compiler-sfc");
const native = createRequire(import.meta.url)("../../npm/native/index.js");
const errors = (values: any[]) =>
  values.map((error) => ({
    name: error.name,
    message: error.message,
    ...Object.fromEntries(Object.entries(error).filter(([key]) => key !== "stack")),
  }));

test("the seven complete SFCs retain independent pinned official error vectors", () => {
  assert.equal(pack.issue, 7879);
  assert.equal(compiler.version, "3.5.35");
  assert.deepEqual(
    pack.fixtures.map((fixture: any) => fixture.name),
    [
      "WithScript",
      "TemplateOnly",
      "VIf",
      "Options",
      "Memo",
      "ValidWithScript",
      "ValidTemplateOnly",
    ],
  );
  for (const fixture of pack.fixtures) {
    const source = fs.readFileSync(new URL(`${fixture.name}.vue.txt`, root), "utf8");
    assert.equal(fixture.compiler, compiler.version);
    assert.equal(createHash("sha256").update(source).digest("hex"), fixture.sha256);
    const parsed = compiler.parse(source, { filename: "Fixture.vue" });
    assert.deepEqual(errors(parsed.errors), fixture.parseErrors, fixture.name);
    const bindings =
      parsed.descriptor.script || parsed.descriptor.scriptSetup
        ? compiler.compileScript(parsed.descriptor, { id: "fixture" }).bindings
        : {};
    assert.equal(parsed.descriptor.template.content, fixture.template);
    assert.deepEqual(JSON.parse(JSON.stringify(bindings)), fixture.bindings);
    const result = compiler.compileTemplate({
      source: fixture.template,
      filename: "Fixture.vue",
      id: "fixture",
      compilerOptions: { mode: "module", prefixIdentifiers: true, bindingMetadata: bindings },
    });
    assert.deepEqual(errors(result.errors), fixture.errors, fixture.name);
  }
});

test("actual native SFC error vectors survive script and template-only API routes", () => {
  for (const fixture of pack.fixtures) {
    const source = fs.readFileSync(new URL(`${fixture.name}.vue.txt`, root), "utf8");
    const vapor = fixture.name === "Memo";
    const actual = native.compileSfc(source, { filename: "Fixture.vue", vapor });
    const templateOnly = native.compileSfc(`<template>${fixture.template}</template>`, {
      filename: "Fixture.vue",
      vapor,
    });
    assert.deepEqual(actual.errors, templateOnly.errors, fixture.name);
    const refused = fixture.errors.length > 0 || vapor;
    assert.equal(actual.errors.length, refused ? 1 : 0, fixture.name);
    if (vapor) {
      // DOM accepts this source; Vize's current Vapor backend explicitly refuses it.
      assert.deepEqual(native.compileSfc(source, { filename: "Fixture.vue" }).errors, []);
      assert.deepEqual(fixture.errors, []);
    }
  }
});
