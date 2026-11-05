import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { ESLint } from "eslint";
import plugin from "eslint-plugin-vize";

const require = createRequire(import.meta.url);
const pluginEntry = import.meta.resolve("eslint-plugin-vize");
const pluginRequire = createRequire(fileURLToPath(pluginEntry));
const parserEntry = pluginRequire.resolve("@typescript-eslint/parser");
const parserRequire = createRequire(parserEntry);
const typescriptEntry = parserRequire.resolve("typescript");
const typescript = parserRequire("typescript");
assert.equal(typescriptEntry, require.resolve("typescript"));
assert.equal(typescript, require("typescript"));
if (process.env.VIZE_ESLINT_EXPECTED_NODE) {
  assert.equal(process.versions.node, process.env.VIZE_ESLINT_EXPECTED_NODE);
}
if (process.env.VIZE_ESLINT_EXPECTED_TYPESCRIPT) {
  assert.equal(typescript.version, process.env.VIZE_ESLINT_EXPECTED_TYPESCRIPT);
}
const native = require("@vizejs/native");
const nativeImages = Object.values(require.cache)
  .filter((entry) => entry?.filename.endsWith(".node") && entry.exports === native)
  .map((entry) => ({
    path: fs.realpathSync(entry.filename),
    sha256: createHash("sha256").update(fs.readFileSync(entry.filename)).digest("hex"),
  }));
assert.equal(nativeImages.length, 1);
assert.equal(plugin.meta.name, "eslint-plugin-vize");
const eslint = new ESLint({
  overrideConfigFile: true,
  overrideConfig: [
    {
      ...plugin.configs.recommended,
      settings: { vize: { helpLevel: "none", locale: "en", preset: "incremental" } },
      rules: { "vize/a11y/alt-text": "error", "vize/vue/no-multi-spaces": "error" },
    },
  ],
  fix: true,
});
const scripts = {
  scriptless: "",
  javascript: "<script setup>const n = 1;</script>\r\n",
  typescript: '<script setup lang="ts">const n: number = 1;</script>\r\n',
  tsx: '<script setup lang="tsx">const node = <span />;</script>\r\n',
  mixed:
    '<script>export default {};</script>\r\n<script setup lang="ts">const n: number = 1;</script>\r\n',
};
const packets = [];
function normalized(results) {
  return JSON.parse(JSON.stringify(results)).map((result) => ({
    ...result,
    filePath: path.basename(result.filePath),
  }));
}
function expectedResult(filePath, messages, output) {
  return {
    filePath,
    messages,
    suppressedMessages: [],
    errorCount: messages.length,
    fatalErrorCount: 0,
    warningCount: 0,
    fixableErrorCount: 0,
    fixableWarningCount: 0,
    ...(output === undefined ? {} : { output }),
    usedDeprecatedRules: [],
  };
}
for (const [kind, script] of Object.entries(scripts)) {
  const filePath = `${kind}.vue`;
  const source = `<template>😀<img src="logo"><div  id="x"></div></template>\r\n${script}`;
  const output = source.replace("div  id", "div id");
  const result = await eslint.lintText(source, { filePath });
  assert.deepEqual(normalized(result), [
    expectedResult(
      filePath,
      [
        {
          ruleId: "vize/a11y/alt-text",
          severity: 2,
          message: "<img> elements must have an alt attribute",
          line: 1,
          column: 13,
          endLine: 1,
          endColumn: 29,
        },
      ],
      output,
    ),
  ]);
  const cleanSource = output.replace('<img src="logo">', '<img src="logo" alt="Logo">');
  const clean = await eslint.lintText(cleanSource, { filePath });
  assert.deepEqual(normalized(clean), [expectedResult(filePath, [])]);
  packets.push({ kind, source, result, cleanSource, clean });
}
const providers = Object.fromEntries(
  ["eslint", "vue-eslint-parser", "@typescript-eslint/parser", "typescript"].map((name) => {
    const providerRequire = name === "eslint" || name === "typescript" ? require : pluginRequire;
    const entry = name === "typescript" ? typescriptEntry : providerRequire.resolve(name);
    const packagePath = providerRequire.resolve(`${name}/package.json`);
    return [
      name,
      {
        entry: fs.realpathSync(entry),
        entrySha256: createHash("sha256").update(fs.readFileSync(entry)).digest("hex"),
        packagePath: fs.realpathSync(packagePath),
        packageSha256: createHash("sha256").update(fs.readFileSync(packagePath)).digest("hex"),
        version: providerRequire(`${name}/package.json`).version,
      },
    ];
  }),
);
console.log(
  JSON.stringify({
    plugin: plugin.meta,
    nodeVersion: process.versions.node,
    eslintVersion: ESLint.version,
    typescriptVersion: typescript.version,
    providers,
    nativeBuildToken: native.getNativeBuildToken(),
    nativeImages,
    pluginEntry,
    packets,
  }),
);
