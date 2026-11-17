// `vue.plugins` resolves once into the canonical plan inside the shared
// vite.config projector (#13074). Empty plans are byte-identical; capabilities
// no product implements fail closed; inspection shows refused capabilities.
import assert from "node:assert/strict";
import { test } from "node:test";

import {
  RESOLVED_PLUGIN_PLAN,
  inspectViteConfigPlugins,
  mergeVueConfigOptions,
  projectViteConfigSource,
} from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";
import { definePlugin } from "../../npm/plugin-sdk/index.js";
import { defineVizePlugin } from "../../npm/plugin-sdk/v2.js";

const design = defineVizePlugin<{ strict: boolean }>({
  apiVersion: 2,
  stability: "experimental",
  name: "@acme/design",
  version: "1.0.0",
  setup(api, options) {
    api.lint.rule({
      id: "no-raw-color",
      meta: { messages: { raw: "Use a token." } },
      visit: ["ui.bind"],
      run(context) {
        if (options.strict) context.report({ node: context.nodes[0]!, messageId: "raw" });
      },
    });
  },
});

const bases = [
  {},
  { vue: {} },
  { vue: { linter: { preset: "essential" }, files: ["src/**/*.vue"] } },
  { root: "app", vue: { version: "3", compile: { vapor: true } } },
  {
    lint: { rules: { "vize/vue/require-v-for-key": "error" } },
    vue: { formatter: { tabWidth: 4 } },
  },
];

test("empty plugin lists project byte-identically to absent ones", async () => {
  for (const base of bases) {
    const absent = JSON.stringify(
      await projectViteConfigSource(structuredClone(base), undefined, "/p"),
    );
    for (const plugins of [[], [false, null, undefined]]) {
      const source = {
        ...structuredClone(base),
        vue: { ...structuredClone(base.vue ?? {}), plugins },
      };
      const projected = await projectViteConfigSource(source, undefined, "/p");
      assert.equal(JSON.stringify(projected), absent, JSON.stringify(base));
      assert.equal(Object.getOwnPropertySymbols(projected as object).length, 0);
    }
  }
});

const routes = defineVizePlugin({
  apiVersion: 2,
  stability: "experimental",
  name: "@acme/routes",
  version: "1.0.0",
  setup(api) {
    api.sfc.block("route", { lang: "json" });
  },
});

const tracking = defineVizePlugin({
  apiVersion: 2,
  stability: "experimental",
  name: "@acme/tracking",
  version: "1.0.0",
  setup(api) {
    api.vue.directive("track", { modifiers: { strict: { optional: false } } });
  },
});

test("capabilities no product implements fail closed in every projection", async () => {
  await assert.rejects(projectViteConfigSource({ vue: { plugins: [routes()] } }, undefined, "/p"), {
    code: "capability-unadmitted",
    message:
      "[plugin/capability-unadmitted] @acme/routes: no Vize product implements sfc.custom-block `route:json` yet; run `vize plugin inspect` for the plan",
  });
  // A capability loads only once every product that consumes it implements it.
  await assert.rejects(
    projectViteConfigSource({ vue: { plugins: [tracking()] } }, undefined, "/p"),
    {
      code: "capability-unadmitted",
      message:
        "[plugin/capability-unadmitted] @acme/tracking: vue.directive `track` is not implemented by typecheck yet; run `vize plugin inspect` for the plan",
    },
  );
  await assert.rejects(projectViteConfigSource({ vue: { plugins: {} } }), {
    message: "[plugin/entry-invalid] vue.plugins: must be an array",
  });
  await assert.rejects(projectViteConfigSource({ vue: { entries: [{ plugins: [] }] } }), {
    message: /vue\.entries\[0\]/,
  });
});

test("an admitted plan travels as canonical data with its in-process handle", async () => {
  const v1 = definePlugin({ name: "legacy", version: "1", cacheInputs: [], rules: { check() {} } });
  const projected = (await projectViteConfigSource(
    { vue: { plugins: [design({ strict: true }), v1], linter: { preset: "essential" } } },
    undefined,
    "/p",
  )) as Record<string | symbol, unknown>;
  const resolved = projected[RESOLVED_PLUGIN_PLAN] as { digest: string; json: string };
  assert.deepEqual(Object.keys(projected), [
    "typeChecker",
    "linter",
    "__vizeProjectRoot",
    "__vizePluginPlan",
  ]);
  assert.deepEqual(projected.__vizePluginPlan, {
    digest: resolved.digest,
    plan: JSON.parse(resolved.json),
  });
  assert.equal(JSON.stringify(JSON.parse(JSON.stringify(projected))), JSON.stringify(projected));
  const absent = await projectViteConfigSource(
    { vue: { linter: { preset: "essential" } } },
    undefined,
    "/p",
  );
  const { __vizePluginPlan: _plan, ...rest } = projected;
  assert.equal(JSON.stringify(rest), JSON.stringify(absent));
});

test("inspection reports the plan and every refused capability", async () => {
  const inspection = await inspectViteConfigPlugins(
    () => ({ vue: { plugins: [design({ strict: false })] } }),
    { mode: "production", command: "build" },
  );
  assert.deepEqual(inspection.order, ["@acme/design"]);
  assert.equal(inspection.admission.lint?.status, "ready");
  assert.equal(inspection.admission.lsp?.status, "ready");
  assert.equal(inspection.admission.format?.status, "unused");
  assert.deepEqual(inspection.plugins[0]?.options, { strict: false });
  const empty = await inspectViteConfigPlugins({ vue: {} });
  assert.deepEqual(empty.plugins, []);
  assert.match(empty.digest, /^[0-9a-f]{64}$/);
});

test("withVue composition replaces plugin lists like other Vue option arrays", () => {
  const first = design({ strict: true });
  const second = design({ strict: false });
  const merged = mergeVueConfigOptions(
    { plugins: [first], linter: { preset: "essential" } },
    { plugins: [second] },
  ) as { plugins: unknown[] };
  assert.deepEqual(merged.plugins, [second]);
  assert.equal(merged.plugins[0], second);
});
