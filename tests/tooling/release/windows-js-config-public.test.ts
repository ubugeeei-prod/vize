import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";

const publicVersion = process.env.VIZE_PUBLIC_VERSION;

void test(
  "published Vize loads a JavaScript config on Windows",
  { skip: process.platform !== "win32" },
  (t) => {
    assert.match(publicVersion ?? "", /^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?$/u);
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "vize-windows-js-config-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    fs.writeFileSync(path.join(root, "package.json"), '{"private":true}\n');
    const install = spawnSync(
      `npm install --save-dev --save-exact --no-audit --no-fund vize@${publicVersion}`,
      { cwd: root, encoding: "utf8", shell: true, timeout: 120_000 },
    );
    assert.equal(install.error, undefined);
    assert.equal(install.signal, null);
    assert.equal(install.status, 0, install.stdout + install.stderr);
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(root, "node_modules/vize/package.json"), "utf8"))
        .version,
      publicVersion,
    );
    fs.writeFileSync(
      path.join(root, "probe-lint.config.mjs"),
      'export default { vue: { linter: { preset: "incremental", rules: { "vue/sort-keys": "off" } } } };\n',
    );
    fs.writeFileSync(
      path.join(root, "App.vue"),
      '<script setup lang="ts">\nconst label = "probe";\n</script>\n\n<template>\n  <p>{{ label }}</p>\n</template>\n',
    );
    const result = spawnSync(
      process.execPath,
      [
        path.join(root, "node_modules/vize/bin/vize"),
        "lint",
        "-c",
        "probe-lint.config.mjs",
        "-f",
        "stylish",
        "App.vue",
      ],
      { cwd: root, encoding: "utf8", timeout: 30_000 },
    );
    assert.equal(result.error, undefined);
    assert.equal(result.signal, null);
    assert.equal(result.status, 0, result.stdout + result.stderr);
    assert.equal(result.stderr, "");
    assert.match(result.stdout, /No problems found in 1 file\(s\)/u);
  },
);
