import assert from "node:assert/strict";
import { test } from "node:test";
import { resolveViteConfigExport } from "../../crates/vize_carton/src/config/loader/vite-runtime.mjs";

test("top-level Vite+ lint settings and namespaced rule tuples share native projection", async () => {
  const lint = {
    plugins: ["vue"],
    jsPlugins: ["oxlint-plugin-vize"],
    settings: { vize: { preset: "incremental", typeAware: false, helpLevel: "short" } },
    rules: {
      "no-console": "error",
      "vize/vue/no-v-html": 0,
      "vize/vue/prefer-props-shorthand": 1,
      "vize/vue/require-v-for-key": 2,
      "vize/vue/no-mutating-props": ["warn", { shallowOnly: true }],
    },
  };
  const before = structuredClone(lint);
  assert.deepEqual(await resolveViteConfigExport({ lint }), {
    typeChecker: { jsxTypecheck: true },
    linter: {
      preset: "incremental",
      typeAware: false,
      rules: {
        "vue/no-v-html": "off",
        "vue/prefer-props-shorthand": "warn",
        "vue/require-v-for-key": "error",
        "vue/no-mutating-props": "warn",
      },
      ruleOptions: { "vue/no-mutating-props": { shallowOnly: true } },
    },
  });
  assert.deepEqual(lint, before);
});

test("Vue lint overrides only explicit leaves and canonical scoped arrays retain precedence", async () => {
  const source = {
    lint: {
      settings: { vize: { preset: "opinionated" } },
      rules: { "vize/vue/no-v-html": "error", "vize/vue/prefer-props-shorthand": "warn" },
      vue: { preset: "incremental", rules: { "vue/no-v-html": "warn" } },
    },
    vue: { linter: { rules: { "vue/no-v-html": "off" } }, entries: [] },
  };
  assert.deepEqual(await resolveViteConfigExport(source), {
    typeChecker: { jsxTypecheck: true },
    linter: {
      preset: "incremental",
      rules: { "vue/no-v-html": "off", "vue/prefer-props-shorthand": "warn" },
    },
    entries: [],
  });
  assert.deepEqual(await resolveViteConfigExport({ lint: { vue: false } }), {
    typeChecker: { jsxTypecheck: true },
    linter: { enabled: false },
  });
  assert.deepEqual(await resolveViteConfigExport({ lint: { rules: { "no-console": "error" } } }), {
    typeChecker: { jsxTypecheck: true },
  });
});

test("invalid shared native lint settings and unowned overrides refuse", async () => {
  for (const lint of [
    { settings: { vize: { preset: "typo" } } },
    { settings: { vize: { typeAware: "false" } } },
    { overrides: [{ rules: { "vize/vue/no-v-html": "warn" } }] },
    { overrides: [{ files: ["src/**"], rules: { "vize/vue/no-v-html": "warn" } }] },
    { rules: { "vize/vue/no-v-html": "warning" } },
  ])
    await assert.rejects(resolveViteConfigExport({ lint }), /lint\./);
});
