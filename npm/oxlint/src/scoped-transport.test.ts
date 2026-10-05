import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const cli = path.join(packageDir, "dist/cli.mjs");
const engine = path.resolve(packageDir, "../../node_modules/oxlint/bin/oxlint");
const plugin = path.join(packageDir, "dist/index.mjs");
const scripted =
  '<script setup lang="ts">\nconst items = [1, 2]\n</script>\n<template>\n  <ul>\n    <li v-for="item in items">{{ item }}</li>\n  </ul>\n</template>\n';
const scriptless =
  '<template>\n  <ul>\n    <li v-for="item in [1, 2]">{{ item }}</li>\n  </ul>\n</template>\n';

void test("real Oxlint scoped transport preserves original filters, positions and namespace custody", (t) => {
  const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-scoped-physical-"));
  t.after(() => fs.rmSync(root, { recursive: true, force: true }));
  fs.mkdirSync(path.join(root, "node_modules/oxlint/bin"), { recursive: true });
  fs.symlinkSync(engine, path.join(root, "node_modules/oxlint/bin/oxlint"));
  fs.mkdirSync(path.join(root, ".git"));
  fs.writeFileSync(path.join(root, ".gitignore"), "app/components/Ignored.vue\n");
  const cases = [
    ["app/components/Scripted.vue", scripted, 6],
    ["app/components/Scriptless.vue", scriptless, 3],
    ["app/components/[slug].vue", scripted, 6],
    ["app/components/Ignored.vue", scripted, 6],
    ["app/components/CliIgnored.vue", scripted, 6],
    ["app/pages/About.vue", scripted, 6],
    ["app/pages/generated/Generated.vue", scriptless, 3],
    ["dist/Nested.vue", scripted, 6],
  ] as const;
  for (const [name, source] of cases) {
    fs.mkdirSync(path.dirname(path.join(root, name)), { recursive: true });
    fs.writeFileSync(path.join(root, name), source);
  }
  const config =
    JSON.stringify(
      {
        plugins: ["vue"],
        jsPlugins: [plugin],
        settings: { vize: { preset: "incremental", helpLevel: "none" } },
        ignorePatterns: ["**/node_modules", "**/dist", "**/oxlint-vize-*/**"],
        rules: { "no-unused-vars": "off", "vize/vue/require-v-for-key": "warn" },
        overrides: [
          {
            files: ["app/pages/**/*.vue"],
            excludeFiles: ["app/pages/generated/**"],
            rules: { "vize/vue/require-v-for-key": "off" },
          },
          { files: ["**/oxlint-vize-*/**"], rules: { "vize/vue/require-v-for-key": "error" } },
        ],
      },
      null,
      2,
    ) + "\n";
  fs.writeFileSync(path.join(root, "config.json"), config);
  const before = fs.readdirSync(root).sort();
  const run = spawnSync(
    process.execPath,
    [
      cli,
      "--config",
      "config.json",
      "--format",
      "json",
      "--ignore-pattern",
      "app/components/CliIgnored.vue",
      "app",
      "dist",
    ],
    { cwd: root, encoding: "utf8", timeout: 30_000 },
  );
  assert.equal(run.error, undefined);
  assert.equal(run.signal, null);
  assert.equal(run.status, 0, run.stdout + run.stderr);
  assert.equal(run.stderr, "");
  const report = JSON.parse(run.stdout);
  assert.equal(report.number_of_files, 5);
  assert.deepEqual(
    report.diagnostics
      .map(
        (row: {
          code: string;
          filename: string;
          message: string;
          severity: string;
          labels: Array<{ span: { line: number; column: number } }>;
        }) => ({
          code: row.code,
          filename: row.filename,
          message: row.message,
          severity: row.severity,
          positions: row.labels.map(({ span }) => [span.line, span.column]),
        }),
      )
      .sort((a: { filename: string }, b: { filename: string }) =>
        a.filename.localeCompare(b.filename),
      ),
    [
      ["app/components/[slug].vue", 6],
      ["app/components/Scripted.vue", 6],
      ["app/components/Scriptless.vue", 3],
      ["app/pages/generated/Generated.vue", 3],
    ]
      .map(([filename, line]) => ({
        code: "vize(vue/require-v-for-key)",
        filename,
        message:
          "Elements in iteration expect to have 'v-bind:key' directives.\n    Details:\n      Element: <li>",
        severity: "warning",
        positions: [[line, 9]],
      }))
      .sort((a, b) => String(a.filename).localeCompare(String(b.filename))),
  );
  assert.equal(fs.readFileSync(path.join(root, "config.json"), "utf8"), config);
  for (const [name, source] of cases)
    assert.equal(fs.readFileSync(path.join(root, name), "utf8"), source);
  assert.deepEqual(fs.readdirSync(root).sort(), before);
});
