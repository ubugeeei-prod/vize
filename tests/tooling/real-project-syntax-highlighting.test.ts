import assert from "node:assert/strict";
import { test } from "node:test";

import { runRealProjectSyntaxAudit } from "./support/real-project-syntax-audit.ts";
import { tokenizeSemanticSource } from "./support/syntax-semantic-divergence.ts";
import { auditTextMateSource, loadVueTextMateGrammar } from "./support/vue-textmate.ts";

test("shipped and pinned oracle grammars tokenize every selected real-project Vue file", async () => {
  const result = await runRealProjectSyntaxAudit();
  if (result.skipped) return;
  assert.ok(result.artifact);
  assert.ok(result.artifact.summary.projectCount > 0);
});

test("real-project syntax budget includes grammar startup", async () => {
  const timestamps = [0, 2];
  await assert.rejects(
    () =>
      runRealProjectSyntaxAudit(
        {
          FIXTURE_SHARD_COUNT: "1",
          FIXTURE_SHARD_INDEX: "0",
          SYNTAX_HIGHLIGHTER_ORACLE_TIMEOUT_MS: "1",
        },
        () => timestamps.shift() ?? 2,
      ),
    /syntax oracle shard exceeded 1ms/,
  );
});

test("real-project TextMate audit exercises the shipped grammar and fail-closed spans", async () => {
  await assert.rejects(
    () => loadVueTextMateGrammar("source.vize-missing-test-grammar"),
    /unresolved TextMate grammar scope: source\.vize-missing-test-grammar/,
  );
  const { grammar, registry } = await loadVueTextMateGrammar();
  try {
    const source = `<script setup lang="ts">\nconst label = 'ready'\n</script>\n<template>{{ label }}</template>\n`;
    const result = auditTextMateSource(grammar, source, "source.vue", "synthetic/App.vue");
    assert.equal(result.lineCount, 5);
    assert.ok(result.tokenCount > 5);
    const pugAttributeValue = [
      '<template lang="pug">',
      'ssh-pre(language="js").',
      "  app.use(WaveUI, { /* Some Wave UI options */ })",
      "</template>",
      "",
    ].join("\n");
    const semantic = tokenizeSemanticSource(
      grammar,
      pugAttributeValue,
      "source.vue",
      "synthetic/wave-ui-install-cdn.vue",
    );
    assert.equal(semantic.lineCount, 5);
    assert.ok(semantic.tokenCount > 5);
  } finally {
    registry.dispose();
  }
  const artVue = await loadVueTextMateGrammar("source.art-vue");
  try {
    const result = auditTextMateSource(
      artVue.grammar,
      '<art title="Button"><variant name="primary"><button /></variant></art>\n',
      "source.art-vue",
      "synthetic/Button.art.vue",
    );
    assert.ok(result.tokenCount > 5);
  } finally {
    artVue.registry.dispose();
  }

  const gapGrammar = {
    tokenizeLine() {
      return {
        ruleStack: null,
        tokens: [{ startIndex: 1, endIndex: 2, scopes: ["source.vue", "meta.tag.vue"] }],
      };
    },
  };
  assert.throws(
    () => auditTextMateSource(gapGrammar, "x", "source.vue", "gap.vue"),
    /not a contiguous positive span/,
  );
  const slowGrammar = {
    tokenizeLine() {
      return {
        ruleStack: null,
        stoppedEarly: true,
        tokens: [{ startIndex: 0, endIndex: 1, scopes: ["source.vue"] }],
      };
    },
  };
  assert.throws(
    () => auditTextMateSource(slowGrammar, "x", "source.vue", "slow.vue"),
    /exceeded 250ms/,
  );
  const oneTokenGrammar = (scope: string) => ({
    tokenizeLine() {
      return {
        ruleStack: null,
        tokens: [{ startIndex: 0, endIndex: 1, scopes: ["source.vue", scope] }],
      };
    },
  });
  const first = auditTextMateSource(oneTokenGrammar("meta.first"), "x", "source.vue", "first.vue");
  const second = auditTextMateSource(
    oneTokenGrammar("meta.second"),
    "x",
    "source.vue",
    "second.vue",
  );
  assert.notEqual(first.sha256, second.sha256, "token digest must include scope identity");
});
