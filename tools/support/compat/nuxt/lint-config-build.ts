// Preserve both existing Nuxt projects and copy the original issue into finite probes.
import assert from "node:assert/strict";
import { execFileSync, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { asRecord, readJson } from "../../json-value.ts";
import { readSourceBindingCustody } from "./source-binding.ts";

const root = fileURLToPath(new URL("../../../../", import.meta.url));
const [outputArgument, custodyArgument] = process.argv.slice(2);
assert.ok(outputArgument && custodyArgument, "usage: lint-config-build.ts OUTPUT CUSTODY");
const output = path.resolve(outputArgument);
const compilerCustody = readSourceBindingCustody(path.resolve(custodyArgument));
assert.equal(compilerCustody.schema, "vize.nuxt.source-binding");
assert.equal(compilerCustody.version, 1);
for (const key of [
  "NAPI_RS_NATIVE_LIBRARY_PATH",
  "NAPI_RS_FORCE_WASI",
  "VIZE_NUXT_NATIVE_CUSTODY",
  "VIZE_NUXT_LINT_NATIVE_CUSTODY",
])
  assert.ok(!process.env[key], `ambient ${key} cannot qualify source`);
assert.equal(compilerCustody.source.head, process.env.GITHUB_SHA);
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" }).trim(),
  compilerCustody.source.head,
);
assert.equal(
  execFileSync("git", ["rev-parse", "HEAD^{tree}"], { cwd: root, encoding: "utf8" }).trim(),
  compilerCustody.source.tree,
);
const hash = (bytes: Uint8Array) => createHash("sha256").update(bytes).digest("hex");
assert.equal(hash(fs.readFileSync(compilerCustody.binary.path)), compilerCustody.binary.sha256);
const inputDir = path.join(root, "tests/_fixtures/differential/linter/nuxt-oxlint-config-root");
const corpus = asRecord(readJson(path.join(inputDir, "corpus.json")), "lint corpus");
for (const [name, digest] of Object.entries(asRecord(corpus.files, "corpus.files"))) {
  assert.equal(hash(fs.readFileSync(path.join(inputDir, name))), digest);
}
fs.mkdirSync(output, { recursive: true });
fs.cpSync(inputDir, path.join(output, "original-inputs"), { recursive: true });
const packages: readonly (readonly [string, string])[] = [
  ["@vizejs/nuxt", "npm/framework/nuxt"],
  ["@vizejs/nuxt-lint-config", "npm/framework/nuxt-lint-config"],
  ["oxlint-plugin-vize", "npm/oxlint"],
];
for (const [name, source] of packages) {
  const target = path.join(output, "current-source-pack", name);
  fs.mkdirSync(target, { recursive: true });
  fs.copyFileSync(path.join(root, source, "package.json"), path.join(target, "package.json"));
  fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
  if (name === "oxlint-plugin-vize")
    fs.cpSync(path.join(root, source, "bin"), path.join(target, "bin"), { recursive: true });
}
const distHashes = (directory: string): Record<string, string> =>
  Object.fromEntries(
    fs
      .readdirSync(directory, { recursive: true, encoding: "utf8" })
      .sort((a, b) => a.localeCompare(b))
      .flatMap((name) => {
        const file = path.join(directory, name);
        return fs.statSync(file).isFile() ? [[name, hash(fs.readFileSync(file))]] : [];
      }),
  );
for (const [cohort, fixture, version] of [
  ["nuxt3", "tools/support/compat/nuxt/fixtures/nuxt3-module-build", "3.19.3"],
  ["nuxt4", "tests/_fixtures/_projects/nuxt-critical-css-build", "4.5.2"],
] as const) {
  const installed = path.join(root, fixture, "node_modules");
  const artifacts = path.join(output, cohort);
  fs.mkdirSync(artifacts);
  const project = fs.mkdtempSync(path.join(artifacts, "project-"));
  const configSource = path.join(root, fixture, "nuxt.config.ts");
  const originalConfig = fs.readFileSync(configSource);
  const modulePackages = path.join(project, "node_modules");
  fs.mkdirSync(modulePackages);
  try {
    fs.writeFileSync(path.join(project, "package.json"), '{"private":true,"type":"module"}\n');
    fs.writeFileSync(path.join(project, "nuxt.config.ts"), originalConfig);
    for (const name of fs.readdirSync(installed)) {
      if (name.startsWith("@")) {
        const scope = path.join(modulePackages, name);
        fs.mkdirSync(scope);
        for (const child of fs.readdirSync(path.join(installed, name))) {
          fs.symlinkSync(path.join(installed, name, child), path.join(scope, child));
        }
      } else fs.symlinkSync(path.join(installed, name), path.join(modulePackages, name));
    }
    const originals: Record<string, Record<string, string>> = {};
    for (const [name, source] of packages) {
      const target = path.join(modulePackages, name);
      fs.rmSync(target, { recursive: true, force: true });
      fs.mkdirSync(target, { recursive: true });
      fs.copyFileSync(
        path.join(installed, name, "package.json"),
        path.join(target, "package.json"),
      );
      fs.cpSync(path.join(root, source, "dist"), path.join(target, "dist"), { recursive: true });
      if (name === "oxlint-plugin-vize")
        fs.cpSync(path.join(root, source, "bin"), path.join(target, "bin"), { recursive: true });
      const original = distHashes(path.join(root, source, "dist"));
      originals[name] = original;
      assert.deepEqual(distHashes(path.join(target, "dist")), original);
    }
    // The exact workspace engine is already installed by this automatic job.
    const engine = path.join(modulePackages, "oxlint");
    fs.rmSync(engine, { recursive: true, force: true });
    fs.symlinkSync(path.join(root, "node_modules/oxlint"), engine);
    const custody = {
      schema: "vize.nuxt.lint-source-binding",
      version: 1,
      source: compilerCustody.source,
      binary: compilerCustody.binary,
      calls: path.join(artifacts, "native-calls.jsonl"),
    };
    const custodyFile = path.join(artifacts, "custody.json");
    fs.writeFileSync(custodyFile, JSON.stringify(custody, null, 2) + "\n");
    const run = spawnSync(
      process.execPath,
      [
        fileURLToPath(new URL("./lint-config-probe.ts", import.meta.url)),
        project,
        artifacts,
        inputDir,
        version,
      ],
      {
        cwd: project,
        encoding: "utf8",
        timeout: 90_000,
        env: {
          ...process.env,
          NO_COLOR: "1",
          VIZE_NUXT_LINT_NATIVE_CUSTODY: custodyFile,
          NODE_OPTIONS:
            `${process.env.NODE_OPTIONS ?? ""} --require=${JSON.stringify(fileURLToPath(new URL("./lint-config-preload.cjs", import.meta.url)))}`.trim(),
        },
      },
    );
    fs.writeFileSync(
      path.join(artifacts, "probe-process.json"),
      JSON.stringify(
        {
          executionHead: process.env.GITHUB_SHA,
          version,
          packages: originals,
          fixtureConfigSha256: hash(originalConfig),
          status: run.status,
          signal: run.signal,
          error: run.error?.message ?? null,
          stdout: run.stdout,
          stderr: run.stderr,
        },
        null,
        2,
      ) + "\n",
    );
    assert.equal(run.error, undefined);
    assert.equal(run.signal, null);
    assert.equal(run.status, 0, run.stderr || run.stdout);
    assert.deepEqual(fs.readFileSync(configSource), originalConfig);
  } finally {
    fs.rmSync(project, { recursive: true, force: true });
  }
}
assert.equal(hash(fs.readFileSync(compilerCustody.binary.path)), compilerCustody.binary.sha256);
console.log("Original Nuxt 3/4 Oxlint ignores, overrides and source-native controls passed");
