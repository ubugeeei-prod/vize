import assert from "node:assert/strict";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { performance } from "node:perf_hooks";
import type { Context } from "@oxlint/plugins";
import { it, vi } from "vite-plus/test";
import * as binding from "./binding.ts";
import {
  clearFileStateCache,
  getDiagnosticsForRule,
  getFileState,
  getFileStateCacheStats,
} from "./file-state.ts";
import { parseRuleSelection } from "./rule-selection.ts";
import type { PatinaSettings } from "./model.ts";
import {
  effectiveRules,
  editorPrefix,
  manifest,
  sha256,
  verifyCorpus,
} from "../scripts/n8n-replay-inputs.mjs";

// This campaign is explicitly invoked by source-built Actions, never silently
// skipped by the ordinary package suite when a fixture/addon is unavailable.
it("qualifies every licensed n8n input, all selected options and bounded revision caches", () => {
  assert.equal(process.env.GITHUB_ACTIONS, "true");
  assert.ok(process.env.VIZE_N8N_NATIVE_CUSTODY);
  const output = process.env.VIZE_N8N_REPLAY_OUTPUT!;
  assert.ok(output);
  fs.mkdirSync(output, { recursive: true });
  const inventory = verifyCorpus();
  fs.writeFileSync(path.join(output, "inventory.json"), JSON.stringify(inventory, null, 2) + "\n");
  const catalog = binding.getPatinaRules();
  fs.writeFileSync(path.join(output, "catalog.json"), JSON.stringify(catalog, null, 2) + "\n");
  const frozen = parseRuleSelection(manifest.adoption.rules)!;
  assert.equal(frozen.names.length, 51);
  for (const name of frozen.names)
    assert.ok(
      catalog.some((rule) => rule.name === name),
      name,
    );
  const vectorPath = path.join(output, "bridge-vectors.jsonl");
  fs.writeFileSync(vectorPath, "");
  let nativeCalls = 0;
  const original = binding.lintPatina;
  const spy = vi.spyOn(binding, "lintPatina").mockImplementation((...args) => {
    nativeCalls++;
    return original(...args);
  });
  const totals: Record<string, { coldCalls: number; warmCalls: number; elapsedMs: number }> = {};
  const context = (filename: string, settings: PatinaSettings) =>
    ({
      filename,
      physicalFilename: filename,
      settings: { vize: settings },
      sourceCode: { text: fs.readFileSync(filename, "utf8") },
    }) as unknown as Context;
  const vector = (ctx: Context, rules: Record<string, unknown>) => {
    const selection = parseRuleSelection(rules)!;
    const state = getFileState(ctx);
    return frozen.names.map((name) => ({
      rule: name,
      configured: rules["vize/" + name],
      options: selection.optionsByRule.get(name) ?? null,
      diagnostics: selection.optionsByRule.has(name)
        ? [...getDiagnosticsForRule(ctx, state, name, selection.optionsByRule.get(name))]
        : [],
    }));
  };
  const modes = [
    "frozenPerRule",
    "frozenBatch",
    "scopedPerRule",
    "scopedSharedBatch",
    "scopedEffectiveBatch",
  ];
  const temporary = fs.mkdtempSync(path.join(os.tmpdir(), "vize-n8n-cache-controls-"));
  try {
    for (const [index, entry] of inventory.files.entries()) {
      const vectors = new Map<string, ReturnType<typeof vector>>();
      for (const mode of modes) {
        clearFileStateCache();
        process.env.VIZE_N8N_REPLAY_PHASE = mode;
        const rules = mode.startsWith("frozen")
          ? manifest.adoption.rules
          : effectiveRules(entry.file);
        const hint = ["frozenBatch", "scopedSharedBatch"].includes(mode)
          ? manifest.adoption.rules
          : mode === "scopedEffectiveBatch"
            ? rules
            : undefined;
        const ctx = context(path.join(inventory.fixture, entry.file), {
          ...manifest.adoption.settings.vize,
          ...(hint ? { rules: hint } : {}),
        });
        const start = performance.now(),
          before = nativeCalls;
        const cold = vector(ctx, rules),
          coldCalls = nativeCalls - before;
        const warmBefore = nativeCalls,
          warm = vector(ctx, rules),
          warmCalls = nativeCalls - warmBefore;
        const elapsedMs = performance.now() - start;
        fs.appendFileSync(
          vectorPath,
          JSON.stringify({
            file: entry.file,
            sourceSha256: entry.sha256,
            mode,
            rules,
            cold,
            warm,
            coldCalls,
            warmCalls,
            elapsedMs,
            cache: getFileStateCacheStats(),
          }) + "\n",
        );
        assert.deepEqual(warm, cold, `${entry.file}/${mode}: warm complete ordered vector`);
        assert.equal(warmCalls, 0);
        const expected = mode.endsWith("PerRule")
          ? parseRuleSelection(rules)!.names.length
          : mode === "scopedSharedBatch" && entry.file.startsWith(editorPrefix)
            ? 2
            : 1;
        assert.equal(coldCalls, expected, `${entry.file}/${mode}`);
        const total = (totals[mode] ??= { coldCalls: 0, warmCalls: 0, elapsedMs: 0 });
        total.coldCalls += coldCalls;
        total.warmCalls += warmCalls;
        total.elapsedMs += elapsedMs;
        vectors.set(mode, cold);
      }
      // Keep each requested rule's complete diagnostic array in native order;
      // no sorting, deduplication, field removal or count-only comparison.
      assert.deepEqual(vectors.get("frozenBatch"), vectors.get("frozenPerRule"), entry.file);
      assert.deepEqual(vectors.get("scopedSharedBatch"), vectors.get("scopedPerRule"), entry.file);
      assert.deepEqual(
        vectors.get("scopedEffectiveBatch"),
        vectors.get("scopedPerRule"),
        entry.file,
      );
      if ((index + 1) % 100 === 0)
        console.log(`n8n native/bridge ${index + 1}/${inventory.files.length}`);
    }
    clearFileStateCache();
    process.env.VIZE_N8N_REPLAY_PHASE = "cacheControls";
    const controlsPath = path.join(output, "cache-controls.jsonl");
    fs.writeFileSync(controlsPath, "");
    const file = path.join(temporary, "Revision.vue");
    const source =
      '<script setup>const html = "";</script>\n<template><MyPanel someProp="x" some-prop="x" v-html="html"/></template>\n';
    fs.writeFileSync(file, source);
    const rules = structuredClone(manifest.adoption.rules);
    const settings: PatinaSettings = { ...manifest.adoption.settings.vize, rules };
    const observe = (
      name: string,
      ctx: Context,
      selected: Record<string, unknown>,
      expected: number,
    ) => {
      const before = nativeCalls,
        result = vector(ctx, selected),
        calls = nativeCalls - before;
      fs.appendFileSync(
        controlsPath,
        JSON.stringify({
          name,
          filename: ctx.physicalFilename,
          sourceSha256: sha256(fs.readFileSync(ctx.physicalFilename)),
          settings: ctx.settings,
          selected,
          result,
          calls,
          cache: getFileStateCacheStats(),
        }) + "\n",
      );
      assert.equal(calls, expected, name);
      return result;
    };
    const first = observe("cold", context(file, settings), rules, 1);
    assert.deepEqual(observe("resident warm", context(file, settings), rules, 0), first);
    fs.writeFileSync(file, "\n\n" + source);
    const shifted = observe("physical source changed", context(file, settings), rules, 1);
    assert.notDeepEqual(shifted, first);
    fs.writeFileSync(file, source);
    assert.deepEqual(observe("physical source reverted", context(file, settings), rules, 1), first);
    rules["vize/vue/attribute-hyphenation"][1] = "never";
    const optionsChanged = observe("in-place options changed", context(file, settings), rules, 1);
    assert.notDeepEqual(optionsChanged, first);
    rules["vize/vue/attribute-hyphenation"][1] = "always";
    assert.deepEqual(
      observe("unchanged prior options safely reused", context(file, settings), rules, 0),
      first,
    );
    observe("settings changed", context(file, { ...settings, helpLevel: "full" }), rules, 1);
    const overridden = { ...rules, "vize/vue/attribute-hyphenation": "warn" };
    const fallback = observe(
      "scalar override resets options, shared hint falls back",
      context(file, settings),
      overridden,
      1,
    );
    const effective = observe(
      "effective override selection",
      context(file, { ...settings, rules: overridden }),
      overridden,
      1,
    );
    assert.deepEqual(fallback, effective);
    assert.notDeepEqual(effective, first);
    const disabled = { ...overridden, "vize/vue/no-v-html": "off" };
    const withoutHtml = observe(
      "scope activation changed",
      context(file, { ...settings, rules: disabled }),
      disabled,
      1,
    );
    assert.equal(withoutHtml.find((row) => row.rule === "vue/no-v-html")!.diagnostics.length, 0);
    assert.ok(first.find((row) => row.rule === "vue/no-v-html")!.diagnostics.length > 0);
    clearFileStateCache();
    const paths = Array.from({ length: 129 }, (_, index) =>
      path.join(temporary, `Bounded${index}.vue`),
    );
    for (const [index, filename] of paths.entries()) {
      fs.writeFileSync(filename, source);
      observe(`LRU cold ${index}`, context(filename, settings), rules, 1);
    }
    assert.deepEqual(getFileStateCacheStats(), { capacity: 128, entries: 128 });
    observe("LRU resident warm", context(paths[128], settings), rules, 0);
    observe("LRU evicted cold", context(paths[0], settings), rules, 1);
    assert.deepEqual(
      verifyCorpus(),
      inventory,
      "all upstream original bytes and Git identity stay unchanged",
    );
    fs.writeFileSync(
      path.join(output, "bridge-summary.json"),
      JSON.stringify(
        {
          totals,
          files: inventory.files.length,
          scriptless: manifest.corpus.scriptlessFiles.length,
          disabledFiles: inventory.disabledFiles,
          vectorSha256: sha256(fs.readFileSync(vectorPath)),
          limits: [
            "native + bridge context qualification; not direct SDK callback coverage",
            "frozen no-hint settings retain per-rule native calls",
            "explicit hint modes are separate optimizations",
            "two n8n-local plugins and unrelated workspace layers excluded",
            "no upstream 11-second timing claim",
          ],
        },
        null,
        2,
      ) + "\n",
    );
  } finally {
    spy.mockRestore();
    clearFileStateCache();
    fs.rmSync(temporary, { recursive: true, force: true });
  }
}, 900_000);
