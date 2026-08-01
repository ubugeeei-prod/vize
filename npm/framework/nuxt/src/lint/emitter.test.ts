import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { mkdir, mkdtemp, rm, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import test from "node:test";

import { renderNuxtOxlintConfig } from "./emitter.ts";
import type { NuxtLintConfigItem } from "@vizejs/nuxt-lint-config";

const plan: NuxtLintConfigItem[] = [
  {
    name: "nuxt/ignores",
    ignores: ["**/dist", "**/.nuxt"],
  },
  {
    name: "nuxt/setup",
    globals: { $fetch: "readonly" },
  },
  {
    name: "nuxt/vue/single-root",
    files: ["app/layouts/**/*.{js,ts,jsx,tsx,vue}", "app/pages/**/*.{js,ts,jsx,tsx,vue}"],
    rules: { "vue/no-multiple-template-root": "error" },
  },
  {
    name: "nuxt/rules",
    rules: { "nuxt/prefer-import-meta": "error" },
  },
  {
    name: "nuxt/pages",
    files: ["app/pages/**/*.{js,ts,jsx,tsx,vue}"],
    ignores: ["app/pages/generated/**"],
    globals: { definePageMeta: "readonly" },
    rules: { "nuxt/no-page-meta-runtime-values": "warn" },
  },
  {
    name: "nuxt/disables/routes",
    files: ["app/pages/**/*.{js,ts,jsx,tsx,vue}"],
    rules: { "vue/multi-word-component-names": "off" },
  },
];

const expected = `{
  "plugins": [
    "vue"
  ],
  "jsPlugins": [
    {
      "name": "vize",
      "specifier": "../node_modules/oxlint-plugin-vize/dist/index.mjs"
    }
  ],
  "settings": {
    "vize": {
      "preset": "incremental"
    }
  },
  "ignorePatterns": [
    "**/dist",
    "**/.nuxt"
  ],
  "globals": {
    "$fetch": "readonly"
  },
  "rules": {
    "vize/nuxt/prefer-import-meta": "error"
  },
  "overrides": [
    {
      "files": [
        "app/layouts/**/*.{js,ts,jsx,tsx,vue}",
        "app/pages/**/*.{js,ts,jsx,tsx,vue}"
      ],
      "rules": {
        "vize/vue/no-multiple-template-root": "error"
      }
    },
    {
      "files": [
        "app/pages/**/*.{js,ts,jsx,tsx,vue}"
      ],
      "excludeFiles": [
        "app/pages/generated/**"
      ],
      "globals": {
        "definePageMeta": "readonly"
      },
      "rules": {
        "vize/nuxt/no-page-meta-runtime-values": "warn"
      }
    },
    {
      "files": [
        "app/pages/**/*.{js,ts,jsx,tsx,vue}"
      ],
      "rules": {
        "vize/vue/multi-word-component-names": "off"
      }
    }
  ]
}
`;

void test("emitter pins the whole generated oxlint artifact byte for byte", () => {
  assert.equal(
    renderNuxtOxlintConfig(plan, "../node_modules/oxlint-plugin-vize/dist/index.mjs"),
    expected,
  );
});

void test("emitter preserves its engine-neutral input", () => {
  const before = structuredClone(plan);
  renderNuxtOxlintConfig(plan, "./plugin.mjs");
  assert.deepEqual(plan, before);
});

void test("emitter prefixes only Patina rule ids", () => {
  const output = JSON.parse(
    renderNuxtOxlintConfig(
      [
        {
          name: "mixed-rules",
          rules: {
            "no-console": "warn",
            "typescript/no-explicit-any": "error",
            "script/no-options-api": "off",
            "vize/no-legacy-api": "error",
          },
        },
      ],
      "./plugin.mjs",
    ),
  ) as { rules: Record<string, string> };

  assert.deepEqual(output.rules, {
    "no-console": "warn",
    "typescript/no-explicit-any": "error",
    "vize/script/no-options-api": "off",
    "vize/vize/no-legacy-api": "error",
  });
});

void test("oxlint loads the generated artifact and applies its ordered overrides", async (t) => {
  const root = await mkdtemp(path.join(os.tmpdir(), "vize-nuxt-oxlint-artifact-"));
  t.after(() => rm(root, { force: true, recursive: true }));
  const generatedDir = path.join(root, ".nuxt");
  const source = path.join(root, "app", "pages", "index.ts");
  await mkdir(path.dirname(source), { recursive: true });
  await mkdir(generatedDir);
  await writeFile(source, "$fetch('/api')\n");
  await writeFile(
    path.join(generatedDir, "plugin.mjs"),
    `export default {
  meta: { name: "vize" },
  rules: Object.fromEntries([
    "vue/no-multiple-template-root",
    "nuxt/prefer-import-meta",
    "nuxt/no-page-meta-runtime-values",
    "vue/multi-word-component-names",
  ].map(name => [name, {
    create: context => ({
      Program: node => context.report({
        node,
        message: \`\${name}:\${context.languageOptions.globals.$fetch}\`
      })
    })
  }]))
}
`,
  );
  await writeFile(
    path.join(generatedDir, "oxlint.config.json"),
    renderNuxtOxlintConfig(plan, "./plugin.mjs"),
  );
  await writeFile(
    path.join(root, "oxlint.config.mts"),
    `import { readFileSync } from "node:fs"
const generatedUrl = new URL("./.nuxt/oxlint.config.json", import.meta.url)
const config = JSON.parse(readFileSync(generatedUrl, "utf8"))
config.jsPlugins = config.jsPlugins.map(plugin => typeof plugin === "string"
  ? new URL(plugin, generatedUrl).href
  : { ...plugin, specifier: new URL(plugin.specifier, generatedUrl).href })
export default config
`,
  );

  const packageRequire = createRequire(import.meta.url);
  const oxlintPackage = packageRequire.resolve("oxlint/package.json");
  const result = spawnSync(
    process.execPath,
    [
      path.join(path.dirname(oxlintPackage), "bin", "oxlint"),
      "-c",
      "oxlint.config.mts",
      "-f",
      "json",
      path.relative(root, source),
    ],
    { cwd: root, encoding: "utf8" },
  );
  assert.equal(result.error, undefined);
  assert.equal(result.status, 1);
  const output = JSON.parse(result.stdout) as {
    diagnostics: Array<{ code: string; message: string; severity: string }>;
  };

  assert.deepEqual(
    output.diagnostics
      .map(({ code, message, severity }) => ({ code, message, severity }))
      .sort((left, right) => left.message.localeCompare(right.message)),
    [
      {
        code: "vize(nuxt/no-page-meta-runtime-values)",
        message: "nuxt/no-page-meta-runtime-values:readonly",
        severity: "warning",
      },
      {
        code: "vize(nuxt/prefer-import-meta)",
        message: "nuxt/prefer-import-meta:readonly",
        severity: "error",
      },
      {
        code: "vize(vue/no-multiple-template-root)",
        message: "vue/no-multiple-template-root:readonly",
        severity: "error",
      },
    ],
  );
});

void test("later global plan items win without losing earlier keys", () => {
  const prototypeGlobal = Object.fromEntries([["__proto__", "readonly"]]) as Record<
    string,
    "readonly"
  >;
  const rendered = renderNuxtOxlintConfig(
    [
      {
        name: "one",
        globals: { shared: "readonly", first: "readonly", ...prototypeGlobal },
      },
      { name: "two", globals: { shared: "writable", second: "readonly" } },
    ],
    "./plugin.mjs",
  );
  const config = JSON.parse(rendered) as { globals: Record<string, string> };

  assert.deepEqual(
    config.globals,
    Object.fromEntries([
      ["shared", "writable"],
      ["first", "readonly"],
      ["__proto__", "readonly"],
      ["second", "readonly"],
    ]),
  );
});
