import assert from "node:assert/strict";
import test from "node:test";

import {
  createVizeLintConfigFromDist,
  lintWorkspaceFixture,
  runIsolatedPluginLoadProbe,
  typecheckVitePlusConfigConsumer,
} from "./test-support/vite-plus-workspace.ts";

const VIOLATING_SFC = `<script setup lang="ts">
const items = ["a", "b", "c"];
const html = "<strong>hi</strong>";
</script>

<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
  <div v-html="html" />
</template>
`;

const CLEAN_SFC = `<script setup lang="ts">
const items = ["a", "b", "c"];
</script>

<template>
  <ul>
    <li v-for="item in items" :key="item">{{ item }}</li>
  </ul>
</template>
`;

const createVizeLintConfig = await createVizeLintConfigFromDist();

void test("createVizeLintConfig emits the whole Vite+ lint block, including jsPlugins", () => {
  assert.deepEqual(
    createVizeLintConfig({
      preset: "incremental",
      rules: {
        "no-console": "warn",
        "vize/vue/require-v-for-key": "error",
        "vize/vue/no-v-html": "warn",
      },
      settings: { helpLevel: "none" },
    }),
    {
      jsPlugins: ["oxlint-plugin-vize"],
      plugins: ["vue"],
      rules: {
        "no-console": "warn",
        "vize/vue/require-v-for-key": "error",
        "vize/vue/no-v-html": "warn",
      },
      settings: {
        vize: {
          helpLevel: "none",
          preset: "incremental",
        },
      },
    },
  );
});

void test("createVizeLintConfig merges built-in Oxlint plugins instead of narrowing them", () => {
  // Replacing a project's plugin list would silently drop every diagnostic those
  // plugins produce, which is the same false negative this helper exists to stop.
  assert.deepEqual(
    createVizeLintConfig({
      plugins: ["eslint", "typescript", "unicorn", "oxc", "vue"],
      preset: "incremental",
    }).plugins,
    ["vue", "eslint", "typescript", "unicorn", "oxc"],
  );
});

void test("createVizeLintConfig preserves runtime rule forms with options", () => {
  assert.deepEqual(
    createVizeLintConfig({
      preset: "incremental",
      rules: {
        "no-console": "off",
        "typescript/consistent-type-imports": [
          "error",
          { disallowTypeAnnotations: false, fixStyle: "inline-type-imports" },
        ],
      },
    }).rules,
    {
      "no-console": "off",
      "typescript/consistent-type-imports": [
        "error",
        { disallowTypeAnnotations: false, fixStyle: "inline-type-imports" },
      ],
    },
  );
});

void test("packed declarations type-check as a strict Vite+ lint consumer", () => {
  typecheckVitePlusConfigConsumer(`import { createVizeLintConfig } from "oxlint-plugin-vize";
import type { VitePlusLintPlugin } from "oxlint-plugin-vize";
import { defineConfig } from "vite-plus";
import type { OxlintConfig } from "vite-plus/lint";

type Equal<Left, Right> =
  (<Value>() => Value extends Left ? 1 : 2) extends
  (<Value>() => Value extends Right ? 1 : 2) ? true : false;
type Assert<Condition extends true> = Condition;
type VitePlusPlugin = NonNullable<OxlintConfig["plugins"]>[number];
type _PluginNamesStayInSync = Assert<Equal<VitePlusLintPlugin, VitePlusPlugin>>;

export default defineConfig({
  lint: {
    ...createVizeLintConfig({
      plugins: ["typescript"],
      preset: "incremental",
      rules: {
        "no-console": "off",
        "typescript/consistent-type-imports": [
          "error",
          { disallowTypeAnnotations: false, fixStyle: "inline-type-imports" },
        ],
      },
    }),
    overrides: [
      {
        files: ["**/*.ts"],
        rules: {
          "typescript/consistent-type-imports": [
            "warn",
            { disallowTypeAnnotations: true, fixStyle: "separate-type-imports" },
          ],
        },
      },
    ],
  },
});

createVizeLintConfig({
  // @ts-expect-error Vite+ keeps built-in plugin names as a closed union.
  plugins: ["not-a-vite-plus-plugin"],
  rules: {
    // @ts-expect-error Unknown severities must not be widened to string or any.
    "no-console": "verbose",
  },
});
`);
});

void test('createVizeLintConfig keeps the rule map and settings.vize.preset in lockstep for "all"', () => {
  const config = createVizeLintConfig({ preset: "all" });

  // "all" spans every bundle, so the runtime gate has to be disabled entirely.
  // Gating an all-bundles rule map by any single preset silently suppresses the
  // rules that only belong to the other bundles.
  assert.deepEqual(config.settings, { vize: { preset: "incremental" } });
  assert.equal(config.rules["vize/ecosystem/router-link-require-to"], "error");
  assert.equal(config.rules["vize/script/no-options-api"], "error");
});

void test("the emitted lint block makes Oxlint report exactly the configured vize diagnostics", () => {
  const config = createVizeLintConfig({
    preset: "incremental",
    rules: {
      "no-unused-vars": "off",
      "vize/vue/require-v-for-key": "error",
      "vize/vue/no-v-html": "warn",
    },
    settings: { helpLevel: "none" },
  });

  assert.deepEqual(
    lintWorkspaceFixture({ config, filename: "src/Violating.vue", source: VIOLATING_SFC }),
    [
      {
        code: "vize(vue/require-v-for-key)",
        filename: "src/Violating.vue",
        labels: [{ column: 2, line: 2 }],
        message:
          "Elements in iteration expect to have 'v-bind:key' directives. (at <template>:8:9)\n" +
          "    Details:\n" +
          "      Element: <li>",
        severity: "error",
      },
      {
        code: "vize(vue/no-v-html)",
        filename: "src/Violating.vue",
        labels: [{ column: 2, line: 2 }],
        message:
          "v-html can lead to XSS attacks. (at <template>:10:8)\n" +
          "    Details:\n" +
          "      Avoid using it with user-provided content",
        severity: "warning",
      },
    ],
  );
});

void test("the emitted lint block reports nothing for a clean SFC", () => {
  const config = createVizeLintConfig({
    preset: "incremental",
    rules: {
      "no-unused-vars": "off",
      "vize/vue/require-v-for-key": "error",
      "vize/vue/no-v-html": "warn",
    },
    settings: { helpLevel: "none" },
  });

  assert.deepEqual(
    lintWorkspaceFixture({ config, filename: "src/Clean.vue", source: CLEAN_SFC }),
    [],
  );
});

void test("an unknown vize rule id throws instead of reporting nothing", () => {
  assert.throws(
    () =>
      createVizeLintConfig({
        preset: "incremental",
        rules: { "vize/vue/require-v-for-keys": "error" },
      }),
    new Error(
      "Unknown Vize rule id: vize/vue/require-v-for-keys. " +
        "Check the id against the rules oxlint-plugin-vize registers.",
    ),
  );
});

void test("a preset-suppressed vize rule id throws instead of reporting nothing", () => {
  // `script/no-options-api` belongs to the "opinionated" bundle only, so the
  // bridge's runtime gate would drop it under "essential" and report nothing.
  assert.throws(
    () =>
      createVizeLintConfig({
        preset: "essential",
        rules: { "vize/script/no-options-api": "error" },
      }),
    new Error(
      'Vize rule id outside the "essential" preset: vize/script/no-options-api. ' +
        'Use preset: "incremental" to run an explicit rule subset, or pick the preset that owns these rules.',
    ),
  );
});

void test("a plugin copy that cannot resolve the native binding refuses to load", () => {
  // A binding that fails to load must be a loud error. Degrading to an
  // importable plugin with zero rules would look exactly like a clean codebase.
  assert.deepEqual(runIsolatedPluginLoadProbe(), {
    outcome: "threw",
    reason: "Failed to load the Vize native binding",
  });
});
