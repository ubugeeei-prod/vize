/** Full-artifact differential test for the Nuxt runtime emitter. */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  buildNuxtLintPlan,
  collectNuxtLintDirs,
  resolveNuxtLintFeatures,
  type NuxtLintProjectState,
} from "@vizejs/nuxt-lint-config";

import {
  setupNuxtLintConfigAddons,
  type NuxtLintConfigAddonNuxt,
  type NuxtLintImport,
} from "./addons.ts";
import {
  resolveNuxtLintCheckerOptions,
  type VizeNuxtLintCheckerOptions,
} from "./checker/options.ts";
import { renderNuxtOxlintConfig } from "./emitter.ts";

const compatDir = join(
  dirname(fileURLToPath(import.meta.url)),
  "..",
  "..",
  "..",
  "nuxt-lint-config",
  "test",
  "nuxt-eslint-compat",
);
const ROOT_DIR = "/project";
const RECORDED_VIZE_PLUGIN_SPECIFIER = "../node_modules/oxlint-plugin-vize/dist/index.mjs";

interface CorpusCase {
  id: string;
  project: Omit<NuxtLintProjectState, "rootDir">;
  config: Record<string, unknown>;
}

interface RecordedCase {
  oxlintConfig: string;
}

function readJson<T>(...segments: string[]): T {
  return JSON.parse(readFileSync(join(compatDir, ...segments), "utf8")) as T;
}

interface CheckerCase {
  id: string;
  description: string;
  checker: true | VizeNuxtLintCheckerOptions;
}

const corpus = readJson<{
  importGlobals: {
    id: string;
    description: string;
    nuxt: NuxtLintImport[];
    nitro: NuxtLintImport[];
  };
  checkerCases: CheckerCase[];
  cases: CorpusCase[];
}>("fixtures", "corpus.json");
const recording = readJson<{
  typeScriptDetected: boolean;
  importGlobals: {
    globals: string[];
    artifacts: { initial: string; regenerated: string };
  };
  checkerOptions: Record<string, Record<string, unknown>>;
  cases: Record<string, RecordedCase>;
}>("fixtures", "nuxt-eslint-output.json");

function createAddonNuxtStub(): NuxtLintConfigAddonNuxt & {
  callRegisteredHook(name: string, value: unknown): Promise<void>;
} {
  const hooks = new Map<string, Array<(value: unknown) => unknown>>();
  return {
    hook(name, callback) {
      hooks.set(name, [...(hooks.get(name) ?? []), callback]);
    },
    callHook() {},
    async callRegisteredHook(name, value) {
      for (const callback of hooks.get(name) ?? []) await callback(value);
    },
  };
}

function projectState(entry: CorpusCase): NuxtLintProjectState {
  return {
    rootDir: ROOT_DIR,
    dir: entry.project.dir,
    layers: entry.project.layers.map((layer) => ({
      ...layer,
      srcDir: join(ROOT_DIR, layer.srcDir),
    })),
  };
}

for (const entry of corpus.checkerCases) {
  void test(`${entry.id}: resolved checker options match @nuxt/eslint in full`, () => {
    const resolved = resolveNuxtLintCheckerOptions(entry.checker, {
      buildDir: "/project/.nuxt",
      srcDir: "/project/app",
    });
    assert.notEqual(resolved, false);
    assert.deepEqual({ ...resolved, worker: true }, recording.checkerOptions[entry.id]);
  });
}

void test(`${corpus.importGlobals.id}: globals and artifacts match @nuxt/eslint byte for byte`, async () => {
  const nuxt = createAddonNuxtStub();
  const resolveAddons = setupNuxtLintConfigAddons(nuxt);
  await nuxt.callRegisteredHook("imports:context", {
    getImports: async () => corpus.importGlobals.nuxt,
  });
  await nuxt.callRegisteredHook("nitro:init", {
    unimport: { getImports: async () => corpus.importGlobals.nitro },
  });

  const addons = await resolveAddons();
  const globals = addons[0]?.globals ?? {};
  assert.deepEqual(Object.keys(globals), recording.importGlobals.globals);
  assert.deepEqual(new Set(Object.values(globals)), new Set(["readonly"]));
  for (const hostileName of ["__proto__", "constructor", "toString"]) {
    assert.equal(Object.hasOwn(globals, hostileName), true);
  }

  const entry = corpus.cases[0];
  assert.ok(entry, "the import-globals oracle requires a base project case");
  const features = resolveNuxtLintFeatures(entry.config, () => recording.typeScriptDetected);
  const plan = buildNuxtLintPlan(features, collectNuxtLintDirs(projectState(entry)));
  assert.equal(
    renderNuxtOxlintConfig(plan, RECORDED_VIZE_PLUGIN_SPECIFIER),
    recording.importGlobals.artifacts.initial,
  );
  assert.equal(
    renderNuxtOxlintConfig([...plan, ...addons], RECORDED_VIZE_PLUGIN_SPECIFIER),
    recording.importGlobals.artifacts.regenerated,
  );
});

for (const entry of corpus.cases) {
  void test(`${entry.id}: whole generated oxlint artifact matches upstream`, () => {
    const recorded = recording.cases[entry.id];
    assert.ok(recorded, `${entry.id} must have a recording`);

    const features = resolveNuxtLintFeatures(entry.config, () => recording.typeScriptDetected);
    const plan = buildNuxtLintPlan(features, collectNuxtLintDirs(projectState(entry)));

    assert.equal(
      renderNuxtOxlintConfig(plan, RECORDED_VIZE_PLUGIN_SPECIFIER),
      recorded.oxlintConfig,
    );
  });
}
