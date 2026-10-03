import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { stripTypeScriptTypes } from "node:module";
import { hash } from "../../npm/native/scripts/formatter-history-build.mjs";

export const originalViteSources = {
  "npm/builder/vite/src/vite-plus/import-sorting.test.ts":
    "02b2910051683dca5108a47a622db63dc086f5a0a9b599b20ab33e043473832c",
  "npm/builder/vite/src/vite-plus/config.test.ts":
    "82570d354aa7cfb9300bca05be49e7bd451694beea019b635cac1ccb7c8056a4",
  "crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue":
    "ae86278aa0a0b247d9e027b703699f02cf370ea37da9dd96368adf3ff334b106",
};

// These immutable sources own inputs, not configuration/output oracles.
export function originalViteInputs(root) {
  const files = Object.fromEntries(
    Object.entries(originalViteSources).map(([file, expected]) => {
      const raw = fs.readFileSync(path.join(root, file));
      assert.equal(hash(raw), expected, `original Vite vector changed: ${file}`);
      return [file, raw.toString()];
    }),
  );
  const fixture = files["crates/vize_glyph/tests/fixtures/sort-imports/UserCard.vue"];
  const raw = stripTypeScriptTypes(files["npm/builder/vite/src/vite-plus/import-sorting.test.ts"], {
    mode: "strip",
  });
  const start = raw.indexOf("const controls");
  assert(start >= 0 && raw.indexOf("const controls", start + 1) < 0);
  const open = raw.indexOf("= [", start) + 2;
  const end = raw.indexOf("\n  ];", open) + 4;
  assert(open > start && end > open);
  const controls = JSON.parse(
    JSON.stringify(vm.runInNewContext(raw.slice(open, end), { fixture }, { timeout: 1_000 })),
  );
  assert.equal(controls.length, 5);
  for (const [source, setting] of controls) {
    assert.equal(typeof source, "string");
    assert(setting && typeof setting === "object" && !Array.isArray(setting));
  }
  const sharedTitle = raw.indexOf(
    'test("Vite+ shares Oxfmt import sorting with native Vue formatting"',
  );
  assert(sharedTitle >= 0);
  const sharedStart = raw.indexOf("async () => {", sharedTitle) + "async () => {".length;
  const sharedEnd = raw.indexOf("const result =", sharedStart);
  const shared = JSON.parse(
    JSON.stringify(
      vm.runInNewContext(
        `(() => { ${raw.slice(sharedStart, sharedEnd)} return sorting; })()`,
        {},
        { timeout: 1_000 },
      ),
    ),
  );
  const config = stripTypeScriptTypes(files["npm/builder/vite/src/vite-plus/config.test.ts"], {
    mode: "strip",
  });
  const title = config.indexOf(
    'test("extends merges native sections and Vite+ options with deterministic alias precedence"',
  );
  assert(title >= 0);
  const body = config.indexOf("async () => {", title) + "async () => {".length;
  const stop = config.indexOf("const result =", body);
  // The input-only VM never resolves a config or runs a tool. Declared constructor
  // placeholders retain the exact literal bases before the actual public calls.
  const inherited = JSON.parse(
    JSON.stringify(
      vm.runInNewContext(
        `(() => { ${config.slice(body, stop)} return { base, own }; })()`,
        { defineConfig: (input) => input, Promise: { resolve: (input) => input } },
        { timeout: 1_000 },
      ),
    ),
  );
  return { controls, fixture, shared, inherited };
}

const integration = {
  plugin: false,
  tasks: false,
  conflicts: false,
  check: false,
  lint: false,
};

// Supplemental public scenarios keep every original law/vector unchanged.
// Whole expected resolved objects below are authored, never copied from IO.
export function viteConfigurationPlans(root) {
  const { controls, shared, inherited } = originalViteInputs(root);
  const cases = controls.map(([script, sorting], index) => ({
    id: `original-script-${index}`,
    script,
    source: { fmt: { sortImports: sorting } },
    formatter: { sortImports: sorting },
    vp: { fmt: { sortImports: sorting }, lint: undefined },
    options: { ...integration, fmt: undefined },
  }));
  for (const [id, sorting, own] of [
    ["public-shared", shared, undefined],
    ["public-false", { order: "asc" }, false],
    ["public-object", { order: "asc" }, { order: "desc" }],
  ]) {
    cases.push({
      id,
      script: controls[0][0],
      source: {
        fmt: { sortImports: sorting, ...(own === undefined ? {} : { vize: { sortImports: own } }) },
      },
      formatter: { sortImports: own === undefined ? sorting : own },
      vp: { fmt: { sortImports: sorting }, lint: undefined },
      options: { ...integration, fmt: own === undefined ? undefined : true },
    });
  }
  cases.push({
    id: "public-inheritance",
    script: controls[0][0],
    base: inherited.base,
    source: Object.fromEntries(Object.entries(inherited.own).filter(([key]) => key !== "extends")),
    promisedBase: inherited.own.extends[1],
    formatter: { singleQuote: true, tabWidth: 4 },
    vp: { server: { port: 4321, host: true }, lint: {}, fmt: { printWidth: 90 } },
    options: { ...integration, check: true, lint: true, fmt: true },
  });
  return cases.map((fixture) => {
    const rootConfig = { formatter: fixture.formatter };
    let originalEntry = rootConfig;
    if (fixture.base) {
      originalEntry = {
        typeChecker: { strict: true },
        linter: { preset: "essential", rules: { "vue/no-v-html": "error" } },
        formatter: fixture.formatter,
      };
      Object.assign(rootConfig, originalEntry, {
        compiler: {},
        vite: {},
        languageServer: {},
        musea: {},
        globalTypes: {},
      });
    }
    return {
      ...fixture,
      integration,
      native: { ...rootConfig, entries: [{ ...rootConfig }, { ...originalEntry }] },
      public: { ...fixture.vp, plugins: [], pack: undefined, run: undefined },
      input: `<script setup lang="ts">\n${fixture.script}</script>\n`,
    };
  });
}
