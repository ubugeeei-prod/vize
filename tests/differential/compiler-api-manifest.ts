import assert from "node:assert/strict";
import { loadProductManifest, readPinnedArtifact, sha256 } from "./harness.mjs";
import type { ObserverSpec } from "./observer-build.ts";
import type {
  CompilerFixture,
  CompilerReference,
  CompilerTarget,
  HistoricalCompilerPacket,
  JsonObject,
  JsonValue,
  PinnedSfcOptions,
} from "./compiler-api-types.ts";

const NATIVE_REASON = "whole-product native compiler path unavailable";
const ARCHIVED_SFC_OPTIONS_WITH_EXTERNAL_WITNESS = new Set([
  "dual-script-inline-type-only-imports",
  "separate-template-authored-render-binding",
]);
const COMBINED_SFC_CAPTURE_NAMES = new Set([
  "counter-source-map",
  "props-default-type-diagnostic",
]);
const names = {
  dom: [
    "imported-component-before-prop",
    "scoped-css-v-bind",
    "dynamic-loop-ref-for",
    "computed-component-unref",
    "custom-directive-child-patch",
    "dual-script-inline-type-only-imports",
    "separate-template-authored-render-binding",
    "invalid-template-expression-diagnostic",
    "counter-source-map",
    "props-default-type-diagnostic",
  ],
  ssr: [
    "textarea-model",
    "slot-fallback-vnode",
    "component-slot-props",
    "named-scoped-slot",
    "component-lone-spread",
  ],
  vapor: [
    "once-directive",
    "insertion-placeholder",
    "root-document-order",
    "hydration-sibling-element",
    "mounted-control-slot",
  ],
};

function observer(target: CompilerTarget): ObserverSpec {
  const product = target === "dom" ? "sfc" : target;
  const packageName = `vize_atelier_${product}`;
  const exampleName = `${product}_fix_history_observer`;
  return {
    product: "compiler",
    packageName,
    exampleName,
    sourcePath: `crates/${packageName}/examples/${exampleName}/main.rs`,
    probes: [["--contract"]],
    expectedFeatures:
      target === "dom" ? ["default", "native"] : target === "ssr" ? ["default"] : [],
  };
}

export const COMPILER_OBSERVER_SPEC = observer("dom");
export const COMPILER_TARGET_CONTRACTS = {
  dom: {
    ids: names.dom.map((name) => `compiler/sfc/${name}`),
    fields: ["code", "css", "map", "errors", "warnings", "bindings", "macroArtifacts"],
    prefix: "crates/vize_atelier_sfc/tests/fixtures/fix-history",
    name: "sfc",
    packetSchema: "vize.compiler.public-sfc-observation",
    capture: null,
    spec: COMPILER_OBSERVER_SPEC,
  },
  ssr: {
    ids: names.ssr.map((name) => `compiler/ssr/${name}`),
    fields: ["code", "preamble", "map", "diagnostics"],
    prefix: "crates/vize_atelier_ssr/tests/fixtures/fix-history-next",
    name: "ssr",
    packetSchema: "vize.compiler.public-target-observation",
    capture: "capture-c2451",
    spec: observer("ssr"),
  },
  vapor: {
    ids: names.vapor.map((name) => `compiler/vapor/${name}`),
    fields: ["code", "templates", "map", "errorMessages"],
    prefix: "crates/vize_atelier_vapor/tests/fixtures/fix-history-next",
    name: "vapor",
    packetSchema: "vize.compiler.public-target-observation",
    capture: "capture-b11eb",
    spec: observer("vapor"),
  },
};

export function originalSfcOptions(pinned: PinnedSfcOptions) {
  const effective = pinned.effectiveAdapterInputs;
  return {
    descriptorParseOptions: pinned.descriptorParseOptions,
    compileOptions: pinned.compileOptions,
    effectiveAdapterInputs: {
      templateSyntax: effective.templateSyntax,
      customElements: { construction: effective.customElements.construction },
      codegen: effective.codegen,
      scriptOutput: effective.scriptOutput,
      experimental: effective.experimental,
    },
  };
}

export function validateCompilerOutput(result: JsonObject, target: CompilerTarget) {
  assert(result && typeof result === "object" && !Array.isArray(result));
  if (target === "dom") {
    const keys = Object.keys(result);
    assert(keys.length === 1 && ["Ok", "Err"].includes(keys[0]), "complete public Result required");
    if (keys[0] === "Err") {
      const error = result.Err as JsonObject;
      assert.deepEqual(Object.keys(error).sort(), ["code", "loc", "message"]);
      assert.equal(typeof error.message, "string");
      return;
    }
    result = result.Ok as JsonObject;
  }
  assert.deepEqual(
    Object.keys(result).sort(),
    [...COMPILER_TARGET_CONTRACTS[target].fields].sort(),
  );
  assert.equal(typeof result.code, "string");
  if (target === "dom") {
    assert(result.css === null || typeof result.css === "string");
    for (const field of ["errors", "warnings", "macroArtifacts"])
      assert(Array.isArray(result[field]));
  } else {
    assert(result.map === null || typeof result.map === "string");
    if (target === "ssr") {
      assert.equal(typeof result.preamble, "string");
      assert(Array.isArray(result.diagnostics));
    } else {
      assert(
        Array.isArray(result.templates) &&
          result.templates.every((template: unknown) => typeof template === "string"),
      );
      assert(Array.isArray(result.errorMessages));
    }
  }
}

export function loadCompilerApiManifest(manifestPath: string, repoRoot: string) {
  const loaded = loadProductManifest(manifestPath, "compiler");
  const fixtures = loaded.cases as CompilerFixture[];
  assert.equal(loaded.manifest.artifactRoot, "repository");
  const target = fixtures[0].targets[0];
  const contract = COMPILER_TARGET_CONTRACTS[target];
  assert(contract, "only the three closed compiler target cohorts are registered");
  assert.deepEqual(
    fixtures.map((fixture) => fixture.id),
    contract.ids,
  );
  return {
    ...loaded,
    target,
    contract,
    cases: fixtures.map((fixture) => {
      const name = fixture.id.split("/").at(-1);
      assert(name, "compiler fixture id must include a case name");
      assert.equal(fixture.state, "active");
      assert.deepEqual(fixture.targets, [target]);
      assert.deepEqual(fixture.comparison.requiredFacets, contract.fields);
      assert.equal(
        fixture.comparison.contract,
        `compiler-complete-public-${contract.name}-result-v1`,
      );
      assert.equal(fixture.adapters.legacy, `compiler-public-${contract.name}-api-v1`);
      assert.equal(fixture.adapters.native, null);
      assert.equal(fixture.adapters.reasons.native, NATIVE_REASON);
      assert.equal(fixture.inputs.files.length, 1);
      assert.equal(fixture.inputs.files[0].path, `${contract.prefix}/${name}.input.txt`);
      assert.equal(fixture.inputs.files[0].role, "entry");
      assert.equal(fixture.reference.path, `${contract.prefix}/${name}.expected.json`);
      const input = readPinnedArtifact(repoRoot, fixture.inputs.files[0]);
      const reference: JsonObject = JSON.parse(
        readPinnedArtifact(repoRoot, fixture.reference).toString(),
      );
      const historical: HistoricalCompilerPacket = JSON.parse(
        readPinnedArtifact(repoRoot, fixture.historicalObservation).toString(),
      );
      let originalOptions: JsonValue, expected: JsonObject, entrypoint: string;
      if (target === "dom") {
        if (COMBINED_SFC_CAPTURE_NAMES.has(name)) {
          assert.equal(fixture.options.path, fixture.reference.path);
          assert.equal(fixture.options.sha256, fixture.reference.sha256);
          assert.equal(
            fixture.historicalObservation.path,
            `${contract.prefix}/map-diagnostic-capture-72293/first.stdout.json`,
          );
          const combined = reference as JsonObject & {
            id: string;
            entrypoint: string;
            options: JsonObject;
            compileResult: JsonObject;
          };
          assert.equal(combined.id, name);
          assert(historical.cases, "historical SFC cases are required");
          const row = historical.cases.find((candidate) => candidate.id === name);
          assert(row, "historical SFC case is required");
          assert.equal(row.entrypoint, combined.entrypoint);
          assert.deepEqual(row.options, combined.options);
          assert.deepEqual(row.compileResult, combined.compileResult);
          const adapter = combined.options.adapterContext as JsonObject;
          const experimental = adapter.experimental as JsonObject;
          originalOptions = {
            descriptorParseOptions: combined.options.descriptorParseOptions,
            compileOptions: combined.options.compileOptions,
            effectiveAdapterInputs: {
              templateSyntax: adapter.templateSyntax,
              customElements: { construction: "CustomElementMatcher::default()" },
              codegen: adapter.codegen,
              scriptOutput: adapter.scriptOutput,
              experimental: { self_component: experimental.selfComponent },
            },
          };
          expected = combined.compileResult;
          entrypoint = combined.entrypoint;
          validateCompilerOutput(expected, target);
          return { ...fixture, input, originalOptions, expected, entrypoint };
        }
        assert.equal(fixture.options.path, `${contract.prefix}/${name}.options.json`);
        assert.equal(
          fixture.historicalObservation.path,
          `${contract.prefix}/${name}.observation.json`,
        );
        const options: PinnedSfcOptions = JSON.parse(
          readPinnedArtifact(repoRoot, fixture.options).toString(),
        );
        if (options.authoredWitness)
          assert.equal(options.authoredWitness.inputSha256, sha256(input));
        else
          assert(
            ARCHIVED_SFC_OPTIONS_WITH_EXTERNAL_WITNESS.has(name),
            `missing authored witness: ${name}`,
          );
        assert.equal(historical.profile, name);
        assert.deepEqual(historical.result, reference);
        originalOptions = originalSfcOptions(options);
        expected = reference;
        entrypoint = options.entrypoint;
      } else {
        const targetReference = reference as JsonObject & CompilerReference;
        assert.equal(fixture.options.path, fixture.reference.path);
        assert.equal(fixture.options.sha256, fixture.reference.sha256);
        assert.equal(
          fixture.historicalObservation.path,
          `${contract.prefix}/${contract.capture}/first.stdout.json`,
        );
        assert(historical.cases, "historical compiler cases are required");
        assert.deepEqual(
          historical.cases.map((row) => row.id),
          names[target],
        );
        const row = historical.cases.find((row) => row.id === name);
        assert(row, "historical compiler case is required");
        assert.deepEqual(row.output, targetReference.output);
        assert.deepEqual(
          target === "ssr" ? historical.options : row.options,
          targetReference.options,
        );
        originalOptions = targetReference.options;
        expected = targetReference.output;
        entrypoint = targetReference.options.entrypoint as string;
        if (fixture.currentReference) {
          assert.equal(fixture.currentReference.supersedesSha256, fixture.reference.sha256);
          assert.match(
            fixture.currentReference.issue,
            /^https:\/\/github\.com\/ubugeeei-prod\/vize\/issues\/[1-9]\d*$/u,
          );
          const current: CompilerReference = JSON.parse(
            readPinnedArtifact(repoRoot, fixture.currentReference).toString(),
          );
          assert.deepEqual(current.options, targetReference.options);
          validateCompilerOutput(current.output, target);
          expected = current.output;
        }
      }
      validateCompilerOutput(expected, target);
      return { ...fixture, input, originalOptions, expected, entrypoint };
    }),
  };
}
