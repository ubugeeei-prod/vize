import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const oxlintBin = path.join(workspaceRoot, "node_modules/.bin/oxlint");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint-plugin-patina/dist/index.js");
const fixtureDir = path.join(workspaceRoot, "__agent_only", "oxlint-plugin-patina-test");
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const noHelpConfigPath = path.join(fixtureDir, ".oxlintrc.no-help.json");
const vuePath = path.join(fixtureDir, "App.vue");
const snapshotsDir = path.join(packageDir, "__snapshots__");
const ansiEscapePattern = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");

fs.rmSync(fixtureDir, { force: true, recursive: true });
fs.mkdirSync(fixtureDir, { recursive: true });

fs.writeFileSync(
  configPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      rules: {
        "no-unused-vars": "off",
        "patina/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  noHelpConfigPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      settings: {
        patina: {
          showHelp: false,
        },
      },
      rules: {
        "no-unused-vars": "off",
        "patina/vue/require-v-for-key": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  vuePath,
  `<script setup lang="ts">
const items = [1]
</script>
<template>
  <ul>
    <li v-for="item in items">{{ item }}</li>
  </ul>
</template>
`,
);

function runOxlint(args: readonly string[]) {
  let output = "";
  let exitCode = 0;

  try {
    execFileSync(oxlintBin, args, {
      cwd: fixtureDir,
      encoding: "utf8",
      stdio: "pipe",
    });
  } catch (error) {
    const execError = error as {
      status?: number;
      stdout?: string | Buffer;
      stderr?: string | Buffer;
    };
    exitCode = execError.status ?? 1;
    output = String(execError.stdout ?? "") + String(execError.stderr ?? "");
  }

  return {
    exitCode,
    output: normalizeOutput(output),
  };
}

function normalizeOutput(output: string): string {
  return output
    .replace(ansiEscapePattern, "")
    .replace(/^WARNING: JS plugins are experimental and not subject to semver\.\n/gmu, "")
    .replace(
      /^Breaking changes are possible while JS plugins support is under development\.\n/gmu,
      "",
    )
    .replace(/^Finished in .*$/gmu, "")
    .trim();
}

function readSnapshot(name: string): string {
  return fs.readFileSync(path.join(snapshotsDir, name), "utf8").trim();
}

const defaultRun = runOxlint(["-c", ".oxlintrc.json", "App.vue"]);
assert.notEqual(defaultRun.exitCode, 0, "oxlint should fail when Patina reports an error");
assert.match(
  defaultRun.output,
  /patina\(vue\/require-v-for-key\)/,
  "Patina rule name should be surfaced",
);
assert.match(
  defaultRun.output,
  /Elements in iteration expect to have 'v-bind\[:\]key' directives\.[\s\S]*Location:[\s\S]*Vue template line 6, column 9/u,
  "Template diagnostics should expose the real Vue location in the message",
);
assert.doesNotMatch(
  defaultRun.output,
  /^  \| Source:/mu,
  "The inline source snippet should be gone",
);
assert.match(defaultRun.output, /^  \| Help:/mu, "Default output should still include help text");

const stylishRun = runOxlint(["-c", ".oxlintrc.no-help.json", "-f", "stylish", "App.vue"]);
assert.notEqual(stylishRun.exitCode, 0, "stylish formatter should still report Patina failures");
assert.match(
  stylishRun.output,
  /App\.vue[\s\S]*2:1[\s\S]*Elements in iteration expect to have 'v-bind\[:\]key' directives\.[\s\S]*^    Details:$[\s\S]*^      Elements in iteration expect to have 'v-bind\[:\]key' directives\. Element\[:\] <li>$[\s\S]*^    Location:$[\s\S]*^      Vue template line 6, column 9  patina\(vue\/require-v-for-key\)$/mu,
  "Stylish formatter should keep the output concise while surfacing the real Vue location",
);
assert.doesNotMatch(stylishRun.output, /^Help:/mu, "showHelp: false should omit the help section");
assert.doesNotMatch(
  stylishRun.output,
  /^Source:/mu,
  "Stylish formatter should avoid the old inline source block",
);
assert.equal(stylishRun.output, readSnapshot("stylish-no-help-output.txt"));

console.log("✅ oxlint-plugin-patina integration tests passed!");
