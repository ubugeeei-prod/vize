import assert from "node:assert/strict";
import test from "node:test";

import type { VizeNuxtCompilerOptions } from "../compiler-options.ts";
import { createNuxtLintInspectorProvider, setupNuxtLintInspector } from "./inspector.ts";

const payload = {
  schema: "vize.inspector.lint-plan",
  version: 1,
  root: "/project",
  items: [],
  files: [],
};

void test("Nuxt lint inspector forwards the cached plan, root, files, and freshness", async () => {
  const freshness: boolean[] = [];
  let received: unknown;
  const provider = createNuxtLintInspectorProvider(
    {
      root: "/project",
      async resolvePlan(fresh) {
        freshness.push(fresh ?? false);
        return [{ name: "nuxt/rules", rules: { "nuxt/prefer-import-meta": "error" } }];
      },
    },
    {
      inspectLintPlan(plan, root, files) {
        received = { plan: JSON.parse(plan), root, files };
        return JSON.stringify(payload);
      },
    },
  );

  assert.deepEqual(await provider({ files: ["app.vue"], fresh: true }), payload);
  assert.deepEqual(freshness, [true]);
  assert.deepEqual(received, {
    plan: {
      items: [{ name: "nuxt/rules", rules: { "nuxt/prefer-import-meta": "error" } }],
    },
    root: "/project",
    files: ["app.vue"],
  });
});

void test("Nuxt lint inspector rejects malformed native payloads", async () => {
  const generation = {
    root: "/project",
    async resolvePlan() {
      return [];
    },
  };
  const malformed = createNuxtLintInspectorProvider(generation, {
    inspectLintPlan: () => "not json",
  });
  await assert.rejects(malformed({ files: [], fresh: false }), SyntaxError);

  const wrongSchema = createNuxtLintInspectorProvider(generation, {
    inspectLintPlan: () => JSON.stringify({ ...payload, schema: "wrong" }),
  });
  await assert.rejects(wrongSchema({ files: [], fresh: false }), /returned an invalid payload/u);
});

void test("Nuxt lint inspector wiring is development-only and preserves explicit providers", () => {
  const generation = {
    configFile: "/project/.nuxt/oxlint.config.json",
    root: "/project",
    async regenerate() {
      return false;
    },
    async resolvePlan() {
      return [];
    },
  };
  const disabled: VizeNuxtCompilerOptions = {};
  setupNuxtLintInspector(disabled, generation, false);
  assert.equal(disabled.inspector, undefined);

  const explicit = () => ({ explicit: true });
  const configured: VizeNuxtCompilerOptions = { inspector: { lintPlan: explicit } };
  setupNuxtLintInspector(configured, generation, true);
  assert.equal(configured.inspector?.lintPlan, explicit);

  const automatic: VizeNuxtCompilerOptions = {};
  setupNuxtLintInspector(automatic, generation, true);
  assert.equal(typeof automatic.inspector?.lintPlan, "function");
});
