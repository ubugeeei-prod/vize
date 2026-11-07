import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const COMPILER_PROJECT_ROOT = "imported-props-path-alias-11794";
export const CONDITIONS_PROJECT_ROOT = "macro-export-conditions-11918";
export const STAR_EXPORT_PROJECT_ROOT = "star-export-shadow-pruning-12059";
export const COMPILER_PROJECT_ADAPTER = "compiler-cli-ssr-project-v1";
export const COMPILER_PROJECT_CONTRACT = "compiler-ssr-project-module-bytes-v1";
const inputs = [
  ["Layout.vue.txt", "entry", "Layout.vue"],
  ["tsconfig.json.txt", "config", "tsconfig.json"],
  ["src/FooProps.ts.txt", "dependency", "src/FooProps.ts"],
] as const;
const conditionsInputs = [
  ["Layout.vue.txt", "entry", "Layout.vue"],
  ["tsconfig.json.txt", "config", "tsconfig.json"],
  ["lib.package.json.txt", "dependency", "node_modules/lib/package.json"],
  ["lib.index.ts.txt", "dependency", "node_modules/lib/src/index.ts"],
  ["lib.runtime.js.txt", "dependency", "node_modules/lib/dist/index.js"],
] as const;
const starExportInputs = [
  ["Layout.vue.txt", "entry", "Layout.vue"],
  ["tsconfig.json.txt", "config", "tsconfig.json"],
  ["types/index.ts.txt", "dependency", "types/index.ts"],
  ["types/Props.ts.txt", "dependency", "types/Props.ts"],
  ["types/Unused.ts.txt", "dependency", "types/Unused.ts"],
] as const;
const cohorts = new Map<
  string,
  { inputs: readonly (readonly [string, string, string])[]; selector: string }
>([
  [COMPILER_PROJECT_ROOT, { inputs, selector: "TypeScript paths" }],
  [
    CONDITIONS_PROJECT_ROOT,
    { inputs: conditionsInputs, selector: "TypeScript custom export conditions" },
  ],
  [
    STAR_EXPORT_PROJECT_ROOT,
    { inputs: starExportInputs, selector: "explicit types shadowing star exports" },
  ],
]);
type ProjectCase = {
  id: string;
  state: string;
  targets: string[];
  inputs: { root: string; files: { path: string; role: string; sha256: string }[] };
  adapters: {
    legacy: string;
    legacyRecipe?: unknown;
    native: unknown;
    reasons: { native: string };
  };
  reference: { path: string; sha256: string; source: string };
  comparison: { contract: string; requiredFacets: string[]; scriptLang: string };
  dialectCoverage: unknown;
};
export type CompilerProjectFile = { path: string; sha256: string; bytes: Buffer };
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");

export function isCompilerProjectCase(value: unknown): boolean {
  return (value as ProjectCase | null)?.adapters?.legacy === COMPILER_PROJECT_ADAPTER;
}

/** Finite project cohorts; the single-SFC and dialect contracts remain separate. */
export function validateCompilerProjectCase(value: unknown): asserts value is ProjectCase {
  const row = value as ProjectCase;
  const cohort = cohorts.get(row.inputs.root);
  assert.ok(cohort, "registered finite compiler project cohort");
  assert.equal(row.id, `compiler/project/${row.inputs.root}`);
  assert.equal(row.state, "active");
  assert.deepEqual(row.targets, ["ssr"]);
  assert.deepEqual(Object.keys(row.inputs).sort(), ["files", "root"]);
  assert.ok(Array.isArray(row.inputs.files));
  assert.equal(row.inputs.files.length, cohort.inputs.length);
  for (const [index, [file, role]] of cohort.inputs.entries()) {
    const input = row.inputs.files[index]!;
    assert.deepEqual(Object.keys(input).sort(), ["path", "role", "sha256"]);
    assert.equal(input.path, file);
    assert.equal(input.role, role);
    assert.match(input.sha256, /^[a-f0-9]{64}$/);
  }
  assert.equal(row.adapters.legacy, COMPILER_PROJECT_ADAPTER);
  assert.equal(row.adapters.legacyRecipe, undefined);
  assert.equal(row.adapters.native, null);
  assert.equal(typeof row.adapters.reasons.native, "string");
  assert.ok(row.adapters.reasons.native.length > 0);
  assert.deepEqual(Object.keys(row.reference).sort(), ["path", "sha256", "source"]);
  assert.equal(row.reference.path, `${row.inputs.root}/module.expected.txt`);
  assert.match(row.reference.sha256, /^[a-f0-9]{64}$/);
  assert.equal(typeof row.reference.source, "string");
  assert.ok(row.reference.source.length > 0);
  assert.equal(row.comparison.contract, COMPILER_PROJECT_CONTRACT);
  assert.equal(row.comparison.scriptLang, "ts");
  assert.deepEqual(row.comparison.requiredFacets, [
    "code",
    "css",
    "errors",
    "warnings",
    "macro_artifacts",
  ]);
  assert.deepEqual(row.dialectCoverage, {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ["vue3-sfc", "html-template", "ts"],
        selector: `${COMPILER_PROJECT_ADAPTER} project inputs with ${cohort.selector}`,
      },
    ],
  });
}

export function loadCompilerProjectInputs(value: unknown, read: (file: string) => Buffer | null) {
  validateCompilerProjectCase(value);
  const cohort = cohorts.get(value.inputs.root)!;
  return cohort.inputs.map(([file, , output], index): CompilerProjectFile => {
    const pinned = value.inputs.files[index]!;
    const bytes = read(`${value.inputs.root}/${file}`);
    assert.ok(bytes, `missing pinned compiler project input: ${file}`);
    assert.equal(
      hash(bytes),
      pinned.sha256,
      `SHA256 mismatch: changed pinned compiler project input: ${file}`,
    );
    return { path: output, sha256: pinned.sha256, bytes };
  });
}

/** Called by the real CLI runner, with every file from the authenticated project. */
export function materializeCompilerProjectInputs(directory: string, files: CompilerProjectFile[]) {
  const paths = files.map((file) => file.path);
  const cohort = [...cohorts.values()].find(
    (cohort) =>
      JSON.stringify(paths) === JSON.stringify(cohort.inputs.map(([, , output]) => output)),
  );
  assert.ok(cohort, "complete registered compiler project input paths");
  for (const file of files) {
    assert.equal(
      hash(file.bytes),
      file.sha256,
      `project input changed before materialization: ${file.path}`,
    );
    const output = path.join(directory, file.path);
    fs.mkdirSync(path.dirname(output), { recursive: true });
    fs.writeFileSync(output, file.bytes);
  }
}
