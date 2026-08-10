import assert from "node:assert/strict";
import { test } from "node:test";

import { compareCorpusFile } from "./support/glyph-corpus-sweep.ts";
import {
  assertNoCompilerErrors,
  blockSignature,
  normalizeCompilerMessages,
} from "./support/sfc-baseline-signatures.ts";
import { compareSfcWithDialectBaseline, compileLegacySide } from "./support/sfc-baselines.ts";
import type { SfcBaselineComparison } from "./support/sfc-baselines.ts";

function assertSemanticDiff(result: SfcBaselineComparison): void {
  assert.equal(result.verdict, "semantic-diff", result.differences.join("\n"));
  assert.equal(result.reasonCode, "semantic-signature-changed");
  assert.equal(result.failure, null);
}

const legacySource = [
  "<template functional>",
  '  <section v-bind.sync="displayProperties" v-on="$listeners">',
  '    <input @keyup.32="submit" />',
  '    <slot name="row" :value="value" />',
  "    <p>{{ value | first(1) | second }}</p>",
  "  </section>",
  "</template>",
  '<script lang="ts">',
  'import Vue from "vue";',
  'import Component from "vue-class-component";',
  "@Component export default class Example extends Vue { value = 1 }",
  "</script>",
  "",
].join("\n");

test("Vue 2 and 2.7 official compilers compare the same legacy SFC contract", () => {
  const formatted = legacySource.replace('v-on="$listeners"', '\n    v-on="$listeners"');
  const vue2 = compareSfcWithDialectBaseline(legacySource, formatted, "Legacy.vue", "2");
  const vue27 = compareSfcWithDialectBaseline(legacySource, formatted, "Legacy.vue", "2.7");

  assert.equal(vue2.verdict, "equivalent");
  assert.equal(vue2.baseline.package, "vue-template-compiler");
  assert.equal(vue2.baseline.version, "2.6.14");
  assert.equal(vue27.verdict, "equivalent");
  assert.equal(vue27.baseline.package, "@vue/compiler-sfc");
  assert.equal(vue27.baseline.version, "2.7.16");
  assert.equal(vue2.baseline.entrySha256?.length, 64);
  assert.equal(vue27.baseline.entrySha256?.length, 64);
});

test("legacy compiler signatures reject semantic mutations", () => {
  const mutations = [
    legacySource.replace('v-bind.sync="displayProperties"', 'v-bind="displayProperties"'),
    legacySource.replace("@keyup.32", "@keyup.113"),
    legacySource.replace('v-on="$listeners"', 'v-on="otherListeners"'),
    legacySource.replace('name="row"', 'name="other"'),
    legacySource.replace("first(1) | second", "second | first(2)"),
    legacySource.replace("<slot", '<div v-on="$listeners" /><slot'),
    legacySource.replace(
      "<p>{{ value | first(1) | second }}</p>",
      "<p> {{ value | first(1) | second }} </p>",
    ),
  ];
  for (const mutation of mutations) {
    assertSemanticDiff(compareSfcWithDialectBaseline(legacySource, mutation, "Legacy.vue", "2"));
  }
  assertSemanticDiff(
    compareSfcWithDialectBaseline(
      '<template><Widget :a="first()" :b="second()" /></template>\n',
      '<template><Widget :b="second()" :a="first()" /></template>\n',
      "Legacy.vue",
      "2",
    ),
  );
});

test("legacy baselines preserve native raw-text and v-pre bytes", () => {
  for (const tag of ["pre", "textarea", "listing"]) {
    const before = `<template><${tag}>  keep   me  </${tag}></template>\n`;
    const after = `<template><${tag}> keep me </${tag}></template>\n`;
    assertSemanticDiff(compareSfcWithDialectBaseline(before, after, "Raw.vue", "2"));
  }
  assertSemanticDiff(
    compareSfcWithDialectBaseline(
      "<template><div v-pre>{{  raw   text  }}</div></template>\n",
      "<template><div v-pre>{{ raw text }}</div></template>\n",
      "Raw.vue",
      "2",
    ),
  );
});

test("Options and Class API fixtures retain their SFC block contract", () => {
  const optionsSource = legacySource.replace(
    "@Component export default class Example extends Vue { value = 1 }",
    "export default Vue.extend({ data: () => ({ value: 1 }) })",
  );
  for (const source of [legacySource, optionsSource]) {
    assert.equal(
      compareSfcWithDialectBaseline(source, source, "Api.vue", "2").verdict,
      "equivalent",
    );
    assertSemanticDiff(
      compareSfcWithDialectBaseline(
        source,
        source.replace('<script lang="ts">', '<script lang="js">'),
        "Api.vue",
        "2",
      ),
    );
  }
});

test("baseline crashes retain pristine versus formatter ownership", () => {
  const malformed = "<template><div></template>\n";
  const originalFailure = compareSfcWithDialectBaseline(malformed, malformed, "Broken.vue", "2");
  const formattedFailure = compareSfcWithDialectBaseline(
    "<template><div /></template>\n",
    malformed,
    "Broken.vue",
    "2",
  );
  assert.equal(originalFailure.verdict, "baseline-unusable");
  assert.equal(originalFailure.failure?.side, "original");
  assert.equal(formattedFailure.verdict, "semantic-diff");
  assert.equal(formattedFailure.reasonCode, "formatted-baseline-unusable");
  assert.equal(formattedFailure.failure?.side, "formatted");
});

test("comparison harness failures retain the selected route and compiler provenance", () => {
  const project = {
    id: "synthetic",
    fixtureDir: "/fixture",
    hydrated: true,
    revision: "a".repeat(40),
    vueGlobs: ["src/**/*.vue"],
  };
  const result = compareCorpusFile(
    "<template><p /></template>\n",
    "<template><p /></template>\n",
    project,
    "src/App.vue",
    { routeId: "vue2", dialect: "2" },
    () => {
      throw new Error("synthetic harness crash");
    },
  );
  assert.equal(result.category, "baseline-unusable");
  assert.equal(result.dialect?.verdict, "baseline-unusable");
  assert.equal(result.dialect?.reasonCode, "comparison-harness-unusable");
  assert.equal(result.dialect?.failure?.side, "harness");
  assert.equal(result.dialect?.baseline.id, "vue2.6");
  assert.match(result.differences.join("\n"), /synthetic harness crash/);
});

test("compiler messages canonicalize nested object key order", () => {
  assert.deepEqual(
    normalizeCompilerMessages([
      { message: "bad", context: { z: 1, a: ["x", { second: 2, first: 1 }] } },
      "  spaced   message  ",
    ]),
    ['{"context":{"a":["x",{"first":1,"second":2}],"z":1},"message":"bad"}', "spaced message"],
  );
  assert.throws(() => assertNoCompilerErrors(undefined), /omitted its errors array/);
  assert.deepEqual(blockSignature({ template: { attrs: { ä: "last", z: "first" } } }), {
    template: [
      null,
      null,
      [
        ["z", "first"],
        ["ä", "last"],
      ],
      null,
    ],
    script: null,
    scriptSetup: null,
    styles: [],
    customBlocks: [],
  });
});

test("legacy comparison separates compiler failures from harness normalization failures", () => {
  const parse = () => ({ template: { content: "<p />" } });
  const compilerFailure = compileLegacySide(
    "<template><p /></template>",
    "App.vue",
    parse,
    () => {
      throw new Error("compiler crashed");
    },
    () => null,
  );
  assert.deepEqual(compilerFailure, {
    ok: false,
    stage: "template-compile",
    message: "compiler crashed",
  });

  const harnessFailure = compileLegacySide(
    "<template><p /></template>",
    "App.vue",
    parse,
    () => ({ render: "ok" }),
    () => {
      throw new Error("normalizer crashed");
    },
  );
  assert.deepEqual(harnessFailure, {
    ok: false,
    stage: "semantic-normalize",
    message: "normalizer crashed",
  });
});

test("Vue 0 and 1 dialect hooks fail before selecting a Vue 3 baseline", () => {
  for (const dialect of ["0.10", "0.11", "1"] as const) {
    const result = compareSfcWithDialectBaseline(
      "<template><p /></template>\n",
      "<template><p /></template>\n",
      "Legacy.vue",
      dialect,
    );
    assert.equal(result.verdict, "baseline-unusable");
    assert.equal(result.failure?.stage, "adapter-load");
    assert.equal(result.baseline.dialect, dialect);
    assert.equal(result.baseline.package, null);
    assert.notEqual(result.baseline.id, "vue3");
  }
});

test("Vue 3 keeps the existing structural comparator", () => {
  const source = '<template><button :title="label">{{ label }}</button></template>\n';
  assert.equal(compareSfcWithDialectBaseline(source, source, "App.vue", "3").verdict, "equivalent");
  assertSemanticDiff(
    compareSfcWithDialectBaseline(
      source,
      '<template><button :title="other">{{ label }}</button></template>\n',
      "App.vue",
      "3",
    ),
  );
});
