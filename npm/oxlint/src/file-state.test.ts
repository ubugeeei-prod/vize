import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { Context } from "@oxlint/plugins";
import { it } from "vite-plus/test";

import {
  clearFileStateCache,
  getDiagnosticsForRule,
  getFileState,
  getFileStateCacheStats,
  markRuleAsReported,
} from "./file-state.ts";
import { appendScriptlessWorkaround, resolveWorkaroundSource } from "./workaround.ts";

function createContext(filename: string, extractedScript: string): Context {
  return {
    filename,
    physicalFilename: filename,
    settings: {},
    sourceCode: { text: extractedScript },
  } as unknown as Context;
}

it("standalone scripts preserve native source locations", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-script-"));
  const filename = path.join(root, "nuxt.config.ts");
  const source = "export default { test: true };\n";

  try {
    fs.writeFileSync(filename, source);
    const state = getFileState(createContext(filename, source));
    assert.equal(state.usesOriginalLocations, true);
    assert.equal(state.source, source);
    assert.equal(state.filename, filename);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("unchanged source reuses revision-safe file work", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-reuse-"));
  const filename = path.join(root, "App.vue");

  try {
    fs.writeFileSync(filename, "<template><div /></template>\n");
    const context = createContext(filename, "const component = {};\n");
    const first = getFileState(context);
    first.partialDiagnosticsByRule.set("vue/example", []);

    const reused = getFileState(context);

    assert.strictEqual(reused, first);
    assert.strictEqual(reused.partialDiagnosticsByRule, first.partialDiagnosticsByRule);
    assert.equal(getFileStateCacheStats().entries, 1);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("same filename with changed source starts a fresh reporting revision", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-revision-"));
  const filename = path.join(root, "App.vue");
  const context = createContext(filename, "const component = {};\n");

  try {
    fs.writeFileSync(filename, "<template><div>first</div></template>\n");
    const first = getFileState(context);
    first.partialDiagnosticsByRule.set("vue/example", []);
    first.allDiagnosticsByRule = new Map([["vue/example", []]]);
    first.allDiagnosticsIncludesTypeAware = true;
    first.requestedRules.add("vue/example");
    first.reportedTypeAwareRuntimeDiagnostic = true;
    assert.equal(markRuleAsReported(first, "vue/example"), true);
    assert.equal(markRuleAsReported(first, "vue/example"), false);

    fs.writeFileSync(filename, "<template><div>other</div></template>\n");
    const changed = getFileState(context);

    assert.notStrictEqual(changed, first);
    assert.match(changed.source, /other/u);
    assert.equal(changed.partialDiagnosticsByRule.size, 0);
    assert.equal(changed.allDiagnosticsByRule, null);
    assert.equal(changed.allDiagnosticsIncludesTypeAware, false);
    assert.equal(changed.requestedRules.size, 0);
    assert.equal(changed.reportedTypeAwareRuntimeDiagnostic, false);
    assert.equal(markRuleAsReported(changed, "vue/example"), true);
    assert.equal(getFileStateCacheStats().entries, 1);

    fs.writeFileSync(filename, "<template><div>first</div></template>\n");
    const reverted = getFileState(context);
    assert.notStrictEqual(reverted, first, "A → B → A must not revive A's reporting state");
    assert.notStrictEqual(reverted, changed);
    assert.equal(markRuleAsReported(reverted, "vue/example"), true);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("diagnostics follow the latest physical revision under one filename", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-diagnostics-"));
  const filename = path.join(root, "App.vue");
  const context = createContext(filename, "const items = [];");
  const ruleName = "vue/require-v-for-key";

  try {
    fs.writeFileSync(
      filename,
      '<template><div v-for="item in items" :key="item">{{ item }}</div></template>',
    );
    const first = getFileState(context);
    assert.equal(getDiagnosticsForRule(context, first, ruleName).length, 0);

    fs.writeFileSync(filename, '<template><div v-for="item in items">{{ item }}</div></template>');
    const changed = getFileState(context);
    const diagnostics = getDiagnosticsForRule(context, changed, ruleName);

    assert.notStrictEqual(changed, first);
    assert.equal(diagnostics.length, 1);
    assert.equal(diagnostics[0]?.rule, ruleName);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("changed extracted script refreshes only revision-local mapping work", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-extracted-"));
  const filename = path.join(root, "App.vue");

  try {
    fs.writeFileSync(filename, "<script setup>const value = 1;</script>\n");
    const first = getFileState(createContext(filename, "const value = 1;\n"));
    first.scriptMap = null;
    assert.equal(markRuleAsReported(first, "vue/example"), true);
    const changed = getFileState(createContext(filename, "const value = 2;\n"));

    assert.strictEqual(changed, first);
    assert.equal(changed.extractedScript, "const value = 2;\n");
    assert.equal(changed.scriptMap, undefined);
    assert.equal(markRuleAsReported(changed, "vue/example"), false);
    assert.equal(getFileStateCacheStats().entries, 1);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("long-lived file-state cache stays bounded and evicts the LRU entry", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-file-state-lru-"));
  const { capacity } = getFileStateCacheStats();
  const contexts: Context[] = [];
  const states = [];

  try {
    for (let index = 0; index < capacity; index += 1) {
      const filename = path.join(root, `${index}.vue`);
      fs.writeFileSync(filename, `<template><div>${index}</div></template>\n`);
      const context = createContext(filename, "const component = {};\n");
      contexts.push(context);
      states.push(getFileState(context));
    }

    assert.strictEqual(getFileState(contexts[0]!), states[0]);
    const overflowFilename = path.join(root, "overflow.vue");
    fs.writeFileSync(overflowFilename, "<template><div>overflow</div></template>\n");
    getFileState(createContext(overflowFilename, "const overflow = true;\n"));

    assert.deepEqual(getFileStateCacheStats(), { capacity, entries: capacity });
    assert.strictEqual(getFileState(contexts[0]!), states[0]);
    assert.notStrictEqual(getFileState(contexts[1]!), states[1]);
    assert.equal(getFileStateCacheStats().entries, capacity);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("only recognizes a scriptless workaround marker at byte zero", () => {
  const fallbackFilename = "/Users/example/fallback.vue";
  const workaround = appendScriptlessWorkaround("<template />", "/Users/example/Real.vue");

  for (const prefix of ["\uFEFF", "<!-- banner -->\n", "#!/usr/bin/env node\n"]) {
    const source = `${prefix}${workaround}`;
    assert.deepEqual(resolveWorkaroundSource(source, fallbackFilename), {
      filename: fallbackFilename,
      source,
      usesOriginalLocations: false,
    });
  }
});
