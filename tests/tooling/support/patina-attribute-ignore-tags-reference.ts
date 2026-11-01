import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";
import { createRequire } from "node:module";
import { spawnSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { baselineConfig } from "../../../tools/support/compat/fixtures/lint-divergence-baseline.mjs";

export const root = fileURLToPath(new URL("../../../", import.meta.url));
export const directory = path.join(
  root,
  "tests/_fixtures/differential/linter/attribute-hyphenation-ignore-tags-6097",
);
export const hash = (data: string | Buffer) => createHash("sha256").update(data).digest("hex");
export interface TagCase {
  id: string;
  file: string;
  filename: string;
  style: "always" | "never";
  ignoreTags: string[];
  scope: "literal" | "upstream-regexp-only";
}
export interface TagInputs {
  files: { path: string; bytes: number; sha256: string }[];
  rows: TagCase[];
}
export const inputsBytes = fs.readFileSync(path.join(directory, "inputs.json"));
export const inputs = JSON.parse(inputsBytes.toString()) as TagInputs;
export function sourceFor(row: TagCase): string {
  const pin = inputs.files.find((pin) => pin.path === row.file);
  assert(pin);
  const source = fs.readFileSync(path.join(root, pin.path));
  assert.equal(source.length, pin.bytes);
  assert.equal(hash(source), pin.sha256);
  return source.toString();
}
export async function capture(): Promise<unknown> {
  const require = createRequire(path.join(root, "tools/benchmarks/scripts/package.json"));
  const versions = {
    eslint: "10.4.1",
    "eslint-plugin-vue": "10.9.2",
    "vue-eslint-parser": "10.4.1",
    "@typescript-eslint/parser": "8.65.0",
  };
  for (const [name, version] of Object.entries(versions))
    assert.equal((require(name + "/package.json") as { version: string }).version, version);
  const pluginRoot = path.dirname(require.resolve("eslint-plugin-vue/package.json"));
  const providerSources = [
    [
      "dist/rules/attribute-hyphenation.js",
      3500,
      "ce050d17c802fb26ed9d8c7245ad3d55f28750705b86d97b59522c071aa46fbd",
    ],
    [
      "dist/utils/regexp.js",
      1875,
      "4ec4a0605ab8f3ce65949d015b5f33ab616a2aeaf47c1fec45d9cc21cc723ea5",
    ],
  ] as const;
  for (const [file, bytes, sha] of providerSources) {
    const body = fs.readFileSync(path.join(pluginRoot, file));
    assert.equal(body.length, bytes);
    assert.equal(hash(body), sha);
  }
  const runtime = {
    ESLint: require("eslint").ESLint,
    plugin: require("eslint-plugin-vue"),
    vueParser: require("vue-eslint-parser"),
    scriptParser: require("@typescript-eslint/parser"),
  };
  const rows = [];
  for (const row of inputs.rows) {
    const source = sourceFor(row);
    const config = baselineConfig(runtime, {
      "vue/attribute-hyphenation": ["warn", row.style, { ignoreTags: row.ignoreTags }],
    });
    const options = { cwd: root, overrideConfigFile: true, overrideConfig: config };
    const lint = new runtime.ESLint(options);
    const fixer = new runtime.ESLint({ ...options, fix: true });
    const file = { filePath: path.join(root, row.filename) };
    const verify = await lint.lintText(source, file);
    const fix = await fixer.lintText(source, file);
    assert.equal(fix.length, 1);
    const fixedSource = fix[0].output ?? source;
    const requery = await lint.lintText(fixedSource, file);
    rows.push({ ...row, source, sourceSha256: hash(source), verify, fix, fixedSource, requery });
  }
  return {
    schema: "vize.attribute-ignore-tags.reference.v1",
    sourceRoot: root,
    versions,
    providerSources,
    inputsSha256: hash(inputsBytes),
    rows,
  };
}

if (process.argv[2] === "--worker") {
  process.stdout.write(JSON.stringify(await capture(), null, 2) + "\n");
} else if (process.argv[2] === "--capture") {
  const output = process.argv[3];
  assert(output);
  fs.mkdirSync(output, { recursive: true });
  const command = [fileURLToPath(import.meta.url), "--worker"];
  const child = spawnSync(process.execPath, command, {
    cwd: root,
    encoding: "utf8",
    maxBuffer: 8 * 1024 * 1024,
  });
  const git = spawnSync("git", ["rev-parse", "HEAD"], { cwd: root, encoding: "utf8" });
  const packet = {
    schema: "vize.attribute-ignore-tags.reference-process.v1",
    head: git.stdout.trim(),
    run: process.env.GITHUB_RUN_ID ?? null,
    attempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    command: [process.execPath, ...command],
    cwd: root,
    status: child.status,
    signal: child.signal,
    error: child.error?.message ?? null,
    stdout: child.stdout,
    stderr: child.stderr,
    producerSha256: hash(fs.readFileSync(fileURLToPath(import.meta.url))),
    inputsSha256: hash(inputsBytes),
  };
  fs.writeFileSync(path.join(output, "process.json"), JSON.stringify(packet, null, 2) + "\n");
  assert.equal(git.status, 0);
  assert.equal(child.status, 0, child.stderr);
  assert.equal(child.signal, null);
  assert.equal(child.error, undefined);
  const result: unknown = JSON.parse(child.stdout);
  assert(result && typeof result === "object");
  fs.writeFileSync(path.join(output, "official.original.json"), child.stdout);
}
