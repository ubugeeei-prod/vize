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

const corpus = readJson<{ cases: CorpusCase[] }>("fixtures", "corpus.json");
const recording = readJson<{
  typeScriptDetected: boolean;
  cases: Record<string, RecordedCase>;
}>("fixtures", "nuxt-eslint-output.json");

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
