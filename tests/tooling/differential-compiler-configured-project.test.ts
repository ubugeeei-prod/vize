import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

import { loadCompilerManifest, runCompilerPack } from "../differential/compiler.mjs";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import {
  COMPILER_CONFIGURED_PROJECT_ARGV,
  loadConfiguredCompilerProjectInputs,
  materializeConfiguredCompilerProjectInputs,
} from "../differential/compiler-configured-project.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const fixtureDirectory = fileURLToPath(
  new URL("../_fixtures/differential/compiler/", import.meta.url),
);
const prefix = "tests/_fixtures/differential/compiler/";
const root = "sfc-entry-grammar-11869";
const manifest = JSON.parse(fs.readFileSync(path.join(fixtureDirectory, "manifest.json"), "utf8"));
const fixture = manifest.cases.find((row: { id: string }) => row.id === `compiler/project/${root}`);
assert.ok(fixture, "the configured root-header regression must be registered");
const hash = (bytes: Buffer) => createHash("sha256").update(bytes).digest("hex");
const sourceFiles = new Map<string, Buffer>(
  [
    ...fixture.inputs.files.map((file: { path: string }) => `${root}/${file.path}`),
    fixture.reference.path,
  ].map((file: string) => [file, fs.readFileSync(path.join(fixtureDirectory, file))]),
);
const envelope = (cases: unknown[]) =>
  Buffer.from(
    JSON.stringify({
      schema: "vize.differential.manifest",
      version: 1,
      baseRevision: "a".repeat(40),
      product: "compiler",
      cases,
    }),
  );

function fixReport(row = fixture, files = sourceFiles) {
  return analyzeLegacyFix({
    title: "fix(parser): select SFC grammar in template entry points",
    paths: ["crates/vize_atelier_sfc/src/compile/entry/root_match.rs"],
    readFile: (side, file) =>
      file === prefix + "manifest.json"
        ? envelope(side === "base" ? [] : [row])
        : (files.get(file.slice(prefix.length)) ?? null),
  });
}

function writePack(directory: string, row = fixture, files = sourceFiles) {
  for (const [file, bytes] of files) {
    const output = path.join(directory, file);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, bytes);
  }
  const manifestPath = path.join(directory, "manifest.json");
  fs.writeFileSync(manifestPath, envelope([row]));
  return manifestPath;
}

test("the production manifest and fix gate authenticate both the regression and its configuration", () => {
  const loaded = loadCompilerManifest(path.join(fixtureDirectory, "manifest.json")).cases.find(
    (row: { id: string }) => row.id === fixture.id,
  );
  assert.ok(loaded);
  assert.match(loaded.input.toString(), /lang="html"v-match="result"data-note="sfc"/);
  assert.deepEqual(loaded.argv, COMPILER_CONFIGURED_PROJECT_ARGV);
  assert.equal(loaded.argv.includes("--no-config"), false);
  assert.deepEqual(
    loaded.configuredProjectFiles.map(({ path }: { path: string }) => path),
    ["Layout.vue", "vite.config.mjs"],
  );
  assert.deepEqual(fixReport().products, [
    { product: "compiler", state: "new-inputs-registered", cases: [fixture.id] },
  ]);
  const control = JSON.parse(
    fs.readFileSync(path.join(fixtureDirectory, root, "spaced-control.public-691.json"), "utf8"),
  );
  assert.deepEqual(Buffer.from(control.code), loaded.expected);
  assert.deepEqual(
    [control.css, control.errors, control.warnings, control.macro_artifacts],
    [null, [], [], []],
  );
  assert.match(fixture.reference.source, /transformed-input reference/);
});

test("missing or changed config, escaping paths and unregistered configuration never receive fixture credit", () => {
  for (const mutate of [
    (row: typeof fixture) => {
      row.inputs.files.pop();
    },
    (row: typeof fixture) => {
      row.inputs.files[1].path = "../vite.config.mjs.txt";
    },
    (row: typeof fixture) => {
      row.inputs.files[1] = { ...row.inputs.files[0] };
    },
    (row: typeof fixture) => {
      row.inputs.files[1].role = "entry";
    },
    (row: typeof fixture) => {
      row.inputs.root = "../outside";
    },
    (row: typeof fixture) => {
      row.comparison.requiredFacets.pop();
    },
  ]) {
    const row = structuredClone(fixture);
    mutate(row);
    assert.equal(fixReport(row).products[0]?.state, "invalid-manifest-or-input");
  }
  for (const replacement of [
    null,
    Buffer.from("forged"),
    Buffer.from("export default { vue: { experimentals: { patternedTemplate: false } } };\n"),
  ]) {
    const row = structuredClone(fixture);
    const files = new Map(sourceFiles);
    const config = `${root}/vite.config.mjs.txt`;
    if (replacement === null) files.delete(config);
    else {
      files.set(config, replacement);
      // Even a consistently rehashed unregistered program is outside this adapter.
      row.inputs.files[1].sha256 = hash(replacement);
    }
    assert.equal(fixReport(row, files).products[0]?.state, "invalid-manifest-or-input");
    assert.throws(() =>
      loadConfiguredCompilerProjectInputs(row, (file) => files.get(file) ?? null),
    );
  }
});

test("authenticated config bytes cannot change between load and materialization", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-configured-inputs-"));
  try {
    const files = loadConfiguredCompilerProjectInputs(
      fixture,
      (file) => sourceFiles.get(file) ?? null,
    );
    materializeConfiguredCompilerProjectInputs(directory, files);
    assert.deepEqual(fs.readdirSync(directory).sort(), ["Layout.vue", "vite.config.mjs"]);
    for (const file of files)
      assert.deepEqual(fs.readFileSync(path.join(directory, file.path)), file.bytes);
    assert.throws(() => materializeConfiguredCompilerProjectInputs(directory, files.slice(0, 1)));
    assert.throws(() =>
      materializeConfiguredCompilerProjectInputs(
        directory,
        files.map((file, i) => (i === 1 ? { ...file, path: "../escape.mjs" } : file)),
      ),
    );
    assert.throws(() =>
      materializeConfiguredCompilerProjectInputs(
        directory,
        files.map((file, i) => (i === 1 ? { ...file, bytes: Buffer.from("changed") } : file)),
      ),
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

test(
  "the compiler pack executes the pinned configured project with its explicit config argv",
  { skip: process.platform === "win32" },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "vize-configured-runner-"));
    try {
      const manifestPath = writePack(path.join(directory, "pack"));
      const repo = path.join(directory, "source");
      fs.mkdirSync(path.join(repo, "target/ci"), { recursive: true });
      fs.writeFileSync(path.join(repo, "Cargo.toml"), '[workspace.package]\nversion = "0.0.0"\n');
      execFileSync("git", ["init", "-q", repo]);
      execFileSync("git", ["add", "Cargo.toml"], { cwd: repo });
      execFileSync(
        "git",
        [
          "-c",
          "user.name=Vize",
          "-c",
          "user.email=test@example.invalid",
          "commit",
          "-qm",
          "fixture",
        ],
        { cwd: repo },
      );
      const sourceRevision = execFileSync("git", ["rev-parse", "HEAD"], {
        cwd: repo,
        encoding: "utf8",
      }).trim();
      const binaryPath = path.join(repo, "target/ci/vize");
      const expectedFiles = fixture.inputs.files.map((file: { path: string }) => [
        file.path.slice(0, -4),
        sourceFiles.get(`${root}/${file.path}`)!.toString("base64"),
      ]);
      const expected = sourceFiles.get(fixture.reference.path)!.toString();
      // This executable checks process transport; it grants no compiler or native qualification.
      fs.writeFileSync(
        binaryPath,
        `#!${process.execPath}\nconst fs=require('node:fs'),assert=require('node:assert/strict');
if(process.argv[2]==='--version'){console.log('vize 0.0.0-dev');process.exit(0);}
assert.deepEqual(process.argv.slice(2),${JSON.stringify(COMPILER_CONFIGURED_PROJECT_ARGV)});
assert.deepEqual(fs.readdirSync('.').sort(),['Layout.vue','vite.config.mjs']);
for(const [file,bytes] of ${JSON.stringify(expectedFiles)})assert.deepEqual(fs.readFileSync(file),Buffer.from(bytes,'base64'));
fs.mkdirSync('out');fs.writeFileSync('out/Layout.json',JSON.stringify({filename:'Layout.vue',code:${JSON.stringify(expected)},css:null,errors:[],warnings:[],script_lang:'js',macro_artifacts:[]}));\n`,
      );
      fs.chmodSync(binaryPath, 0o755);
      writeBuildReceipt(repo);
      const result: {
        rows: Array<{
          legacy: {
            verdict: string;
            error?: string;
            configuredProjectInputs?: Array<{ path: string; sha256: string }>;
          };
          native: { state: string };
        }>;
      } = runCompilerPack({ manifestPath, binaryPath, sourceRevision, repoRoot: repo });
      assert.equal(result.rows[0].legacy.verdict, "matched-reference", result.rows[0].legacy.error);
      assert.deepEqual(
        result.rows[0].legacy.configuredProjectInputs,
        fixture.inputs.files.map((file: { path: string; sha256: string }) => ({
          path: file.path.slice(0, -4),
          sha256: file.sha256,
        })),
      );
      assert.equal(result.rows[0].native.state, "unsupported");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
