import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";

import type { Context } from "@oxlint/plugins";
import { it, vi } from "vite-plus/test";
import * as binding from "./binding.ts";
import { parseRuleSelection } from "./rule-selection.ts";

import {
  clearFileStateCache,
  getDiagnosticsForRule,
  getFileState,
  getFileStateCacheStats,
  markDiagnosticAsReported,
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

const sampleDiagnostic = {
  rule: "vue/example",
  severity: "error",
  message: "example",
  help: null,
  location: {
    start: { line: 1, column: 1, offset: 0 },
    end: { line: 1, column: 2, offset: 1 },
  },
} as const;

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
    assert.equal(markDiagnosticAsReported(first, sampleDiagnostic), true);
    assert.equal(markDiagnosticAsReported(first, sampleDiagnostic), false);

    fs.writeFileSync(filename, "<template><div>other</div></template>\n");
    const changed = getFileState(context);

    assert.notStrictEqual(changed, first);
    assert.match(changed.source, /other/u);
    assert.equal(changed.partialDiagnosticsByRule.size, 0);
    assert.equal(changed.allDiagnosticsByRule, null);
    assert.equal(changed.allDiagnosticsIncludesTypeAware, false);
    assert.equal(changed.requestedRules.size, 0);
    assert.equal(changed.reportedTypeAwareRuntimeDiagnostic, false);
    assert.equal(markDiagnosticAsReported(changed, sampleDiagnostic), true);
    assert.equal(getFileStateCacheStats().entries, 1);

    fs.writeFileSync(filename, "<template><div>first</div></template>\n");
    const reverted = getFileState(context);
    assert.notStrictEqual(reverted, first, "A → B → A must not revive A's reporting state");
    assert.notStrictEqual(reverted, changed);
    assert.equal(markDiagnosticAsReported(reverted, sampleDiagnostic), true);
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
    assert.equal(markDiagnosticAsReported(first, sampleDiagnostic), true);
    const changed = getFileState(createContext(filename, "const value = 2;\n"));

    assert.strictEqual(changed, first);
    assert.equal(changed.extractedScript, "const value = 2;\n");
    assert.equal(changed.scriptMap, undefined);
    assert.equal(markDiagnosticAsReported(changed, sampleDiagnostic), false);
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

it("does not strip user-authored whitespace blocks that mimic the workaround marker", () => {
  const fallbackFilename = "/Users/example/fallback.vue";
  const workaround = appendScriptlessWorkaround("<template />", "/Users/example/Real.vue");
  const openTagEnd = workaround.indexOf(">") + 1;
  const source = `${workaround.slice(0, openTagEnd)}\n</script>\n<template />`;

  assert.deepEqual(resolveWorkaroundSource(source, fallbackFilename), {
    filename: fallbackFilename,
    source,
    usesOriginalLocations: false,
  });
});

it("the selected n8n native pass preserves complete per-rule diagnostics and options", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-n8n-native-parity-"));
  const corpus = new URL(
    "../../../tests/_fixtures/differential/linter/oxlint-batched-selection/",
    import.meta.url,
  );
  const source = fs.readFileSync(new URL("App.vue.txt", corpus), "utf8");
  const rules = JSON.parse(fs.readFileSync(new URL("n8n-rules.json", corpus), "utf8")) as Record<
    string,
    unknown
  >;
  const selection = parseRuleSelection(rules)!;
  const filename = path.join(root, "App.vue");
  const settings = { preset: "incremental" as const, helpLevel: "none" as const, rules };
  fs.writeFileSync(filename, source);
  const context = createContext(filename, "const items = [1, 2];");
  Object.assign(context, { settings: { vize: settings } });
  const sortWholeDiagnostics = (diagnostics: unknown[]) =>
    diagnostics.map((entry) => JSON.stringify(entry)).sort();
  const expected = selection.names.flatMap(
    (name) =>
      binding.lintPatina(source, filename, settings, [name], selection.optionsByRule.get(name))
        .diagnostics,
  );
  assert.ok(expected.some((diagnostic) => diagnostic.rule === "vue/attribute-hyphenation"));
  assert.ok(expected.some((diagnostic) => diagnostic.rule === "vue/require-v-for-key"));
  assert.ok(expected.some((diagnostic) => diagnostic.rule === "vue/no-v-html"));

  const spy = vi.spyOn(binding, "lintPatina");
  try {
    const actual = selection.names.flatMap((name) => [
      ...getDiagnosticsForRule(
        context,
        getFileState(context),
        name,
        selection.optionsByRule.get(name),
      ),
    ]);
    assert.deepEqual(sortWholeDiagnostics(actual), sortWholeDiagnostics(expected));
    assert.equal(spy.mock.calls.length, 1);
  } finally {
    spy.mockRestore();
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});

it("a missing runtime preset retains explicit rules outside general-recommended", () => {
  clearFileStateCache();
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-no-settings-"));
  const source = fs.readFileSync(
    new URL(
      "../../../tests/_fixtures/differential/linter/oxlint-batched-selection/NoSettings.vue.txt",
      import.meta.url,
    ),
    "utf8",
  );
  const filename = path.join(root, "NoSettings.vue");
  fs.writeFileSync(filename, source);
  const context = createContext(filename, "\nexport default {};\n");
  const rule = "script/no-options-api";
  const expected = binding.lintPatina(source, filename, { preset: "incremental" }, [
    rule,
  ]).diagnostics;
  assert.equal(expected.length, 1);
  try {
    assert.deepEqual(getDiagnosticsForRule(context, getFileState(context), rule), expected);
  } finally {
    clearFileStateCache();
    fs.rmSync(root, { force: true, recursive: true });
  }
});
