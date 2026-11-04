import assert from "node:assert/strict";
import { execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import fs from "node:fs";
import os from "node:os";
import path from "node:path";
import { test } from "node:test";
import { loadCompilerManifest, runCompilerPack } from "../differential/compiler.mjs";
import { writeBuildReceipt } from "../differential/build-receipt.ts";
import {
  loadCompilerProjectInputs,
  materializeCompilerProjectInputs,
} from "../differential/compiler-project.ts";
import { analyzeLegacyFix } from "../../tools/support/compat/github/report-legacy-fix-fixtures.ts";

const root = "imported-props-path-alias-11794";
const prefix = "tests/_fixtures/differential/compiler/";
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");
const sources = new Map([
  [
    "Layout.vue.txt",
    Buffer.from(
      '<script setup lang="ts">import type { FooProps } from "@lib/FooProps";defineProps<FooProps>();</script><template><div /></template>\n',
    ),
  ],
  ["tsconfig.json.txt", Buffer.from('{"compilerOptions":{"paths":{"@lib/*":["./src/*"]}}}\n')],
  [
    "src/FooProps.ts.txt",
    Buffer.from("export interface FooProps { bar?: Record<string, unknown> }\n"),
  ],
]);
const expected = Buffer.from("export const fixture = true;\n");
const row = {
  id: `compiler/project/${root}`,
  state: "active",
  targets: ["ssr"],
  inputs: {
    root,
    files: [...sources].map(([file, body], i) => ({
      path: file,
      role: ["entry", "config", "dependency"][i],
      sha256: hash(body),
    })),
  },
  adapters: {
    legacy: "compiler-cli-ssr-project-v1",
    native: null,
    reasons: { native: "Whole native compiler path is not qualified by this fixture." },
  },
  reference: {
    path: `${root}/module.expected.txt`,
    sha256: hash(expected),
    source: "11794 whole SSR oracle",
  },
  comparison: {
    contract: "compiler-ssr-project-module-bytes-v1",
    requiredFacets: ["code", "css", "errors", "warnings", "macro_artifacts"],
    scriptLang: "ts",
  },
  dialectCoverage: {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ["vue3-sfc", "html-template", "ts"],
        selector: "compiler-cli-ssr-project-v1 project inputs with TypeScript paths",
      },
    ],
  },
};
function tree(fixture = row) {
  const manifest = (cases: unknown[]) =>
    Buffer.from(
      JSON.stringify({
        schema: "vize.differential.manifest",
        version: 1,
        baseRevision: "a".repeat(40),
        product: "compiler",
        cases,
      }),
    );
  const files = new Map([...sources].map(([file, body]) => [prefix + root + "/" + file, body]));
  files.set(prefix + root + "/module.expected.txt", expected);
  return { files, before: manifest([]), after: manifest([fixture]) };
}
function report(fixture = row, mutate?: (files: Map<string, Buffer>) => void) {
  const data = tree(fixture);
  mutate?.(data.files);
  return analyzeLegacyFix({
    title: "fix(compiler): resolve imported props paths",
    paths: ["crates/vize_atelier_sfc/src/compile.rs"],
    readFile: (side, file) =>
      file === prefix + "manifest.json"
        ? side === "base"
          ? data.before
          : data.after
        : (data.files.get(file) ?? null),
  });
}
void test("the fix gate accepts the complete pinned three-file compiler project", () => {
  assert.deepEqual(report().products, [
    { product: "compiler", state: "new-inputs-registered", cases: [row.id] },
  ]);
});
void test("the actual compiler manifest loader admits a project without changing its SSR facets", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "compiler-project-11794-"));
  try {
    const data = tree();
    for (const [file, body] of data.files) {
      const output = path.join(directory, file.slice(prefix.length));
      fs.mkdirSync(path.dirname(output), { recursive: true });
      fs.writeFileSync(output, body);
    }
    const manifest = path.join(directory, "manifest.json");
    fs.writeFileSync(manifest, data.after);
    const loaded = loadCompilerManifest(manifest).cases[0];
    assert.equal(loaded.id, row.id);
    assert.deepEqual(loaded.input, sources.get("Layout.vue.txt"));
    assert.deepEqual(loaded.expected, expected);
    assert.deepEqual(loaded.comparison.requiredFacets, row.comparison.requiredFacets);
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test("the finite project contract refuses missing, duplicate, escaping and unknown inputs", () => {
  for (const change of [
    (value: typeof row) => {
      value.inputs.files.pop();
    },
    (value: typeof row) => {
      value.inputs.files[1] = value.inputs.files[0]!;
    },
    (value: typeof row) => {
      value.inputs.files.push({ ...value.inputs.files[0]!, path: "extra.ts.txt" });
    },
    (value: typeof row) => {
      value.inputs.files[1]!.role = "unknown";
    },
    ...[
      "../outside.txt",
      "/outside.txt",
      "src/../FooProps.ts.txt",
      "src\\FooProps.ts.txt",
      "src/%2E%2E/FooProps.ts.txt",
    ].map((file) => (value: typeof row) => {
      value.inputs.files[2]!.path = file;
    }),
    (value: typeof row) => {
      value.inputs.root = "../outside";
    },
    (value: typeof row) => {
      value.comparison.requiredFacets.pop();
    },
    (value: typeof row) => {
      value.reference.path = "../module.expected.txt";
    },
    (value: typeof row) => {
      value.adapters.legacy = "compiler-cli-ssr-v1";
    },
  ]) {
    const invalid = structuredClone(row);
    change(invalid);
    assert.equal(report(invalid).products[0]?.state, "invalid-manifest-or-input");
    assert.throws(() =>
      loadCompilerProjectInputs(invalid, (file) => tree().files.get(prefix + file) ?? null),
    );
  }
});

void test("every project input and the whole SSR module must retain their pinned bytes", () => {
  for (const file of [...sources.keys(), "module.expected.txt"]) {
    const physical = prefix + root + "/" + file;
    for (const replacement of [null, Buffer.from("forged")]) {
      const mutate = (files: Map<string, Buffer>) => {
        if (replacement === null) files.delete(physical);
        else files.set(physical, replacement);
      };
      assert.equal(report(row, mutate).products[0]?.state, "invalid-manifest-or-input");
      if (file !== "module.expected.txt") {
        const data = tree();
        mutate(data.files);
        assert.throws(() =>
          loadCompilerProjectInputs(row, (file) => data.files.get(prefix + file) ?? null),
        );
      }
    }
  }
});

void test("the real compiler runner's materializer writes exactly the complete authenticated project", () => {
  const directory = fs.mkdtempSync(path.join(os.tmpdir(), "compiler-project-bytes-"));
  try {
    const files = loadCompilerProjectInputs(row, (file) => tree().files.get(prefix + file) ?? null);
    materializeCompilerProjectInputs(directory, files);
    assert.deepEqual(fs.readdirSync(directory).sort(), ["Layout.vue", "src", "tsconfig.json"]);
    assert.deepEqual(fs.readdirSync(path.join(directory, "src")), ["FooProps.ts"]);
    for (const file of files)
      assert.deepEqual(fs.readFileSync(path.join(directory, file.path)), file.bytes);
    assert.throws(() => materializeCompilerProjectInputs(directory, files.slice(0, 1)));
    assert.throws(() => materializeCompilerProjectInputs(directory, [...files, files[0]!]));
    assert.throws(() =>
      materializeCompilerProjectInputs(
        directory,
        files.map((file, i) => (i === 1 ? { ...file, path: "../escape" } : file)),
      ),
    );
    assert.throws(() =>
      materializeCompilerProjectInputs(
        directory,
        files.map((file, i) =>
          i === 2 ? { ...file, bytes: Buffer.from("changed after admission") } : file,
        ),
      ),
    );
  } finally {
    fs.rmSync(directory, { recursive: true, force: true });
  }
});

void test(
  "the actual compiler pack passes all three files to its executable",
  { skip: process.platform === "win32" },
  () => {
    const directory = fs.mkdtempSync(path.join(os.tmpdir(), "compiler-project-cli-contract-"));
    try {
      const data = tree();
      const pack = path.join(directory, "compiler");
      for (const [file, body] of data.files) {
        const output = path.join(pack, file.slice(prefix.length));
        fs.mkdirSync(path.dirname(output), { recursive: true });
        fs.writeFileSync(output, body);
      }
      const manifestPath = path.join(pack, "manifest.json");
      fs.writeFileSync(manifestPath, data.after);
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
      const expectedFiles = [...sources].map(([file, body]) => [
        file.slice(0, -4),
        body.toString("base64"),
      ]);
      // A protocol test executable, not a source-build or native qualification.
      fs.writeFileSync(
        binaryPath,
        `#!${process.execPath}\nconst fs=require('node:fs'),assert=require('node:assert/strict');
if(process.argv[2]==='--version'){console.log('vize 0.0.0-dev');process.exit(0);}
for(const [file,bytes] of ${JSON.stringify(expectedFiles)})assert.deepEqual(fs.readFileSync(file),Buffer.from(bytes,'base64'));
assert.deepEqual(fs.readdirSync('.').sort(),['Layout.vue','src','tsconfig.json']);
fs.mkdirSync('out');fs.writeFileSync('out/Layout.json',JSON.stringify({filename:'Layout.vue',code:${JSON.stringify(expected.toString())},css:null,errors:[],warnings:[],script_lang:'ts',macro_artifacts:[]}));\n`,
      );
      fs.chmodSync(binaryPath, 0o755);
      writeBuildReceipt(repo);
      const result = runCompilerPack({ manifestPath, binaryPath, sourceRevision, repoRoot: repo });
      const rows: { legacy: { verdict: string; error?: string }; native: { state: string } }[] =
        result.rows;
      assert.equal(rows.length, 1);
      assert.equal(rows[0]!.legacy.verdict, "matched-reference", rows[0]!.legacy.error);
      assert.equal(rows[0]!.native.state, "unsupported");
    } finally {
      fs.rmSync(directory, { recursive: true, force: true });
    }
  },
);
