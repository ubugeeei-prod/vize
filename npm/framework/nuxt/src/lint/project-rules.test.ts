import assert from "node:assert/strict";
import { mkdir, mkdtemp, readFile, rm, symlink, writeFile } from "node:fs/promises";
import { createRequire } from "node:module";
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

void test("a project without configuration adds no project rules", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-missing-");
  assert.equal(await readProjectLintRules(root, unexpectedPreset), undefined);
});

void test("Vite shared lint settings work without a dedicated Vize config", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-vite-");
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `
    export default { vue: { linter: { preset: "essential", typeAware: true,
      rules: { "vize/vue/require-v-for-key": "off" }
    } } };
  `,
  );
  const seen: Array<{ preset: VizeRuleConfigPreset; typeAware: boolean }> = [];
  const rules = await readProjectLintRules(root, async (preset, typeAware) => {
    seen.push({ preset, typeAware });
    return { "vue/require-v-for-key": "error", "vue/no-array-index-key": "error" };
  });
  assert.deepEqual(seen, [{ preset: "essential", typeAware: true }]);
  assert.deepEqual(rules, { "vue/require-v-for-key": "off", "vue/no-array-index-key": "error" });
});

void test("Nuxt Vite factories receive serve and retired files cannot override them", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-config-command-");
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `export default ({ command, mode }) => {
      if (command !== "serve" || mode !== "development") throw new Error("invalid Vite env");
      return { vue: { linter: { rules: { "vue/require-v-for-key": "error" } } } };
    };`,
  );
  assert.deepEqual(await readProjectLintRules(root, unexpectedPreset), {
    "vue/require-v-for-key": "error",
  });
  await writeFile(
    path.join(root, "vize.config.mjs"),
    `export default ({ command, mode }) => {
      if (command !== "lint" || mode !== "development") throw new Error("invalid Vize env");
      return { linter: { rules: { "vue/require-v-for-key": "warn" } } };
    };`,
  );
  assert.deepEqual(await readProjectLintRules(root, unexpectedPreset), {
    "vue/require-v-for-key": "error",
  });
  await rm(path.join(root, "vite.config.mjs"));
  await assert.rejects(
    readProjectLintRules(root, unexpectedPreset),
    /vize\.config\.\* is no longer supported/,
  );
});

void test("Nuxt reads nearest Vue settings and refuses retired config fallback in a monorepo", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-monorepo-");
  const app = path.join(root, "packages/app");
  await mkdir(app, { recursive: true });
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `
    export default { vue: { linter: { rules: { "vue/require-v-for-key": "error" } } } };
  `,
  );
  assert.deepEqual(await readProjectLintRules(app, unexpectedPreset), {
    "vue/require-v-for-key": "error",
  });
  await writeFile(
    path.join(app, "vite.config.mjs"),
    `
    export default { vue: { linter: { rules: { "vue/require-v-for-key": "warn" } } } };
  `,
  );
  assert.deepEqual(await readProjectLintRules(app, unexpectedPreset), {
    "vue/require-v-for-key": "warn",
  });
  await writeFile(
    path.join(app, "vize.config.json"),
    JSON.stringify({
      linter: { rules: { "vue/require-v-for-key": "off" } },
    }),
  );
  assert.deepEqual(await readProjectLintRules(app, unexpectedPreset), {
    "vue/require-v-for-key": "warn",
  });
  await rm(path.join(app, "vite.config.mjs"));
  await assert.rejects(
    readProjectLintRules(app, unexpectedPreset),
    /vize\.config\.\* is no longer supported/,
  );
});

void test("an essential preset is expanded and explicit rules win", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-project-rules-essential-");
  await writeFile(
    path.join(root, "vite.config.mjs"),
    `export default ${JSON.stringify({
      vue: {
        linter: {
          preset: "essential",
          typeAware: true,
          rules: { "vue/require-v-for-key": "off", "vize/a11y/anchor-is-valid": "error" },
        },
      },
    })};`,
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
    path.join(root, "vite.config.mjs"),
    `export default ${JSON.stringify({
      vue: {
        linter: { preset: "incremental", rules: { "vue/require-v-for-key": "error" } },
      },
    })};`,
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

await test("Nuxt's public config consumer reads canonical helpers and rejects unchanged historical config", async (t) => {
  const root = await temporaryRoot(t, "vize-nuxt-original-vite-module-");
  const original = JSON.parse(
    await readFile(
      new URL(
        "../../../../../tests/_fixtures/differential/config/public-vite-module-8371/project.json",
        import.meta.url,
      ),
      "utf8",
    ),
  ) as { files: Record<string, string>; rules: ProjectLintRules };
  const require = createRequire(import.meta.url);
  assert.equal(path.basename(require.resolve("vize/config")), "config.mjs");
  await mkdir(path.join(root, "node_modules"));
  await symlink(
    path.dirname(require.resolve("vite/package.json")),
    path.join(root, "node_modules/vite"),
    "junction",
  );
  const current = {
    ...original.files,
    "vite.config.ts": "import { settings } from './settings';\nexport default { vue: settings };\n",
  };
  for (const [name, source] of Object.entries(current))
    await writeFile(path.join(root, name), source);
  assert.deepEqual(await readProjectLintRules(root, unexpectedPreset), original.rules);
  for (const [name, source] of Object.entries(current)) {
    assert.equal(await readFile(path.join(root, name), "utf8"), source);
  }
  const historical = await temporaryRoot(t, "vize-nuxt-historical-vite-module-");
  for (const [name, source] of Object.entries(original.files))
    await writeFile(path.join(historical, name), source);
  await mkdir(path.join(historical, "node_modules"));
  await symlink(
    path.dirname(require.resolve("vite/package.json")),
    path.join(historical, "node_modules/vite"),
    "junction",
  );
  await assert.rejects(
    readProjectLintRules(historical, unexpectedPreset),
    /vize: configuration is no longer supported/,
  );
  for (const [name, source] of Object.entries(original.files)) {
    assert.equal(await readFile(path.join(historical, name), "utf8"), source);
  }
});
