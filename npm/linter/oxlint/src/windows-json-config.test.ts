import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import test from "node:test";
import { fileURLToPath } from "node:url";

const packageDir = fileURLToPath(new URL("..", import.meta.url));
const sourceCli = path.join(packageDir, "dist/cli.mjs");
const sourcePlugin = path.join(packageDir, "dist/index.mjs");
const publicVersion = process.env.OXLINT_VIZE_PUBLIC_VERSION;

void test(
  "Windows JSON configs retain original Vue locations",
  { skip: process.platform !== "win32" },
  (t) => {
    // Keep the fixture outside this repository: Oxlint honors the repository's
    // ignore rules even for an explicit file below an ignored target directory.
    const root = fs.mkdtempSync(path.join(os.tmpdir(), "oxlint-windows-json-config-"));
    t.after(() => fs.rmSync(root, { recursive: true, force: true }));
    let cli = sourceCli;
    let plugin = sourcePlugin;
    fs.writeFileSync(path.join(root, "package.json"), '{"private":true}\n');
    const packages = ["oxlint@1.92.0"];
    if (publicVersion != null) {
      assert.match(publicVersion, /^\d+\.\d+\.\d+$/u);
      packages.push(`oxlint-plugin-vize@${publicVersion}`);
    }
    const installCommand = [
      "npm",
      "install",
      "--save-dev",
      "--save-exact",
      "--no-audit",
      "--no-fund",
      ...packages,
    ].join(" ");
    const installed = spawnSync(installCommand, {
      cwd: root,
      encoding: "utf8",
      shell: true,
      timeout: 120_000,
    });
    assert.equal(installed.error, undefined);
    assert.equal(installed.signal, null);
    assert.equal(installed.status, 0, installed.stdout + installed.stderr);
    assert.equal(
      JSON.parse(fs.readFileSync(path.join(root, "node_modules/oxlint/package.json"), "utf8"))
        .version,
      "1.92.0",
    );
    if (publicVersion != null) {
      cli = path.join(root, "node_modules/oxlint-plugin-vize/dist/cli.mjs");
      plugin = "oxlint-plugin-vize";
      assert.equal(
        JSON.parse(
          fs.readFileSync(path.join(root, "node_modules/oxlint-plugin-vize/package.json"), "utf8"),
        ).version,
        publicVersion,
      );
    }
    const config = {
      jsPlugins: [plugin],
      rules: { "vize/vue/no-v-html": "error" },
    };
    fs.writeFileSync(path.join(root, ".oxlintrc.json"), `${JSON.stringify(config)}\n`);
    fs.writeFileSync(
      path.join(root, "App.vue"),
      '<script setup lang="ts">\nconst text = "<b>hello</b>";\n</script>\n\n<template>\n  <div v-html="text" />\n</template>\n',
    );
    const invoke = (args: readonly string[]) =>
      spawnSync(process.execPath, [cli, ...args], {
        cwd: root,
        encoding: "utf8",
        timeout: 30_000,
      });
    for (const args of [
      ["-c", ".oxlintrc.json", "-f", "stylish", "App.vue"],
      ["-f", "stylish", "App.vue"],
    ]) {
      const result = invoke(args);
      assert.equal(result.error, undefined);
      assert.equal(result.signal, null);
      assert.equal(result.status, 1, result.stderr);
      assert.equal(result.stderr, "");
      const colors = new RegExp(String.raw`\u001B\[[0-9;]*m`, "gu");
      const plain = result.stdout.replaceAll(colors, "");
      assert.match(plain, /App\.vue/u);
      assert.match(plain, /6:8\s+error\s+v-html can lead to XSS attacks\./u);
      assert.doesNotMatch(plain, /oxlint-vize-/u);
    }
    const writeCorpus = (directory: string, template: string) => {
      const target = path.join(root, directory);
      fs.mkdirSync(target);
      for (let index = 1; index <= 400; index += 1)
        fs.writeFileSync(
          path.join(target, `Component${String(index).padStart(3, "0")}.vue`),
          `<script setup lang="ts">\nconst text = "<b>hello</b>";\n</script>\n\n<template>\n  ${template}\n</template>\n`,
        );
    };
    writeCorpus("large", '<div v-html="text" />');
    writeCorpus("clean", "<div>{{ text }}</div>");
    const large = invoke(["-c", ".oxlintrc.json", "-f", "json", "large"]);
    assert.equal(large.error, undefined);
    assert.equal(large.signal, null);
    assert.equal(large.status, 1, large.stdout + large.stderr);
    assert.equal(large.stderr, "");
    const largeReport = JSON.parse(large.stdout) as {
      diagnostics: Array<{ code: string; filename: string }>;
      number_of_files: number;
    };
    assert.equal(largeReport.number_of_files, 400);
    assert.equal(
      largeReport.diagnostics.filter(({ code }) => code === "vize(vue/no-v-html)").length,
      400,
    );
    assert.equal(
      largeReport.diagnostics.every(({ filename }) => filename.includes("large")),
      true,
    );
    const clean = invoke(["-c", ".oxlintrc.json", "-f", "json", "clean"]);
    assert.equal(clean.error, undefined);
    assert.equal(clean.signal, null);
    assert.equal(clean.status, 0, clean.stdout + clean.stderr);
    assert.equal(clean.stderr, "");
    const cleanReport = JSON.parse(clean.stdout) as {
      diagnostics: unknown[];
      number_of_files: number;
    };
    assert.equal(cleanReport.number_of_files, 400);
    assert.deepEqual(cleanReport.diagnostics, []);
    const entries = fs.readdirSync(root).sort();
    assert.deepEqual(
      entries.filter((entry) => entry.startsWith("oxlint-vize-")),
      [],
    );
  },
);
