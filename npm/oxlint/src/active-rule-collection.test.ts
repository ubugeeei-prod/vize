import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import type { Context } from "@oxlint/plugins";
import { afterAll, afterEach, it, vi } from "vite-plus/test";
import { collectActiveRule, getActiveRuleDiagnostics } from "./active-rule-collection.ts";
import {
  clearFileStateCache,
  getDiagnosticsForRule,
  getFileState,
  getFileStateCacheStats,
} from "./file-state.ts";
import type { PatinaDiagnostic, PatinaRuleOptions, PatinaSettings } from "./model.js";
import { getRuleOptions } from "./rule-options.ts";

const { nativeLint } = vi.hoisted(() => ({ nativeLint: vi.fn() }));
vi.mock("./binding.js", () => ({ lintPatina: nativeLint }));
const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-active-rules-"));
const manifest = JSON.parse(
  fs.readFileSync(new URL("../../../tests/_fixtures/n8n-adoption.json", import.meta.url), "utf8"),
);
const frozenRules = manifest.adoption.rules as Record<string, unknown>;
const source =
  "<script setup>const html='';</script>\n<template><MyPanel someProp='x' v-html='html'/></template>\n";

function contextFor(filename: string, settings: PatinaSettings = {}): Context {
  return {
    filename,
    physicalFilename: filename,
    settings: { vize: settings },
    sourceCode: { text: "const html='';" },
  } as unknown as Context;
}
function fixture(name = "Owned.vue"): string {
  const filename = path.join(root, name);
  fs.writeFileSync(filename, source);
  return filename;
}
function optionsFor(rule: string, entry: unknown): PatinaRuleOptions | undefined {
  return getRuleOptions(rule, Array.isArray(entry) ? entry.slice(1) : []);
}
function run(context: Context, rules: Record<string, unknown>) {
  const program = {};
  const names = Object.keys(rules).filter((name) => rules[name] !== "off");
  for (const id of names)
    collectActiveRule(context, program, id.slice(5), optionsFor(id.slice(5), rules[id]));
  return names.map((id) => ({
    rule: id.slice(5),
    diagnostics: getActiveRuleDiagnostics(context, program, id.slice(5))!.diagnostics,
  }));
}
function diagnostic(rule: string, message: string): PatinaDiagnostic {
  return {
    rule,
    message,
    severity: "warning",
    help: null,
    location: {
      start: { line: 2, column: 20, offset: 57 },
      end: { line: 2, column: 28, offset: 65 },
    },
  };
}
function installNative() {
  nativeLint.mockImplementation(
    (_source, filename, _settings, names: string[], options: PatinaRuleOptions) => ({
      filename,
      errorCount: 0,
      warningCount: 0,
      diagnostics: names.flatMap((name) => [
        diagnostic(name, JSON.stringify({ options: optionsForNative(name, options) })),
        diagnostic(name, "duplicate"),
      ]),
    }),
  );
}
function optionsForNative(name: string, options: PatinaRuleOptions) {
  if (name === "vue/attribute-hyphenation") return options?.attributeHyphenation;
  if (name === "vue/component-name-in-template-casing")
    return options?.componentNameInTemplateCasing;
  if (name === "vue/sfc-element-order") return options?.sfcElementOrder;
  return undefined;
}
afterEach(() => {
  clearFileStateCache();
  nativeLint.mockReset();
});
afterAll(() => fs.rmSync(root, { recursive: true, force: true }));

it("collects the actual frozen 51 options before one native call, without authored hints", () => {
  installNative();
  const context = contextFor(fixture(), { preset: "incremental", helpLevel: "none" });
  const program = {};
  for (const [id, entry] of Object.entries(frozenRules))
    collectActiveRule(context, program, id.slice(5), optionsFor(id.slice(5), entry));
  assert.equal(nativeLint.mock.calls.length, 0);
  const expected = Object.entries(frozenRules).map(([id, entry]) => ({
    rule: id.slice(5),
    diagnostics: getDiagnosticsForRule(
      context,
      getFileState(context),
      id.slice(5),
      optionsFor(id.slice(5), entry),
    ),
  }));
  assert.equal(nativeLint.mock.calls.length, 51);
  nativeLint.mockClear();
  const actual = Object.keys(frozenRules).map((id) => ({
    rule: id.slice(5),
    diagnostics: getActiveRuleDiagnostics(context, program, id.slice(5))!.diagnostics,
  }));
  assert.deepEqual(actual, expected);
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.deepEqual(
    nativeLint.mock.calls[0][3],
    Object.keys(frozenRules)
      .map((id) => id.slice(5))
      .sort(),
  );
  assert.deepEqual(nativeLint.mock.calls[0][4], {
    attributeHyphenation: "always",
    componentNameInTemplateCasing: "PascalCase",
    sfcElementOrder: { order: ["script", "template", "style"] },
  });
  assert.equal((context.settings as { vize: PatinaSettings }).vize.rules, undefined);
});

it("actual scalar override resets options and activation changes invalidate only the current bounded result", () => {
  installNative();
  const context = contextFor(fixture(), { preset: "incremental" });
  const original = run(context, frozenRules);
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.deepEqual(run(context, frozenRules), original);
  assert.equal(nativeLint.mock.calls.length, 1);
  const overridden = {
    ...frozenRules,
    "vize/vue/attribute-hyphenation": "warn",
    "vize/vue/no-multiple-template-root": "off",
  };
  const changed = run(context, overridden);
  assert.equal(nativeLint.mock.calls.length, 2);
  assert.equal(nativeLint.mock.calls[1][3].length, 50);
  assert.equal(nativeLint.mock.calls[1][4].attributeHyphenation, undefined);
  assert.notDeepEqual(changed, original);
  assert.deepEqual(run(context, overridden), changed);
  assert.equal(nativeLint.mock.calls.length, 2);
  assert.deepEqual(run(context, frozenRules), original);
  assert.equal(nativeLint.mock.calls.length, 3);
});

it("source, filename, settings and nested in-place options cannot reuse stale results", () => {
  installNative();
  const filename = fixture();
  const settings: PatinaSettings = { preset: "incremental", helpLevel: "none" };
  const context = contextFor(filename, settings);
  const rules = structuredClone(frozenRules);
  run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 1);
  fs.writeFileSync(filename, "\n" + source);
  run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 2);
  assert.equal(nativeLint.mock.calls[1][0], "\n" + source);
  fs.writeFileSync(filename, source);
  run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 3);
  settings.helpLevel = "short";
  run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 4);
  (rules["vize/vue/sfc-element-order"] as [string, { order: string[] }])[1].order.reverse();
  run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 5);
  assert.deepEqual(nativeLint.mock.calls[4][4].sfcElementOrder.order, [
    "style",
    "template",
    "script",
  ]);
  run(contextFor(fixture("Other.vue"), settings), rules);
  assert.equal(nativeLint.mock.calls.length, 6);
});

it("two extracted programs share a physical result while changed active maps refresh report identity", () => {
  installNative();
  const filename = fixture();
  const first = contextFor(filename),
    second = contextFor(filename);
  (second.sourceCode as { text: string }).text = "export default {};";
  run(first, frozenRules);
  const state = getFileState(first);
  state.reportedDiagnostics.add("original witness");
  run(second, frozenRules);
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.ok(state.reportedDiagnostics.has("original witness"));
  run(second, { ...frozenRules, "vize/vue/attribute-hyphenation": ["error", "never"] });
  assert.equal(nativeLint.mock.calls.length, 2);
  assert.equal(state.reportedDiagnostics.size, 0);
});

it("empty or ineligible rule visitors make no native call and mid-pass physical changes fail closed", () => {
  installNative();
  const filename = fixture(),
    context = contextFor(filename),
    program = {};
  assert.equal(getActiveRuleDiagnostics(context, program, "vue/no-v-html"), undefined);
  assert.equal(nativeLint.mock.calls.length, 0);
  collectActiveRule(context, program, "vue/no-v-html");
  fs.writeFileSync(filename, "\n" + source);
  assert.throws(
    () => getActiveRuleDiagnostics(context, program, "vue/no-v-html"),
    /changed before rule execution/u,
  );
  assert.equal(nativeLint.mock.calls.length, 0);
});

it("actual-option caches retain the 128-entry LRU bound, including empty diagnostics", () => {
  nativeLint.mockReturnValue({
    filename: "Owned.vue",
    diagnostics: [],
    errorCount: 0,
    warningCount: 0,
  });
  const rules = {
    "vize/vue/no-v-html": "error",
    "vize/vue/attribute-hyphenation": ["error", "always"],
  };
  const contexts = Array.from({ length: 129 }, (_, index) =>
    contextFor(fixture(`Lru${index}.vue`)),
  );
  for (const context of contexts) run(context, rules);
  assert.equal(nativeLint.mock.calls.length, 129);
  assert.deepEqual(getFileStateCacheStats(), { capacity: 128, entries: 128 });
  run(contexts[128], rules);
  assert.equal(nativeLint.mock.calls.length, 129);
  run(contexts[0], rules);
  assert.equal(nativeLint.mock.calls.length, 130);
});

it("registration snapshots nested options before execution and releases completed traversal state", () => {
  installNative();
  const context = contextFor(fixture()),
    program = {};
  const options: PatinaRuleOptions = {
    sfcElementOrder: { order: ["script", "template", "style"] },
  };
  collectActiveRule(context, program, "vue/sfc-element-order", options);
  options.sfcElementOrder!.order!.reverse();
  getActiveRuleDiagnostics(context, program, "vue/sfc-element-order");
  assert.deepEqual(nativeLint.mock.calls[0][4].sfcElementOrder.order, [
    "script",
    "template",
    "style",
  ]);
  assert.equal(getActiveRuleDiagnostics(context, program, "vue/sfc-element-order"), undefined);
});

it("a selected type-aware rule enables the original runtime and relays its whole packet once", () => {
  const runtime = diagnostic("type/corsa-runtime", "runtime unavailable");
  nativeLint.mockReturnValue({
    filename: "Owned.vue",
    diagnostics: [runtime],
    errorCount: 0,
    warningCount: 1,
  });
  const context = contextFor(fixture(), {
    corsaPath: "/owned/runtime",
    locale: "ja",
    helpLevel: "short",
  });
  const result = run(context, { "vize/type/no-any": "error", "vize/type/no-unknown": "warn" });
  assert.equal(nativeLint.mock.calls.length, 1);
  assert.deepEqual(nativeLint.mock.calls[0][2], {
    corsaPath: "/owned/runtime",
    locale: "ja",
    helpLevel: "short",
    typeAware: true,
  });
  assert.deepEqual(
    result.map((row) => row.diagnostics),
    [[runtime], []],
  );
});

it("changes after the first exit cannot mix old native packets with later source/settings", () => {
  installNative();
  for (const change of ["source", "settings"] as const) {
    const filename = fixture(`AfterFirst-${change}.vue`);
    const settings: PatinaSettings = { helpLevel: "none" };
    const context = contextFor(filename, settings),
      program = {};
    collectActiveRule(context, program, "vue/no-v-html");
    collectActiveRule(context, program, "vue/attribute-hyphenation");
    const before = nativeLint.mock.calls.length;
    getActiveRuleDiagnostics(context, program, "vue/no-v-html");
    assert.equal(nativeLint.mock.calls.length, before + 1);
    if (change === "source") fs.writeFileSync(filename, "\n" + source);
    else settings.helpLevel = "short";
    assert.throws(
      () => getActiveRuleDiagnostics(context, program, "vue/attribute-hyphenation"),
      /changed before rule execution/u,
    );
    assert.equal(nativeLint.mock.calls.length, before + 1);
    assert.equal(
      getActiveRuleDiagnostics(context, program, "vue/attribute-hyphenation"),
      undefined,
    );
  }
});
