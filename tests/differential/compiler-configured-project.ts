import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
import path from "node:path";

export const COMPILER_CONFIGURED_PROJECT_ROOT = "sfc-entry-grammar-11869";
export const COMPILER_CONFIGURED_PROJECT_ADAPTER = "compiler-cli-ssr-configured-project-v1";
export const COMPILER_CONFIGURED_PROJECT_CONTRACT =
  "compiler-ssr-configured-project-module-bytes-v1";
export const COMPILER_CONFIGURED_PROJECT_CONFIG =
  "export default { vue: { experimentals: { patternedTemplate: true } } };\n";
export const COMPILER_CONFIGURED_PROJECT_ARGV = [
  "build",
  "Layout.vue",
  "--format",
  "json",
  "--output",
  "out",
  "--ssr",
  "--config",
  "vite.config.mjs",
];
const inputs = [
  ["Layout.vue.txt", "entry", "Layout.vue"],
  ["vite.config.mjs.txt", "config", "vite.config.mjs"],
] as const;
type ConfiguredProjectCase = {
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
export type ConfiguredCompilerProjectFile = { path: string; sha256: string; bytes: Buffer };
const hash = (body: Buffer) => createHash("sha256").update(body).digest("hex");

export function isConfiguredCompilerProjectCase(value: unknown): boolean {
  return (
    (value as ConfiguredProjectCase | null)?.adapters?.legacy ===
    COMPILER_CONFIGURED_PROJECT_ADAPTER
  );
}

/** Only the registered literal configuration is executable through this adapter. */
export function validateConfiguredCompilerProjectCase(
  value: unknown,
): asserts value is ConfiguredProjectCase {
  const row = value as ConfiguredProjectCase;
  assert.equal(row.id, `compiler/project/${COMPILER_CONFIGURED_PROJECT_ROOT}`);
  assert.equal(row.state, "active");
  assert.deepEqual(row.targets, ["ssr"]);
  assert.deepEqual(Object.keys(row.inputs).sort(), ["files", "root"]);
  assert.equal(row.inputs.root, COMPILER_CONFIGURED_PROJECT_ROOT);
  assert.equal(row.inputs.files.length, inputs.length);
  for (const [index, [file, role]] of inputs.entries()) {
    const input = row.inputs.files[index]!;
    assert.deepEqual(Object.keys(input).sort(), ["path", "role", "sha256"]);
    assert.equal(input.path, file);
    assert.equal(input.role, role);
    assert.match(input.sha256, /^[a-f0-9]{64}$/);
  }
  assert.equal(row.adapters.legacy, COMPILER_CONFIGURED_PROJECT_ADAPTER);
  assert.equal(row.adapters.legacyRecipe, undefined);
  assert.equal(row.adapters.native, null);
  assert.equal(typeof row.adapters.reasons.native, "string");
  assert.ok(row.adapters.reasons.native.length > 0);
  assert.deepEqual(Object.keys(row.reference).sort(), ["path", "sha256", "source"]);
  assert.equal(row.reference.path, `${COMPILER_CONFIGURED_PROJECT_ROOT}/module.expected.txt`);
  assert.match(row.reference.sha256, /^[a-f0-9]{64}$/);
  assert.equal(typeof row.reference.source, "string");
  assert.ok(row.reference.source.length > 0);
  assert.equal(row.comparison.contract, COMPILER_CONFIGURED_PROJECT_CONTRACT);
  assert.equal(row.comparison.scriptLang, "js");
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
        dialects: ["vue3-sfc", "html-template"],
        selector: `${COMPILER_CONFIGURED_PROJECT_ADAPTER} pinned vite.config.mjs enables patterned templates`,
      },
    ],
  });
}

export function loadConfiguredCompilerProjectInputs(
  value: unknown,
  read: (file: string) => Buffer | null,
) {
  validateConfiguredCompilerProjectCase(value);
  return inputs.map(([file, , output], index): ConfiguredCompilerProjectFile => {
    const pinned = value.inputs.files[index]!;
    const bytes = read(`${COMPILER_CONFIGURED_PROJECT_ROOT}/${file}`);
    assert.ok(bytes, `missing pinned configured compiler input: ${file}`);
    assert.equal(hash(bytes), pinned.sha256, `SHA256 mismatch: configured compiler input: ${file}`);
    if (output === "vite.config.mjs") {
      assert.equal(
        bytes.toString("utf8"),
        COMPILER_CONFIGURED_PROJECT_CONFIG,
        "unregistered compiler configuration",
      );
    }
    return { path: output, sha256: pinned.sha256, bytes };
  });
}

export function materializeConfiguredCompilerProjectInputs(
  directory: string,
  files: ConfiguredCompilerProjectFile[],
) {
  assert.deepEqual(
    files.map((file) => file.path),
    inputs.map(([, , output]) => output),
  );
  for (const file of files) {
    assert.equal(
      hash(file.bytes),
      file.sha256,
      `configured input changed before materialization: ${file.path}`,
    );
    if (file.path === "vite.config.mjs") {
      assert.equal(file.bytes.toString("utf8"), COMPILER_CONFIGURED_PROJECT_CONFIG);
    }
  }
  for (const file of files) fs.writeFileSync(path.join(directory, file.path), file.bytes);
}

export function compilerRecipeFailure(
  fixture: { id: string; requiredBuildRecipe?: string },
  observedRecipe: string,
): string | null {
  return fixture.requiredBuildRecipe && fixture.requiredBuildRecipe !== observedRecipe
    ? `${fixture.id} requires source-build recipe ${fixture.requiredBuildRecipe}; observed ${observedRecipe}`
    : null;
}
