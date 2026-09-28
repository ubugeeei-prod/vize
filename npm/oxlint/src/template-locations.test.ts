import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const workspaceRoot = path.resolve(packageDir, "../..");
const pluginEntry = path.join(packageDir, "dist/index.mjs");
const cliEntry = path.join(packageDir, "dist/cli.mjs");
const oxlintBin = path.join(workspaceRoot, "node_modules/oxlint/bin/oxlint");
const fixture = fs.mkdtempSync(path.join(os.tmpdir(), "vize-oxlint-template-loc-"));

try {
  fs.mkdirSync(path.join(fixture, ".git"));
  fs.mkdirSync(path.join(fixture, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(oxlintBin, path.join(fixture, "node_modules/oxlint/bin/oxlint"));
  fs.writeFileSync(
    path.join(fixture, ".oxlintrc.json"),
    JSON.stringify({
      plugins: ["vue"],
      jsPlugins: [pluginEntry],
      rules: { "vize/vue/require-v-for-key": "error" },
    }),
  );

  const cases = [
    {
      filename: "ScriptAndTemplate.vue",
      source: `<script setup lang="ts">\nconst items = [1, 2]\n</script>\n<template>\n  <ul>\n    <li v-for="item in items">{{ item }}</li>\n  </ul>\n</template>\n`,
      line: 6,
      column: 9,
    },
    {
      filename: "EmptyScript.vue",
      source: `<script setup lang="ts"></script>\n<template>\n  <ul>\n    <li v-for="item in [1, 2]">{{ item }}</li>\n  </ul>\n</template>\n`,
      line: 4,
      column: 9,
    },
    {
      filename: "Scriptless.vue",
      source: `<template>\n  <ul>\n    <li v-for="item in [1, 2]">{{ item }}</li>\n  </ul>\n</template>\n`,
      line: 3,
      column: 9,
    },
  ];

  for (const { filename, source, line, column } of cases) {
    fs.writeFileSync(path.join(fixture, filename), source);
    const run = spawnSync(
      process.execPath,
      [cliEntry, "-c", ".oxlintrc.json", "-f", "json", filename],
      { cwd: fixture, encoding: "utf8" },
    );
    assert.equal(run.error, undefined);
    const output = `${run.stdout ?? ""}${run.stderr ?? ""}`;
    assert.equal(run.status, 1, `${filename}: ${output}`);
    const report = JSON.parse(run.stdout) as {
      diagnostics: Array<{
        code: string;
        filename: string;
        labels: Array<{ span: { line: number; column: number } }>;
      }>;
    };
    const diagnostics = report.diagnostics.filter(
      (diagnostic) => diagnostic.code === "vize(vue/require-v-for-key)",
    );
    assert.equal(diagnostics.length, 1, `${filename}: ${output}`);
    assert.equal(diagnostics[0].filename, filename);
    assert.equal(diagnostics[0].labels[0].span.line, line);
    assert.equal(diagnostics[0].labels[0].span.column, column);
    assert.doesNotMatch(output, /node_modules\/\.vize\/oxlint-plugin-vize/u);
  }
} finally {
  fs.rmSync(fixture, { recursive: true, force: true });
}
