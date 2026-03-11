import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = path.dirname(fileURLToPath(import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../../..");
const oxlintBin = path.join(workspaceRoot, "node_modules/.bin/oxlint");
const pluginEntry = path.join(workspaceRoot, "npm/oxlint-plugin-patina/dist/index.js");
const fixtureDir = fs.mkdtempSync(path.join(os.tmpdir(), "patina-oxlint-plugin-"));
const configPath = path.join(fixtureDir, ".oxlintrc.json");
const vuePath = path.join(fixtureDir, "App.vue");

fs.writeFileSync(
  configPath,
  JSON.stringify(
    {
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      rules: {
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

let output = "";
let exitCode = 0;

try {
  execFileSync(oxlintBin, ["-c", configPath, vuePath], {
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

assert.notEqual(exitCode, 0, "oxlint should fail when Patina reports an error");
assert.match(output, /patina\(vue\/require-v-for-key\)/, "Patina rule name should be surfaced");
assert.match(
  output,
  /Actual Vue location: line 6, column \d+/,
  "Template diagnostics should preserve their true file location in the message",
);
assert.doesNotMatch(output, /\*\*|```/, "Terminal output should not contain raw Markdown markers");

console.log("✅ oxlint-plugin-patina integration tests passed!");
