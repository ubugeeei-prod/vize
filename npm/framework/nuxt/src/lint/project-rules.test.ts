import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, writeFile } from "node:fs/promises";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import type { TestContext } from "node:test";

import type { VizeRuleConfigPreset } from "oxlint-plugin-vize";

import { setupNuxtLintConfigGeneration } from "./generation.ts";
import { readProjectLintRules, type ProjectLintRules } from "./project-rules.ts";

async function temporaryRoot(t: TestContext, prefix: string): Promise<string> {
  const root = await mkdtemp(path.join(os.tmpdir(), prefix));
  t.after(() => rm(root, { force: true, recursive: true }));
  return root;
}

function createNuxt(rootDir: string) {
  return {
    options: {
      rootDir,
      srcDir: path.join(rootDir, "app"),
      buildDir: path.join(rootDir, ".nuxt"),
      dir: {},
      _layers: [{ config: { srcDir: path.join(rootDir, "app") } }],
    },
    hook() {},
  };
}

void test("a missing vize config adds no project rules", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-missing-");
  assert.equal(await readProjectLintRules(root, unexpectedPreset), undefined);
});

void test("an essential preset is expanded and explicit rules win", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-essential-");
  await writeFile(
    path.join(root, "vize.config.json"),
    JSON.stringify({
      linter: {
        preset: "essential",
        typeAware: true,
        rules: { "vue/require-v-for-key": "off", "vize/a11y/anchor-is-valid": "error" },
      },
    }),
  );
  const seen: Array<{ preset: VizeRuleConfigPreset; typeAware: boolean }> = [];

  const rules = await readProjectLintRules(root, async (preset, typeAware) => {
    seen.push({ preset, typeAware });
    return { "vue/require-v-for-key": "error", "vue/no-array-index-key": "error" };
  });

  assert.deepEqual(seen, [{ preset: "essential", typeAware: true }]);
  assert.deepEqual(rules, {
    "vue/no-array-index-key": "error",
    "vue/require-v-for-key": "off",
    "a11y/anchor-is-valid": "error",
  });
});

void test("incremental keeps explicit rules and does not load a preset", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-incremental-");
  await writeFile(
    path.join(root, "vize.config.json"),
    JSON.stringify({
      linter: { preset: "incremental", rules: { "vue/require-v-for-key": "error" } },
    }),
  );

  const rules = await readProjectLintRules(root, unexpectedPreset);
  assert.deepEqual(rules, { "vue/require-v-for-key": "error" });
});

void test("generated config lists project rules under the incremental preset", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-emit-");
  await mkdir(path.join(root, "app"));
  const projectRules: ProjectLintRules = { "vue/require-v-for-key": "error" };

  const generation = await setupNuxtLintConfigGeneration({ autoInit: false }, createNuxt(root), {
    resolvePluginSpecifier: () => "../plugin.mjs",
    resolveProjectLintRules: async () => projectRules,
  });
  const artifact = JSON.parse(await readFile(generation?.configFile ?? "", "utf8")) as {
    settings: { vize: { preset: string } };
    rules: Record<string, string>;
  };

  assert.equal(artifact.settings.vize.preset, "incremental");
  assert.equal(artifact.rules["vize/vue/require-v-for-key"], "error");
  assert.equal(artifact.rules["vize/nuxt/prefer-import-meta"], "error");
});

function unexpectedPreset(): Promise<ProjectLintRules> {
  throw new Error("preset loader should not run");
}
