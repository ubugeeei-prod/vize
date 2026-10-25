import assert from "node:assert/strict";
import { spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import { createRequire } from "node:module";
import os from "node:os";
import path from "node:path";
import { pathToFileURL } from "node:url";
import {
  nativePreparationIsActive,
  readActiveNativePreparation,
} from "../../native/scripts/test-preparation.mjs";

const root = path.resolve(import.meta.dirname, "../../..");
const packageRoot = path.join(root, "npm/prettier");
const args = process.argv.slice(2);
function argument(name, fallback) {
  const index = args.indexOf(name);
  return index < 0 ? fallback : args[index + 1];
}
const mode = argument("--mode", "published-binding");
assert.ok(["source-native", "published-binding"].includes(mode));
const output = path.resolve(
  argument("--output", fs.mkdtempSync(path.join(os.tmpdir(), "vize-prettier-consumer-"))),
);
fs.mkdirSync(output, { recursive: true });
const nativeVersion = argument("--native-version", "0.571.0");
if (mode === "source-native") {
  assert.equal(
    nativePreparationIsActive(path.join(root, "npm/native")),
    true,
    "source consumer requires the authenticated native preparation wrapper",
  );
}
const observations = [];
function run(command, parameters, cwd, input) {
  const result = spawnSync(command, parameters, {
    cwd,
    input,
    encoding: "utf8",
    timeout: 120_000,
    env: process.env,
  });
  observations.push({
    command,
    parameters,
    cwd,
    input,
    status: result.status,
    stdout: result.stdout,
    stderr: result.stderr,
    error: result.error?.message,
  });
  fs.writeFileSync(path.join(output, "commands.json"), JSON.stringify(observations, null, 2));
  assert.ifError(result.error);
  return result;
}
const manifest = JSON.parse(fs.readFileSync(path.join(packageRoot, "package.json"), "utf8"));
const packingNativeVersion = createRequire(path.join(packageRoot, "package.json"))(
  "@vizejs/native/package.json",
).version;
const pack = run(
  "npx",
  ["--yes", "pnpm@12.9.0", "pack", "--pack-destination", output],
  packageRoot,
);
assert.equal(pack.status, 0, pack.stderr);
const tarball = path.join(output, `${manifest.name}-${manifest.version}.tgz`);
assert.equal(fs.existsSync(tarball), true);
const consumer = path.join(output, "project with spaces 雪");
fs.mkdirSync(path.join(consumer, "src"), { recursive: true });
fs.writeFileSync(
  path.join(consumer, "package.json"),
  JSON.stringify(
    {
      private: true,
      type: "module",
      dependencies: {
        prettier: "3.9.10",
        "prettier-plugin-vize": `file:${tarball}`,
        "@vizejs/native": nativeVersion,
      },
      overrides: { "@vizejs/native": nativeVersion },
    },
    null,
    2,
  ),
);
const install = run(
  "npm",
  [
    "install",
    "--ignore-scripts",
    "--no-audit",
    "--no-fund",
    ...(mode === "source-native" ? ["--omit=optional"] : []),
  ],
  consumer,
);
assert.equal(install.status, 0, install.stderr);
if (mode === "source-native") {
  // The tarball is installed normally. Only its native dependency is explicitly
  // routed to the authenticated source build, rather than a registry addon.
  const installed = path.join(consumer, "node_modules/@vizejs/native");
  assert.equal(
    JSON.parse(fs.readFileSync(path.join(installed, "package.json"), "utf8")).name,
    "@vizejs/native",
  );
  fs.rmSync(installed, { recursive: true });
  fs.symlinkSync(path.join(root, "npm/native"), installed, "dir");
}
const require = createRequire(path.join(consumer, "package.json"));
const preparation =
  mode === "source-native" ? readActiveNativePreparation(path.join(root, "npm/native")) : null;
if (mode === "source-native") {
  assert.ok(preparation);
  const directSourceAddon = require(path.join(root, "npm/native", preparation.binding));
  assert.equal(
    require("@vizejs/native").formatSfc,
    directSourceAddon.formatSfc,
    "the installed adapter reaches the authenticated local source addon",
  );
}
const prettierPackage = require.resolve("prettier/package.json");
const cli = path.join(path.dirname(prettierPackage), "bin/prettier.cjs");
assert.equal(require("prettier/package.json").version, "3.9.10");
const installedRoot = path.join(consumer, "node_modules/prettier-plugin-vize");
const installedManifest = JSON.parse(
  fs.readFileSync(path.join(installedRoot, "package.json"), "utf8"),
);
assert.equal(
  installedManifest.dependencies["@vizejs/native"],
  packingNativeVersion,
  "the pack operation rewrites the installed workspace/native dependency for npm",
);
const installedPlugin = path.resolve(installedRoot, installedManifest.exports["."].import);
assert.equal(
  installedPlugin.startsWith(path.join(consumer, "node_modules/prettier-plugin-vize")),
  true,
);
const config = {
  plugins: ["prettier-plugin-vize"],
  printWidth: 64,
  singleQuote: true,
  semi: false,
  endOfLine: "crlf",
};
fs.writeFileSync(
  path.join(consumer, "prettier.config.mjs"),
  `export default ${JSON.stringify(config)};\n`,
);
fs.writeFileSync(path.join(consumer, ".prettierignore"), "ignored.vue\n");
const input =
  '<script setup lang="ts">const label:string="雪 😀";</script><template><p class="note">{{label}}</p></template><style>.note{color:red}</style>';
const golden =
  '<script setup lang="ts">\nconst label: string = \'雪 😀\'\n</script>\n\n<template>\n  <p class="note">{{ label }}</p>\n</template>\n\n<style>\n.note {\n  color: red;\n}\n</style>\n'.replaceAll(
    "\n",
    "\r\n",
  );
const app = path.join(consumer, "src/App.vue");
const art = path.join(consumer, "src/Example.art");
const empty = path.join(consumer, "src/Empty.vue");
const ignored = path.join(consumer, "ignored.vue");
fs.writeFileSync(app, input);
fs.writeFileSync(art, '<art title="Demo"><variant name="Basic"><p>雪</p></variant></art>');
fs.writeFileSync(empty, "");
fs.writeFileSync(ignored, "<template>< invalid ignored bytes\r\n");
const ignoredBytes = fs.readFileSync(ignored);
const execute = (parameters, input) => run(process.execPath, [cli, ...parameters], consumer, input);
assert.equal(execute(["--find-config-path", "src/App.vue"]).stdout.trim(), "prettier.config.mjs");
assert.equal(execute(["--check", "src/App.vue"]).status, 1);
assert.equal(
  execute(["--write", "src/App.vue", "src/Example.art", "src/Empty.vue", "ignored.vue"]).status,
  0,
);
assert.equal(fs.readFileSync(app, "utf8"), golden);
assert.equal(fs.statSync(empty).size, 0);
assert.deepEqual(fs.readFileSync(ignored), ignoredBytes);
const artBytes = fs.readFileSync(art);
assert.equal(
  execute(["--check", "src/App.vue", "src/Example.art", "src/Empty.vue", "ignored.vue"]).status,
  0,
);
assert.equal(execute(["--write", "src/App.vue", "src/Example.art"]).status, 0);
assert.equal(fs.readFileSync(app, "utf8"), golden);
assert.deepEqual(fs.readFileSync(art), artBytes);
const stdin = execute(["--stdin-filepath", "src/App.vue"], input);
assert.equal(stdin.status, 0, stdin.stderr);
assert.equal(stdin.stdout, golden);
const cursor = execute(
  ["--stdin-filepath", "src/App.vue", "--cursor-offset", String(input.indexOf("雪 😀") + 3)],
  input,
);
assert.equal(cursor.status, 0, cursor.stderr);
assert.equal(cursor.stdout, golden);
assert.equal(Number(cursor.stderr.trim()), golden.indexOf("雪 😀") + 3);
assert.equal(execute(["--check", "src/App.vue", "--vue-indent-script-and-style"]).status, 2);
assert.equal(fs.readFileSync(app, "utf8"), golden);
fs.writeFileSync(app, input);
assert.equal(execute(["--write", "src/App.vue", "--require-pragma"]).status, 0);
assert.equal(fs.readFileSync(app, "utf8"), input);
assert.equal(execute(["--write", "src/App.vue", "--insert-pragma"]).status, 0);
assert.equal(fs.readFileSync(app, "utf8"), `<!-- @format -->\r\n\r\n${golden}`);
const skip = `<!-- @noprettier -->\r\n${input}`;
fs.writeFileSync(app, skip);
assert.equal(execute(["--write", "src/App.vue", "--check-ignore-pragma"]).status, 0);
assert.equal(fs.readFileSync(app, "utf8"), skip);
fs.writeFileSync(app, '<template><!-- prettier-ignore --><p  id="x">雪</p></template>');
assert.equal(execute(["--write", "src/App.vue"]).status, 2);

const prettier = require("prettier");
const plugin = await import(pathToFileURL(installedPlugin).href);
const timings = [];
const benchmarkInput = input;
for (let iteration = 0; iteration < 31; iteration++) {
  const start = performance.now();
  const formatted = await prettier.format(benchmarkInput, {
    ...config,
    filepath: "Benchmark.vue",
    plugins: [plugin],
  });
  const elapsedMs = performance.now() - start;
  assert.equal(formatted, golden);
  timings.push(elapsedMs);
}
const warm = timings.slice(1).sort((a, b) => a - b);
const report = {
  mode,
  node: process.version,
  prettier: "3.9.10",
  plugin: manifest.name,
  pluginVersion: manifest.version,
  adapterTarballSha256: createHash("sha256").update(fs.readFileSync(tarball)).digest("hex"),
  packedNativeDependencyVersion: installedManifest.dependencies["@vizejs/native"],
  runtimeNativePackageVersion: require("@vizejs/native/package.json").version,
  sourcePreparation: preparation,
  nativePackage: require.resolve("@vizejs/native"),
  nativeDependencyInstalledVersion: nativeVersion,
  nativeTransport:
    mode === "source-native"
      ? "explicit workspace source symlink with mandatory prepared-binding identity"
      : "installed registry dependency override, not current native source",
  golden,
  art: artBytes.toString(),
  ignored: ignoredBytes.toString(),
  observations,
  timing: {
    firstFormatMs: timings[0],
    warmMedianMs: warm[Math.floor(warm.length / 2)],
    warmMinMs: warm[0],
    warmMaxMs: warm.at(-1),
    warmSamples: timings.slice(1),
    claim:
      "API time after module loading on one small mixed-language file; CLI checks use separate processes; no speedup or instruction-ceiling claim",
  },
};
fs.writeFileSync(path.join(output, "report.json"), JSON.stringify(report, null, 2));
console.log(
  JSON.stringify({ mode, report: path.join(output, "report.json"), timing: report.timing }),
);
