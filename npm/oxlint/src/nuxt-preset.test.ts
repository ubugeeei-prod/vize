import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import { resetFixtureDir } from "./test-support/fixture-dir.ts";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint/dist/index.mjs");
const cliEntry = path.join(workspaceRoot, "npm/oxlint/dist/cli.mjs");
const fixtureDir = path.join(workspaceRoot, "target", "vize-tests", "oxlint-plugin-vize-nuxt-test");
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const optionsApiVuePath = path.join(fixtureDir, "OptionsApi.vue");
const processFlagsVuePath = path.join(fixtureDir, "ProcessFlags.vue");
const pageMetaVuePath = path.join(fixtureDir, "PageMeta.vue");
const internalLinkVuePath = path.join(fixtureDir, "InternalLink.vue");
const nuxtConfigPath = path.join(fixtureDir, "nuxt.config.ts");
const ansiEscapePattern = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");
const { configs } = await import(pathToFileURL(pluginEntry).href);

assert.equal(configs.nuxt["vize/script/no-options-api"], undefined);
assert.equal(configs.opinionated["vize/script/no-options-api"], "error");
assert.equal(configs.nuxt["vize/nuxt/prefer-import-meta"], "error");
assert.equal(configs.opinionated["vize/nuxt/prefer-import-meta"], undefined);
assert.equal(configs.nuxt["vize/nuxt/no-page-meta-runtime-values"], "error");
assert.equal(configs.opinionated["vize/nuxt/no-page-meta-runtime-values"], undefined);
assert.equal(configs.nuxt["vize/nuxt/no-nuxt-config-test-key"], "error");
assert.equal(configs.opinionated["vize/nuxt/no-nuxt-config-test-key"], undefined);
assert.equal(configs.nuxt["vize/nuxt/nuxt-config-keys-order"], "error");
assert.equal(configs.opinionated["vize/nuxt/nuxt-config-keys-order"], undefined);
assert.equal(configs.nuxt["vize/ecosystem/nuxt-prefer-nuxt-link"], "warn");
assert.equal(configs.ecosystem["vize/ecosystem/nuxt-prefer-nuxt-link"], undefined);

resetFixtureDir(fixtureDir);

fs.writeFileSync(
  configPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        vize: {
          helpLevel: "none",
          preset: "nuxt",
        },
      },
      rules: {
        ...configs.nuxt,
        "no-unused-vars": "off",
        "vize/script/no-options-api": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  pageMetaVuePath,
  `<script setup lang="ts">
definePageMeta({ title: useRoute().path })
</script>
`,
);

fs.writeFileSync(
  internalLinkVuePath,
  `<template>
  <a href="/settings">Settings</a>
</template>
`,
);

fs.writeFileSync(
  nuxtConfigPath,
  `export default defineNuxtConfig({
  ssr: true,
  modules: [],
  test: true,
})
`,
);

fs.writeFileSync(
  optionsApiVuePath,
  `<script lang="ts">
import { defineComponent } from 'vue'

export default defineComponent({
  name: 'AppLoader',
  props: {
    active: Boolean
  }
})
</script>
<template>
  <div>{{ active }}</div>
</template>
`,
);

fs.writeFileSync(
  processFlagsVuePath,
  `<script setup lang="ts">
const enabled = process.client
</script>
`,
);

const run = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "OptionsApi.vue"]);

assert.equal(run.exitCode, 0, "nuxt preset should allow Options API components");
assert.doesNotMatch(run.output, /vize\(script\/no-options-api\)/);

const processFlagsRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "ProcessFlags.vue"]);
assert.notEqual(processFlagsRun.exitCode, 0, "nuxt preset should reject legacy process flags");
assert.match(
  processFlagsRun.output,
  /ProcessFlags\.vue[\s\S]*2:17[\s\S]*Replace `process\.client` with `import\.meta\.client`\.[\s\S]*vize\(nuxt\/prefer-import-meta\)/u,
);

const pageMetaRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "PageMeta.vue"]);
assert.notEqual(pageMetaRun.exitCode, 0, "nuxt preset should reject eager runtime page meta");
assert.match(
  pageMetaRun.output,
  /PageMeta\.vue[\s\S]*2:25[\s\S]*`useRoute\(\)` requires a Nuxt\/Vue runtime context[\s\S]*vize\(nuxt\/no-page-meta-runtime-values\)/u,
);

// `InternalLink.vue` has no script block, so it has to go through the
// `oxlint-vize` CLI: plain Oxlint never hands scriptless SFCs to JS plugins.
const internalLinkRun = runOxlintVize([
  "-c",
  ".oxlintrc.json",
  "-f",
  "stylish",
  "InternalLink.vue",
]);
assert.equal(internalLinkRun.exitCode, 0, "NuxtLink preference is a warning by default");
assert.match(
  internalLinkRun.output,
  /InternalLink\.vue[\s\S]*2:6[\s\S]*Use NuxtLink for internal links[\s\S]*vize\(ecosystem\/nuxt-prefer-nuxt-link\)/u,
);

const nuxtConfigRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "nuxt.config.ts"]);
assert.notEqual(nuxtConfigRun.exitCode, 0, "nuxt preset should reject the config test key");
assert.match(
  nuxtConfigRun.output,
  /nuxt\.config\.ts[\s\S]*4:3[\s\S]*Do not set `test` key in Nuxt config[\s\S]*vize\(nuxt\/no-nuxt-config-test-key\)/u,
);
assert.match(
  nuxtConfigRun.output,
  /nuxt\.config\.ts[\s\S]*2:3[\s\S]*Expected config key "ssr" to come after "modules"[\s\S]*vize\(nuxt\/nuxt-config-keys-order\)/u,
);

fs.writeFileSync(
  nuxtConfigPath,
  `export default defineNuxtConfig({
  plugins: [],
  buildModules: [],
  modules: [],
  vize: { compatibility: { nuxtVersion: 2 } },
})
`,
);
const nuxtTwoConfigRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "nuxt.config.ts"]);
assert.equal(
  nuxtTwoConfigRun.exitCode,
  0,
  `nuxt preset should not enforce Nuxt 3 ordering in Nuxt 2 compatibility mode:\n${nuxtTwoConfigRun.output}`,
);
assert.doesNotMatch(nuxtTwoConfigRun.output, /vize\(nuxt\/nuxt-config-keys-order\)/u);

console.log("oxlint-plugin-vize Nuxt preset tests passed!");
await import("./type-aware.test.ts");

function findOxlintBin() {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const candidates = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .sort((left, right) => right.localeCompare(left))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .filter((entry) => fs.existsSync(entry));

  const match = candidates[0];
  if (match == null) {
    throw new Error(`Unable to locate the oxlint binary in ${pnpmStoreDir}`);
  }

  return match;
}

function runOxlint(args: string[]) {
  return runCommand(findOxlintBin(), args);
}

function runOxlintVize(args: string[]) {
  return runCommand(process.execPath, [cliEntry, ...args]);
}

function runCommand(executable: string, args: string[]) {
  const env = { ...process.env };
  delete env.GITHUB_ACTIONS;

  const result = spawnSync(executable, args, {
    cwd: fixtureDir,
    encoding: "utf8",
    env,
    stdio: ["ignore", "pipe", "pipe"],
  });
  if (result.error) {
    throw result.error;
  }

  return {
    exitCode: result.status ?? 1,
    output: normalizeOutput(`${result.stdout}${result.stderr}`),
  };
}

function normalizeOutput(output: string): string {
  return output
    .replace(ansiEscapePattern, "")
    .replace(new RegExp(escapeRegExp(workspaceRoot), "gu"), "<workspaceRoot>")
    .replace(/^WARNING: JS plugins are experimental and not subject to semver\.\n/gmu, "")
    .replace(
      /^Breaking changes are possible while JS plugins support is under development\.\n/gmu,
      "",
    )
    .trim();
}

function escapeRegExp(value: string): string {
  return value.replaceAll(/[.*+?^${}()|[\]\\]/gu, String.raw`\$&`);
}
