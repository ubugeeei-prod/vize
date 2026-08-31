import assert from "node:assert/strict";
import { test } from "node:test";

import { evaluateBaselineAmbientEnvironment } from "../../legacy-tools/fixtures/typecheck-baseline-ambient.mjs";

/**
 * Run 31979524200: the ledger could not see the difference between a fixture typed
 * against its own dependencies and one typed against Vize's.
 *
 * Run 31979524200 breached elk's budget with `711 false positives / 894 false
 * negatives`, ratios 0.996, and classified it "Vize divergence, the vue-tsc
 * baseline loaded cleanly over the same 259 Vue files". Every existing check
 * agreed: zero configuration diagnostics, 259 Vue files on both sides, seeded
 * mutation oracle passed. 893 of the 894 were `TS2339` on `$t`, `$d`,
 * `useNuxtApp` and 50 more of elk's own Nuxt auto-imports, on component instance
 * types with 21 members where the real ones have 680.
 *
 * The cause is in the program listing the coverage check already reads and only
 * looked at `.vue` entries of. `.nuxt/nuxt.d.ts` carries
 * `/// <reference types="vue-router" />`; a type reference directive resolves by
 * walking `node_modules` upward and ignores `compilerOptions.paths`, so it left
 * the fixture — pnpm does not hoist transitive dependencies — and was answered
 * by Vize's own `vue-router@4.5.1`, which brought Vize's `vue@3.6.0-beta.10`
 * into a program that already had elk's `vue@3.5.30`. `declare module 'vue'`
 * merges by module identity, so elk's augmentations went to one copy and its
 * compiled components were typed against the other.
 *
 * The listings below are the package paths of that program and of the same
 * fixture once isolated, taken from replaying the run's byte-identical generated
 * config against the pinned elk revision; only the absolute prefix is
 * normalized. Locally, closing the escape moves the baseline from 902
 * diagnostics to 9.
 */

const fixtureRoot = "/w/tests/_fixtures/_git/elk";
const store = `${fixtureRoot}/node_modules/.pnpm`;
const vizeStore = "/w/node_modules/.pnpm";

/** The Vue runtime as elk's own lockfile pins it. */
const elkVue = `${store}/vue@3.5.30_typescript@5.9.3/node_modules/vue/dist/vue.d.mts`;
const elkRuntimeDom = `${store}/@vue+runtime-dom@3.5.30/node_modules/@vue/runtime-dom/dist/runtime-dom.d.ts`;
const elkRuntimeCore = `${store}/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core/dist/runtime-core.d.ts`;
/** The copies Vize's own `node_modules` answered with. */
const vizeVue = `${vizeStore}/vue@3.6.0-beta.10_typescript@6.0.3/node_modules/vue/dist/vue.d.mts`;
const vizeRuntimeDom = `${vizeStore}/@vue+runtime-dom@3.6.0-beta.10/node_modules/@vue/runtime-dom/dist/runtime-dom.d.ts`;
const vizeRouter = `${vizeStore}/vue-router@4.5.1_vue@3.6.0-beta.10_typescript@6.0.3_/node_modules/vue-router/dist/vue-router.d.ts`;
/** vue-tsc always loads these from its own installation, whatever the fixture is. */
const compilerLib = `${vizeStore}/typescript@6.0.3/node_modules/typescript/lib/lib.esnext.d.ts`;
const volarShim = `${vizeStore}/@vue+language-core@3.3.4/node_modules/@vue/language-core/index.d.ts`;

const isolatedProgram = [
  `${fixtureRoot}/app/app.vue`,
  `${fixtureRoot}/.nuxt/nuxt.d.ts`,
  elkVue,
  elkRuntimeDom,
  elkRuntimeCore,
  `${store}/vue-router@5.1.0_vue@3.5.30/node_modules/vue-router/dist/vue-router.d.ts`,
  compilerLib,
  volarShim,
].join("\n");

const contaminatedProgram = [isolatedProgram, vizeVue, vizeRuntimeDom, vizeRouter].join("\n");

test("a program with one fixture-owned copy of each Vue runtime package is isolated", () => {
  assert.deepEqual(evaluateBaselineAmbientEnvironment(isolatedProgram, fixtureRoot), {
    externalFileCount: 2,
    externalPackages: ["@vue/language-core", "typescript"],
    unusableReason: null,
    verdict: "isolated",
    vueRuntime: [
      {
        name: "@vue/runtime-core",
        copies: [
          {
            path: "node_modules/.pnpm/@vue+runtime-core@3.5.30/node_modules/@vue/runtime-core",
            insideFixture: true,
          },
        ],
        insideFixtureCount: 1,
        outsideFixtureCount: 0,
      },
      {
        name: "@vue/runtime-dom",
        copies: [
          {
            path: "node_modules/.pnpm/@vue+runtime-dom@3.5.30/node_modules/@vue/runtime-dom",
            insideFixture: true,
          },
        ],
        insideFixtureCount: 1,
        outsideFixtureCount: 0,
      },
      {
        name: "vue",
        copies: [
          {
            path: "node_modules/.pnpm/vue@3.5.30_typescript@5.9.3/node_modules/vue",
            insideFixture: true,
          },
        ],
        insideFixtureCount: 1,
        outsideFixtureCount: 0,
      },
    ],
  });
});

test("elk's run 31979524200 program is contaminated, and says which copies split it", () => {
  const ambient = evaluateBaselineAmbientEnvironment(contaminatedProgram, fixtureRoot);
  assert.equal(ambient.verdict, "contaminated");
  assert.equal(ambient.externalFileCount, 5);
  assert.deepEqual(ambient.externalPackages, [
    "@vue/language-core",
    "@vue/runtime-dom",
    "typescript",
    "vue",
    "vue-router",
  ]);
  // Reported against `@vue/runtime-dom` rather than `vue` only because the
  // failure is listed in package-name order; both are duplicated, and either one
  // alone is enough to split `declare module 'vue'`.
  assert.equal(
    ambient.unusableReason,
    "vue-tsc resolved 2 copies of '@vue/runtime-dom' into the baseline program " +
      "(../../../../node_modules/.pnpm/@vue+runtime-dom@3.6.0-beta.10/node_modules/@vue/runtime-dom, " +
      "node_modules/.pnpm/@vue+runtime-dom@3.5.30/node_modules/@vue/runtime-dom), so the fixture's " +
      "own module augmentations merged into a different module identity than its components",
  );
  assert.deepEqual(
    ambient.vueRuntime.map((entry) => [
      entry.name,
      entry.insideFixtureCount,
      entry.outsideFixtureCount,
    ]),
    [
      ["@vue/runtime-core", 1, 0],
      ["@vue/runtime-dom", 1, 1],
      ["vue", 1, 1],
    ],
  );
});

test("a single Vue runtime copy outside the fixture is unusable, not isolated", () => {
  // The fixture never installed `vue`, so the whole comparison ran against
  // Vize's. One copy, so the duplicate rule cannot see it.
  const program = [`${fixtureRoot}/app/app.vue`, vizeVue, compilerLib].join("\n");
  const ambient = evaluateBaselineAmbientEnvironment(program, fixtureRoot);
  assert.equal(ambient.verdict, "contaminated");
  assert.equal(
    ambient.unusableReason,
    "vue-tsc resolved 'vue' outside the fixture " +
      "(../../../../node_modules/.pnpm/vue@3.6.0-beta.10_typescript@6.0.3/node_modules/vue), so the " +
      "baseline measured a type environment the fixture does not own",
  );
});

test("diagnostic text and store paths are not read as program packages", () => {
  // `--listFilesOnly` output is absolute paths; anything else in the stream is a
  // message. A relative path, and a `.pnpm` entry with no package after it,
  // must not become evidence.
  const program = [
    `${fixtureRoot}/app/app.vue`,
    elkVue,
    compilerLib,
    "app/app.vue(1,1): error TS2339: Property '$t' does not exist on /w/node_modules/vue.",
    `${store}/`,
  ].join("\n");
  const ambient = evaluateBaselineAmbientEnvironment(program, fixtureRoot);
  assert.equal(ambient.verdict, "isolated");
  assert.deepEqual(ambient.externalPackages, ["typescript"]);
  assert.deepEqual(
    ambient.vueRuntime.map((entry) => entry.name),
    ["vue"],
  );
});

test("a program with no Vue runtime at all asserts nothing and records nothing", () => {
  const ambient = evaluateBaselineAmbientEnvironment(`${fixtureRoot}/src/App.vue`, fixtureRoot);
  assert.deepEqual(ambient, {
    externalFileCount: 0,
    externalPackages: [],
    unusableReason: null,
    verdict: "isolated",
    vueRuntime: [],
  });
});

test("non-string vue-tsc output is rejected rather than silently passing", () => {
  assert.throws(
    () => evaluateBaselineAmbientEnvironment(undefined, fixtureRoot),
    /vue-tsc output must be a string/,
  );
});
