import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { pathToFileURL } from "node:url";
import { parse } from "yaml";

import type * as NuxtLintConfig from "../../npm/framework/nuxt-lint-config/src/index.ts";

const root = path.resolve(import.meta.dirname, "../..");
const packageDir = path.join(root, "npm/framework/nuxt-lint-config");

async function loadSource<T>(file: string): Promise<T> {
  return import(pathToFileURL(path.join(root, file)).href) as Promise<T>;
}

test("shareable Nuxt lint config owns the preset implementation", async () => {
  const shareable = await loadSource<typeof NuxtLintConfig>(
    "npm/framework/nuxt-lint-config/src/index.ts",
  );
  const nuxtIndex = fs.readFileSync(
    path.join(root, "npm/framework/nuxt/src/lint/index.ts"),
    "utf8",
  );

  assert.match(nuxtIndex, /export \* from "@vizejs\/nuxt-lint-config";/);
  for (const name of [
    "collectNuxtLintDirs",
    "resolveNuxtLintDirs",
    "resolveNuxtLintFeatures",
    "buildNuxtLintPlan",
  ] as const) {
    assert.equal(typeof shareable[name], "function", `${name} must remain public`);
  }

  for (const implementation of ["dirs.ts", "features.ts", "paths.ts", "plan.ts"]) {
    assert.equal(
      fs.existsSync(path.join(root, "npm/framework/nuxt/src/lint", implementation)),
      false,
      `@vizejs/nuxt must not retain a second ${implementation}`,
    );
  }
});

test("shareable Nuxt lint config resolves a complete standalone plan", async () => {
  const preset = await loadSource<typeof NuxtLintConfig>(
    "npm/framework/nuxt-lint-config/src/index.ts",
  );
  const dirs = preset.resolveNuxtLintDirs(undefined);
  const features = preset.resolveNuxtLintFeatures(undefined, () => true);

  assert.deepEqual(preset.buildNuxtLintPlan(features, dirs), [
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
        "app/components/**/*.server.{js,ts,jsx,tsx,vue}",
        "app/layouts/**/*.{js,ts,jsx,tsx,vue}",
        "app/pages/**/*.{js,ts,jsx,tsx,vue}",
        "components/**/*.server.{js,ts,jsx,tsx,vue}",
        "layouts/**/*.{js,ts,jsx,tsx,vue}",
        "pages/**/*.{js,ts,jsx,tsx,vue}",
      ],
      rules: { "vue/no-multiple-template-root": "error" },
    },
    { name: "nuxt/rules", rules: { "nuxt/prefer-import-meta": "error" } },
    {
      name: "nuxt/pages",
      files: ["app/pages/**/*.{js,ts,jsx,tsx,vue}", "pages/**/*.{js,ts,jsx,tsx,vue}"],
      rules: { "nuxt/no-page-meta-runtime-values": "error" },
    },
    {
      name: "nuxt/nuxt-config",
      files: ["**/.config/nuxt.?([cm])[jt]s?(x)", "**/nuxt.config.?([cm])[jt]s?(x)"],
      rules: { "nuxt/no-nuxt-config-test-key": "error" },
    },
    {
      name: "nuxt/disables/routes",
      files: [
        "app.{js,ts,jsx,tsx,vue}",
        "app/app.{js,ts,jsx,tsx,vue}",
        "app/components/*/**/*.{js,ts,jsx,tsx,vue}",
        "app/error.{js,ts,jsx,tsx,vue}",
        "app/layouts/**/*.{js,ts,jsx,tsx,vue}",
        "app/pages/**/*.{js,ts,jsx,tsx,vue}",
        "components/*/**/*.{js,ts,jsx,tsx,vue}",
        "error.{js,ts,jsx,tsx,vue}",
        "layouts/**/*.{js,ts,jsx,tsx,vue}",
        "pages/**/*.{js,ts,jsx,tsx,vue}",
      ],
      rules: { "vue/multi-word-component-names": "off" },
    },
  ]);
});

test("shareable Nuxt lint config is a public dependency of the Nuxt module", () => {
  const packageJson = JSON.parse(
    fs.readFileSync(path.join(packageDir, "package.json"), "utf8"),
  ) as {
    exports?: unknown;
    name?: string;
    publishConfig?: { access?: string };
    scripts?: Record<string, string>;
  };
  const nuxtPackage = JSON.parse(
    fs.readFileSync(path.join(root, "npm/framework/nuxt/package.json"), "utf8"),
  ) as { dependencies?: Record<string, string> };

  assert.equal(packageJson.name, "@vizejs/nuxt-lint-config");
  assert.equal(packageJson.publishConfig?.access, "public");
  assert.ok(packageJson.exports);
  assert.equal(packageJson.scripts?.build, "vp pack");
  assert.equal(packageJson.scripts?.dev, "vp pack --watch");
  assert.equal(nuxtPackage.dependencies?.["@vizejs/nuxt-lint-config"], "workspace:*");
});

test("release workflow publishes the Nuxt module after its runtime lint dependencies", () => {
  const workflow = parse(
    fs.readFileSync(path.join(root, ".github/workflows/release.yml"), "utf8"),
  ) as { jobs?: Record<string, { needs?: string | string[] }> };
  const needs = workflow.jobs?.["release-npm-nuxt"]?.needs;
  const nuxtNeeds = Array.isArray(needs) ? needs : needs == null ? [] : [needs];

  for (const dependency of ["release-npm-nuxt-lint-config", "release-npm-oxlint-plugin"]) {
    assert.ok(nuxtNeeds.includes(dependency), `release-npm-nuxt must wait for ${dependency}`);
  }
});
