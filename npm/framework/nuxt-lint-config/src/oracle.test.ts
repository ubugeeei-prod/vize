/**
 * Differential test against the recorded `@nuxt/eslint` ground truth.
 *
 * The recording in `test/nuxt-eslint-compat/fixtures/` is produced from the
 * real packages by `oracle.mjs`; this suite reads it with no `@nuxt/*` import
 * of its own, so the package's tests stay offline. CI re-derives the recording
 * in `tests/tooling/nuxt-eslint-oracle.test.ts`, so an upstream bump surfaces
 * there as a hard failure rather than as silent compatibility drift.
 */
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { dirname, join } from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

import {
  collectNuxtLintDirs,
  resolveNuxtLintDirs,
  type NuxtLintDirs,
  type NuxtLintProjectState,
  buildNuxtLintPlan,
  resolveNuxtLintFeatures,
  type NuxtLintConfigItem,
} from "./index.ts";

const compatDir = join(dirname(fileURLToPath(import.meta.url)), "..", "test", "nuxt-eslint-compat");

interface CorpusCase {
  id: string;
  category: string;
  description: string;
  project: Omit<NuxtLintProjectState, "rootDir">;
  config: Record<string, unknown>;
}

interface RecordedItem {
  name: string | null;
  files: string[] | null;
  ignores: string[] | null;
  rules: Record<string, string> | null;
  globals: Record<string, string> | null;
}

interface RecordedCase {
  dirs: Record<string, string[]>;
  features: Record<string, unknown>;
  configNames: Array<string | null>;
  nuxtConfigs: RecordedItem[];
}

function readJson<T>(...segments: string[]): T {
  return JSON.parse(readFileSync(join(compatDir, ...segments), "utf8")) as T;
}

interface DirDefaultCase {
  id: string;
  description: string;
  dirs: Partial<NuxtLintDirs>;
}

interface RecordedPageMetaMessage {
  ruleId: string;
  severity: number;
  message: string;
  line: number;
  column: number;
  endLine: number;
  endColumn: number;
  range: [number, number];
  fix: null;
}

const corpus = readJson<{
  oracle: Record<string, string>;
  preferImportMetaCases: Array<{ id: string; description: string; source: string }>;
  noPageMetaRuntimeValuesCases: Array<{ id: string; description: string; source: string }>;
  dirDefaultCases: DirDefaultCase[];
  cases: CorpusCase[];
}>("fixtures", "corpus.json");
const recording = readJson<{
  schemaVersion: number;
  moduleVersion: string;
  configVersion: string;
  pluginVersion: string;
  typeScriptDetected: boolean;
  preferImportMetaCases: Record<
    string,
    {
      messages: Array<{ ruleId: string; severity: number; message: string }>;
      output: string;
      fixed: boolean;
      secondPassMessages: unknown[];
      secondPassOutput: string;
      secondPassFixed: boolean;
    }
  >;
  noPageMetaRuntimeValuesCases: Record<
    string,
    {
      messages: RecordedPageMetaMessage[];
      output: string;
      fixed: boolean;
      secondPassMessageCount: number;
      secondPassMessagesMatch: boolean;
      secondPassOutput: string;
      secondPassFixed: boolean;
    }
  >;
  dirDefaults: Record<string, NuxtLintDirs>;
  cases: Record<string, RecordedCase>;
}>("fixtures", "nuxt-eslint-output.json");

/**
 * The corpus states layer source directories relative to the project root, the
 * way a `nuxt.config` does. Nuxt itself hands the module absolute paths, so the
 * oracle joins them onto its scratch root and this rebuild does the same
 * against a fixed synthetic root — which is what keeps the recording free of
 * machine-specific paths.
 */
const ROOT_DIR = "/project";

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

/**
 * Upstream leaves its ignore block unnamed; Vize gives every block a stable
 * identity so the emitter can address it. That rename is the one intentional
 * difference between the plan and the recording, so it is applied here in one
 * place rather than being absorbed into a looser comparison.
 */
function recordedItemName(item: RecordedItem): string {
  return item.name ?? "nuxt/ignores";
}

/** Render a plan item in the recording's shape so the two compare directly. */
function planItemAsRecorded(item: NuxtLintConfigItem) {
  return {
    name: item.name,
    files: item.files ?? null,
    ignores: item.ignores ?? null,
    rules: item.rules ?? null,
    globals: item.globals ?? null,
  };
}

void test("corpus pins the package versions the recording was produced with", () => {
  assert.equal(corpus.oracle.module, "@nuxt/eslint");
  assert.equal(corpus.oracle.config, "@nuxt/eslint-config");
  assert.equal(corpus.oracle.moduleVersion, recording.moduleVersion);
  assert.equal(corpus.oracle.configVersion, recording.configVersion);
  assert.equal(corpus.oracle.plugin, "@nuxt/eslint-plugin");
  assert.equal(corpus.oracle.pluginVersion, recording.pluginVersion);
});

void test("recording covers exactly the corpus cases", () => {
  assert.equal(recording.schemaVersion, 4);
  assert.deepEqual(
    Object.keys(recording.cases).sort(),
    corpus.cases.map((entry) => entry.id).sort(),
  );
  assert.deepEqual(
    Object.keys(recording.dirDefaults).sort(),
    corpus.dirDefaultCases.map((entry) => entry.id).sort(),
  );
  assert.deepEqual(
    Object.keys(recording.preferImportMetaCases).sort(),
    corpus.preferImportMetaCases.map((entry) => entry.id).sort(),
  );
});

for (const entry of corpus.preferImportMetaCases) {
  void test(`${entry.id}: recorded fix converges after one application`, () => {
    const recorded = recording.preferImportMetaCases[entry.id];
    assert.ok(recorded);
    assert.equal(recorded.fixed, recorded.messages.length > 0);
    assert.equal(recorded.secondPassOutput, recorded.output);
    assert.equal(recorded.secondPassFixed, false);
    assert.deepEqual(recorded.secondPassMessages, []);
    for (const message of recorded.messages) {
      assert.equal(message.ruleId, "nuxt/prefer-import-meta");
      assert.equal(message.severity, 2);
    }
  });
}

void test("recording covers exactly the page-meta rule corpus", () => {
  assert.deepEqual(
    Object.keys(recording.noPageMetaRuntimeValuesCases).sort(),
    corpus.noPageMetaRuntimeValuesCases.map((entry) => entry.id).sort(),
  );
});

for (const entry of corpus.noPageMetaRuntimeValuesCases) {
  void test(`${entry.id}: recorded non-fixable diagnostics are stable`, () => {
    const recorded = recording.noPageMetaRuntimeValuesCases[entry.id];
    assert.ok(recorded);
    assert.equal(recorded.fixed, false);
    assert.equal(recorded.output, entry.source);
    assert.equal(recorded.secondPassFixed, false);
    assert.equal(recorded.secondPassOutput, entry.source);
    assert.equal(recorded.secondPassMessageCount, recorded.messages.length);
    assert.equal(recorded.secondPassMessagesMatch, true);
    for (const message of recorded.messages) {
      assert.equal(message.ruleId, "nuxt/no-page-meta-runtime-values");
      assert.equal(message.severity, 2);
      assert.equal(message.fix, null);
      assert.ok(message.range[0] < message.range[1]);
    }
  });
}

for (const entry of corpus.dirDefaultCases) {
  void test(`${entry.id}: directory defaults match @nuxt/eslint-config`, () => {
    assert.deepEqual(resolveNuxtLintDirs(entry.dirs), recording.dirDefaults[entry.id]);
  });
}

for (const entry of corpus.cases) {
  const recorded = recording.cases[entry.id];

  void test(`${entry.id}: directories match @nuxt/eslint`, () => {
    const dirs = collectNuxtLintDirs(projectState(entry));
    assert.deepEqual(dirs, recorded.dirs);
  });

  void test(`${entry.id}: features match @nuxt/eslint-config`, () => {
    const features = resolveNuxtLintFeatures(entry.config, () => recording.typeScriptDetected);
    assert.deepEqual(features, recorded.features);
  });

  void test(`${entry.id}: config plan matches @nuxt/eslint-config`, () => {
    const features = resolveNuxtLintFeatures(entry.config, () => recording.typeScriptDetected);
    const plan = buildNuxtLintPlan(features, collectNuxtLintDirs(projectState(entry)));

    // Compare the whole ordered list in one assertion: order decides which
    // rule wins, so an item appearing in the wrong place is a real defect.
    assert.deepEqual(
      plan.map(planItemAsRecorded),
      recorded.nuxtConfigs.map((item) => ({ ...item, name: recordedItemName(item) })),
    );
  });
}
