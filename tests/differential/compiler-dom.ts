import assert from "node:assert/strict";
import { BUILD_RECIPE } from "./build-receipt.ts";

export const COMPILER_DOM_ID = "compiler/dom/typed-handler-runtime-spelling-11815";
export const COMPILER_DOM_ROOT = "typed-handler-runtime-spelling-11815";
export const COMPILER_DOM_ADAPTER = "compiler-cli-dom-11815";
export const COMPILER_DOM_CONTRACT = "compiler-dom-module-bytes-11815";
export const COMPILER_DOM_ARGV = [
  "build",
  "Layout.vue",
  "--format",
  "json",
  "--output",
  "out",
  "--no-config",
];

// The one reviewed case is not a general-purpose target or argument registry.
const INPUT_SHA256 = "f96447efcb9f0d9336a08fadb27bafb922c27a250b616c038aed98b864ebdee8";
const REFERENCE_SHA256 = "c3da518002b36d5396993d6963a5f1d4893502c041b9cc947ee37f186d6aebb7";

const ORACLE_SOURCE =
  "Independent public @vizejs/native 0.680.0 untyped control compiled as Layout.vue with script lang=ts, ssr=false and sourceMap=false. The runtime onPick declaration, script, filename, and module wrapper are identical; only the template handler type assertion is absent from the control. Complete module bytes are frozen without normalization. The source Rust handler unit consumes this same typed input; source CLI equivalence remains an Actions qualification.";

interface CommandFixture {
  readonly id?: unknown;
  readonly targets?: unknown;
  readonly argv?: unknown;
  readonly dialectInput?: unknown;
  readonly inputs?: { readonly dialectInput?: unknown };
}

export function compilerDomArgv(fixture: CommandFixture): string[] {
  assert.equal(fixture.id, COMPILER_DOM_ID, "unregistered DOM compiler case");
  assert.deepEqual(fixture.targets, ["dom"], "unregistered compiler target");
  assert.equal(fixture.argv, undefined, "compiler arguments cannot be overridden");
  assert.equal(fixture.dialectInput, undefined, "DOM dialect override is not registered");
  assert.equal(fixture.inputs?.dialectInput, undefined, "DOM dialect input is not registered");
  return [...COMPILER_DOM_ARGV];
}

function record(value: unknown, label: string): Record<string, unknown> {
  assert.ok(value !== null && typeof value === "object" && !Array.isArray(value), label);
  return value as Record<string, unknown>;
}

export function validateCompilerDomCase(value: unknown): void {
  const fixture = record(value, "DOM compiler fixture");
  assert.deepEqual(Object.keys(fixture).sort(), [
    "adapters",
    "comparison",
    "dialectCoverage",
    "id",
    "inputs",
    "reference",
    "state",
    "targets",
  ]);
  compilerDomArgv(fixture);
  assert.equal(fixture.state, "active");
  const inputs = record(fixture.inputs, "DOM authored input");
  assert.deepEqual(Object.keys(inputs).sort(), ["files", "root"]);
  assert.equal(inputs.root, COMPILER_DOM_ROOT);
  assert.deepEqual(inputs.files, [{ path: "Layout.vue.txt", role: "entry", sha256: INPUT_SHA256 }]);
  const adapters = record(fixture.adapters, "DOM source adapter");
  assert.deepEqual(Object.keys(adapters).sort(), ["legacy", "legacyRecipe", "native", "reasons"]);
  assert.equal(adapters.legacy, COMPILER_DOM_ADAPTER);
  assert.equal(adapters.legacyRecipe, BUILD_RECIPE, "DOM requires the current source recipe");
  assert.equal(adapters.native, null);
  const reasons = record(adapters.reasons, "DOM native unsupported reason");
  assert.deepEqual(Object.keys(reasons), ["native"]);
  assert.equal(typeof reasons.native, "string");
  assert.ok((reasons.native as string).length > 0);
  const reference = record(fixture.reference, "DOM whole-module reference");
  assert.deepEqual(Object.keys(reference).sort(), ["path", "sha256", "source"]);
  assert.equal(reference.path, `${COMPILER_DOM_ROOT}/module.expected.txt`);
  assert.equal(reference.sha256, REFERENCE_SHA256, "independent DOM reference identity");
  assert.equal(reference.source, ORACLE_SOURCE, "independent DOM source authority");
  assert.equal(reference.cssPath, undefined);
  assert.equal(reference.cssSha256, undefined);
  assert.deepEqual(fixture.dialectCoverage, {
    state: "source-inspected",
    registryRef: null,
    evidence: [
      {
        kind: "source-inspection",
        dialects: ["vue3-sfc", "html-template", "ts"],
        selector: `${COMPILER_DOM_ADAPTER} default Vue 3 dialect`,
      },
    ],
  });
  assert.deepEqual(fixture.comparison, {
    contract: COMPILER_DOM_CONTRACT,
    requiredFacets: ["code", "css", "errors", "warnings", "macro_artifacts"],
    scriptLang: "ts",
  });
}
