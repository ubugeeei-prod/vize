import assert from "node:assert/strict";
import { test } from "node:test";

import {
  loadPinnedShikiVueOracle,
  validateOracleGrammarClosure,
  validateOraclePin,
} from "./support/shiki-vue-oracle.ts";
import { compareSemanticSpans } from "./support/syntax-semantic-comparison.ts";
import { tokenizeSemanticSource } from "./support/syntax-semantic-divergence.ts";
import { loadVueTextMateGrammar } from "./support/vue-textmate.ts";

test("pinned @shikijs/langs/vue oracle carries exact independent provenance", async () => {
  const oracle = await loadPinnedShikiVueOracle();
  try {
    const source = `<script setup lang="ts">\nconst count = 1\n</script>\n<template>{{ count }}</template>\n`;
    const result = tokenizeSemanticSource(
      oracle.grammar,
      source,
      oracle.rootScope,
      "oracle/synthetic.vue",
    );
    assert.equal(result.lineCount, 5);
    assert.ok(result.tokenCount > 5);
    const evidence = oracle.getEvidence();
    assert.deepEqual(
      {
        module: evidence.module,
        moduleSha256: evidence.moduleSha256,
        package: evidence.package,
        grammarClosureSha256: evidence.grammarClosureSha256,
        version: evidence.version,
      },
      {
        module: "@shikijs/langs/vue",
        moduleSha256: "610450422f0a3b39468c42c078ff9e2dfd2d55045d31bbbb95173eba5986962d",
        package: "@shikijs/langs",
        grammarClosureSha256: "9d6e6d7c4109574dec2aedcd2adb501b327c7e69943ead3adfee0ecccd379960",
        version: "4.0.2",
      },
    );
    assert.ok(evidence.requestedScopes.includes("text.html.vue"));
  } finally {
    oracle.registry.dispose();
  }
});

test("oracle provenance validation fails closed on version, grammar, and license drift", () => {
  const exact = {
    license: "MIT",
    licenseSha256: "7a9d8d01038aeacf9e5bcdabbddf2a7815200dce9fc1118468cc553e00ae3eee",
    moduleSha256: "610450422f0a3b39468c42c078ff9e2dfd2d55045d31bbbb95173eba5986962d",
    name: "@shikijs/langs",
    version: "4.0.2",
  };
  assert.doesNotThrow(() => validateOraclePin(exact));
  for (const changed of [
    { ...exact, version: "4.0.3" },
    { ...exact, moduleSha256: "0".repeat(64) },
    { ...exact, licenseSha256: "0".repeat(64) },
  ]) {
    assert.throws(() => validateOraclePin(changed), /oracle provenance drifted/);
  }
  assert.throws(
    () => validateOracleGrammarClosure([{ scopeName: "text.html.vue" }]),
    /oracle grammar closure drifted/,
  );
});

test("oracle fails closed when source activates an unresolved embedded grammar", async () => {
  const oracle = await loadPinnedShikiVueOracle();
  try {
    const source = `<docs lang="md">\n\`\`\`ignore\nignored-file\n\`\`\`\n</docs>\n`;
    assert.throws(
      () =>
        tokenizeSemanticSource(oracle.grammar, source, oracle.rootScope, "oracle/unresolved.vue"),
      /activated unresolved oracle grammar/,
    );
    assert.ok(oracle.getEvidence().unresolvedScopeSentinels.includes("source.ignore"));
  } finally {
    oracle.registry.dispose();
  }
});

test("shipped Art Vue grammar records intentional Vize-only spans against the oracle", async () => {
  const artVue = await loadVueTextMateGrammar("source.art-vue");
  const oracle = await loadPinnedShikiVueOracle();
  try {
    const source = [
      '<art title="Button" component="Button">',
      '  <variant name="typed">',
      '    <Button :items="makeList<User>()" />',
      "  </variant>",
      "</art>",
    ].join("\n");
    const shipped = tokenizeSemanticSource(
      artVue.grammar,
      source,
      "source.art-vue",
      "vize/editor-art-vue",
    );
    const upstream = tokenizeSemanticSource(
      oracle.grammar,
      source,
      oracle.rootScope,
      "oracle/editor-art-vue",
    );
    const comparison = compareSemanticSpans(
      "editor/Variant.art.vue",
      source,
      shipped.semanticSpans,
      upstream.semanticSpans,
    );

    assert.equal(artVue.getEvidence().rootScope, "source.art-vue");
    assert.deepEqual(
      comparison.falsePositives.map(({ category, line, startColumn, endColumn }) => ({
        category,
        line,
        startColumn,
        endColumn,
      })),
      [
        { category: "tag", line: 2, startColumn: 4, endColumn: 11 },
        { category: "attribute", line: 2, startColumn: 12, endColumn: 16 },
        { category: "tag", line: 3, startColumn: 6, endColumn: 12 },
        { category: "attribute", line: 3, startColumn: 13, endColumn: 19 },
        { category: "tag", line: 4, startColumn: 5, endColumn: 12 },
      ],
    );
    assert.deepEqual(comparison.falseNegatives, []);
  } finally {
    artVue.registry.dispose();
    oracle.registry.dispose();
  }
});
