/**
 * Script-block diagnostic location contract for the Oxlint bridge.
 *
 * Oxlint hands JS plugins the *extracted program*, not the `.vue` file, so the
 * bridge has to translate Patina's authored SFC line/column back through the
 * script block's own offset. When that translation silently fails, every
 * `vize/*` diagnostic collapses onto the script block's first line: editors and
 * CI annotations read `line`/`column` rather than the `(at <script setup>:L:C)`
 * message suffix, so they send the reader to the wrong place and two findings on
 * different lines become indistinguishable.
 *
 * The fixture is built so neither half of the translation can regress silently:
 * the script block starts on line 5 rather than line 1, so a hardcoded
 * line-1 anchor fails, and it carries diagnostics on two different lines, so
 * collapsing them onto one position fails.
 */
import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { resetFixtureDir } from "./test-support/fixture-dir.ts";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint/dist/index.mjs");
const fixtureDir = path.join(
  workspaceRoot,
  "target",
  "vize-tests",
  "oxlint-plugin-vize-script-location-test",
);
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const scriptOffsetVuePath = path.join(fixtureDir, "ScriptOffset.vue");
const dualScriptVuePath = path.join(fixtureDir, "DualScriptLocations.vue");
const genericSetupVuePath = path.join(fixtureDir, "GenericSetup.vue");
const snapshotPath = path.join(packageDir, "__snapshots__", "stylish-script-offset-output.txt");
const ansiEscapePattern = new RegExp(String.raw`\[[0-9;]*m`, "gu");
const workspaceRootPattern = new RegExp(escapeRegExp(workspaceRoot), "gu");

function escapeRegExp(value: string): string {
  return value.replaceAll(/[.*+?^${}()|[\]\\]/gu, String.raw`\$&`);
}

function findOxlintBin(): string {
  const pnpmStoreDir = path.join(workspaceRoot, "node_modules", ".pnpm");
  const candidates = fs
    .readdirSync(pnpmStoreDir)
    .filter((entry) => entry.startsWith("oxlint@"))
    .map((entry) => path.join(pnpmStoreDir, entry, "node_modules", "oxlint", "bin", "oxlint"))
    .filter((candidate) => fs.existsSync(candidate));
  assert.ok(candidates.length > 0, "oxlint binary must be installed in the workspace");
  return candidates[candidates.length - 1];
}

const oxlintBin = findOxlintBin();
const oxlintEnv = { ...process.env };
delete oxlintEnv.GITHUB_ACTIONS;

function normalizeOutput(output: string): string {
  return output
    .replace(ansiEscapePattern, "")
    .replace(workspaceRootPattern, "<workspaceRoot>")
    .replace(/^WARNING: JS plugins are experimental and not subject to semver\.\n/gmu, "")
    .replace(
      /^Breaking changes are possible while JS plugins support is under development\.\n/gmu,
      "",
    )
    .replace(/^Finished in .*$/gmu, "")
    .trim();
}

function runOxlint(args: readonly string[]): { exitCode: number; output: string } {
  try {
    const output = String(
      execFileSync(oxlintBin, args, {
        cwd: fixtureDir,
        encoding: "utf8",
        env: oxlintEnv,
        stdio: "pipe",
      }),
    );
    return { exitCode: 0, output: normalizeOutput(output) };
  } catch (error) {
    const execError = error as {
      status?: number;
      stdout?: string | Buffer;
      stderr?: string | Buffer;
    };
    return {
      exitCode: execError.status ?? 1,
      output: normalizeOutput(String(execError.stdout ?? "") + String(execError.stderr ?? "")),
    };
  }
}

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
          preset: "opinionated",
        },
      },
      rules: {
        "no-unused-vars": "off",
        "vize/script/no-get-current-instance": "error",
      },
    },
    null,
    2,
  ),
);

fs.writeFileSync(
  scriptOffsetVuePath,
  `<template>
  <div>{{ instance }}</div>
</template>

<script setup lang="ts">
import { getCurrentInstance } from 'vue'

const instance = getCurrentInstance()
</script>
`,
);

fs.writeFileSync(
  dualScriptVuePath,
  `<template>
  <div>{{ instance }} {{ setupInstance }}</div>
</template>

<script lang="ts">
const instance = getCurrentInstance()
</script>
<script setup lang="ts">
const setupInstance = getCurrentInstance()
</script>
`,
);

fs.writeFileSync(
  genericSetupVuePath,
  `<template>
  <div>{{ instance }}</div>
</template>

<script setup lang="ts" generic="T extends Record<string, unknown>">
const instance = getCurrentInstance()
</script>
`,
);

const scriptOffsetRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "ScriptOffset.vue"]);
assert.notEqual(
  scriptOffsetRun.exitCode,
  0,
  "script-offset fixture should report opinionated script diagnostics",
);

// Patina reports these at SFC 6:10 and 8:18. Asserting the positions is the only
// thing that catches a bridge which silently collapses every diagnostic onto one
// line, because the message text is identical either way.
assert.match(
  scriptOffsetRun.output,
  /^  6:10 {2}error {2}getCurrentInstance import is not supported in Vapor-oriented components {2}vize\(script\/no-get-current-instance\)$/mu,
  "a diagnostic on the script block's first content line should keep its real SFC position",
);
assert.match(
  scriptOffsetRun.output,
  /^  8:18 {2}error {2}getCurrentInstance\(\) is not supported in Vapor-oriented components {2}vize\(script\/no-get-current-instance\)$/mu,
  "a diagnostic further into the script block should keep its real SFC position",
);
assert.doesNotMatch(
  scriptOffsetRun.output,
  /\(at <script setup>:/u,
  "mapped script diagnostics should not need the fallback location suffix",
);
assert.equal(scriptOffsetRun.output, normalizeOutput(fs.readFileSync(snapshotPath, "utf8")));

const dualScriptRun = runOxlint([
  "-c",
  ".oxlintrc.json",
  "-f",
  "stylish",
  "DualScriptLocations.vue",
]);
assert.notEqual(
  dualScriptRun.exitCode,
  0,
  "dual-script fixture should report diagnostics from both script blocks",
);
assert.match(
  dualScriptRun.output,
  /^  6:18 {2}error {2}getCurrentInstance\(\) is not supported in Vapor-oriented components {2}vize\(script\/no-get-current-instance\)$/mu,
  "a normal <script> diagnostic should map through its own extracted program",
);
assert.match(
  dualScriptRun.output,
  /^  9:23 {2}error {2}getCurrentInstance\(\) is not supported in Vapor-oriented components {2}vize\(script\/no-get-current-instance\)$/mu,
  "a <script setup> diagnostic should map through its own extracted program",
);
assert.doesNotMatch(
  dualScriptRun.output,
  /\(at <script/u,
  "dual-script diagnostics should not fall back to block-label suffixes",
);

const genericSetupRun = runOxlint(["-c", ".oxlintrc.json", "-f", "stylish", "GenericSetup.vue"]);
assert.notEqual(
  genericSetupRun.exitCode,
  0,
  "generic script setup fixture should report script diagnostics",
);
assert.match(
  genericSetupRun.output,
  /^  6:18 {2}error {2}getCurrentInstance\(\) is not supported in Vapor-oriented components {2}vize\(script\/no-get-current-instance\)$/mu,
  "quoted > characters in generic attributes must not shift the script content start",
);
assert.doesNotMatch(
  genericSetupRun.output,
  /\(at <script setup>:/u,
  "generic script setup diagnostics should not need fallback location suffixes",
);

console.log("✅ oxlint-plugin-vize script location tests passed!");
