import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";

type OxlintResult = { exitCode: number; output: string };

export function assertEmptyScriptDiagnostic(
  fixtureDir: string,
  runOxlint: (args: readonly string[]) => OxlintResult,
): void {
  fs.writeFileSync(
    path.join(fixtureDir, "EmptyScript.vue"),
    `<script setup lang="ts"></script>
<template>
  <ul>
    <li v-for="item in [1, 2]">{{ item }}</li>
  </ul>
</template>
`,
  );

  const result = runOxlint(["-c", ".oxlintrc.scriptless.json", "-f", "json", "EmptyScript.vue"]);
  assert.notEqual(result.exitCode, 0, "empty scripts must still report template diagnostics");
  assert.match(result.output, /vize\(vue\/require-v-for-key\)/u);
  assert.doesNotMatch(result.output, /Error running JS plugin|RangeError/u);
}
