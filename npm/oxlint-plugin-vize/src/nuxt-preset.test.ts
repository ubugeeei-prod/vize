import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint-plugin-vize/dist/index.mjs");
const fixtureDir = path.join(workspaceRoot, "target", "vize-tests", "oxlint-plugin-vize-nuxt-test");
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const optionsApiVuePath = path.join(fixtureDir, "OptionsApi.vue");
const ansiEscapePattern = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");
const { configs } = await import(pathToFileURL(pluginEntry).href);

assert.equal(configs.nuxt["vize/script/no-options-api"], undefined);
assert.equal(configs.opinionated["vize/script/no-options-api"], "error");

fs.rmSync(fixtureDir, { force: true, recursive: true });
fs.mkdirSync(fixtureDir, { recursive: true });

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
        "no-unused-vars": "off",
        "vize/script/no-options-api": "error",
      },
    },
    null,
    2,
  ),
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

const run = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "OptionsApi.vue"]);

assert.equal(run.exitCode, 0, "nuxt preset should allow Options API components");
assert.doesNotMatch(run.output, /vize\(script\/no-options-api\)/);

console.log("oxlint-plugin-vize Nuxt preset tests passed!");

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
  const env = { ...process.env };
  delete env.GITHUB_ACTIONS;

  try {
    const stdout = execFileSync(findOxlintBin(), args, {
      cwd: fixtureDir,
      encoding: "utf8",
      env,
      stdio: ["ignore", "pipe", "pipe"],
    });
    return { exitCode: 0, output: normalizeOutput(stdout) };
  } catch (error) {
    if (
      typeof error === "object" &&
      error !== null &&
      "status" in error &&
      "stdout" in error &&
      "stderr" in error
    ) {
      const processError = error as { status: number | null; stdout: string; stderr: string };
      return {
        exitCode: processError.status ?? 1,
        output: normalizeOutput(`${processError.stdout}${processError.stderr}`),
      };
    }

    throw error;
  }
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
