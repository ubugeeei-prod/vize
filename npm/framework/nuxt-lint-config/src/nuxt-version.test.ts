import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import test from "node:test";

import { buildNuxtLintPlan, resolveNuxtLintDirs, resolveNuxtLintFeatures } from "./index.ts";

const corpus = JSON.parse(
  readFileSync(new URL("../test/nuxt-version-compat/corpus.json", import.meta.url), "utf8"),
) as {
  cases: Array<{
    id: string;
    nuxtVersion: 2 | 3 | 4;
    expectedRules: Record<string, string>;
  }>;
};
const features = resolveNuxtLintFeatures({ stylistic: true }, () => true);
const dirs = resolveNuxtLintDirs({ src: ["src"], pages: ["src/pages"] });

for (const entry of corpus.cases) {
  void test(`${entry.id}: version-specific runtime rules match the authored corpus`, () => {
    const plan = buildNuxtLintPlan(features, dirs, entry.nuxtVersion);
    const runtime = plan.filter(({ name }) => name === "nuxt/rules" || name === "nuxt/pages");
    assert.deepEqual(Object.assign({}, ...runtime.map(({ rules }) => rules)), entry.expectedRules);
    assert.deepEqual(
      plan.filter(({ name }) => name !== "nuxt/rules" && name !== "nuxt/pages"),
      [
        {
          name: "nuxt/ignores",
          ignores: [
            "**/dist",
            "**/node_modules",
            "**/.nuxt",
            "**/.output",
            "**/.vercel",
            "**/.netlify",
            "**/public",
          ],
        },
        { name: "nuxt/setup", globals: { $fetch: "readonly" } },
        {
          name: "nuxt/vue/single-root",
          files: [
            "src/components/**/*.server.{js,ts,jsx,tsx,vue}",
            "src/layouts/**/*.{js,ts,jsx,tsx,vue}",
            "src/pages/**/*.{js,ts,jsx,tsx,vue}",
          ],
          rules: { "vue/no-multiple-template-root": "error" },
        },
        {
          name: "nuxt/nuxt-config",
          files: ["**/.config/nuxt.?([cm])[jt]s?(x)", "**/nuxt.config.?([cm])[jt]s?(x)"],
          rules: { "nuxt/no-nuxt-config-test-key": "error" },
        },
        {
          name: "nuxt/sort-config",
          files: ["**/.config/nuxt.?([cm])[jt]s?(x)", "**/nuxt.config.?([cm])[jt]s?(x)"],
          rules: { "nuxt/nuxt-config-keys-order": "error" },
        },
        {
          name: "nuxt/disables/routes",
          files: [
            "src/app.{js,ts,jsx,tsx,vue}",
            "src/components/*/**/*.{js,ts,jsx,tsx,vue}",
            "src/error.{js,ts,jsx,tsx,vue}",
            "src/layouts/**/*.{js,ts,jsx,tsx,vue}",
            "src/pages/**/*.{js,ts,jsx,tsx,vue}",
          ],
          rules: { "vue/multi-word-component-names": "off" },
        },
      ],
    );
  });
}

void test("standalone callers retain the modern plan when the major is omitted", () => {
  assert.deepEqual(buildNuxtLintPlan(features, dirs), buildNuxtLintPlan(features, dirs, 3));
});
