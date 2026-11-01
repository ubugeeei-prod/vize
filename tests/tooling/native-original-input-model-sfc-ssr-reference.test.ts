// Strict current whole-source/primary/SSR comparison to independently reviewed packets.
import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  assertContext,
  assertFixture,
  hash,
  object,
  type ModelSsrContext,
  type ModelSsrFixture,
  type ModelSsrMode,
  type ObjectValue,
} from "./support/native-original-input-model-sfc-ssr-boundary.ts";
import {
  compileInputModelSsrPrimary,
  typescript,
} from "./support/native-original-input-model-sfc-ssr-primary.ts";
import {
  executeInputModelSsr,
  modelSsrError,
} from "./support/native-original-input-model-sfc-ssr-runtime.ts";

const fixtureRoot = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const inputRaw = fs.readFileSync(
  new URL("native_original_input_model_inputs_vue_3_5_35.json", fixtureRoot),
  "utf8",
);
const pack: unknown = JSON.parse(inputRaw);
object(pack);
assert.equal(pack.schema, "vize.native-sfc.original-input-model-inputs");
assert.equal(pack.compiler, "@vue/compiler-sfc@3.5.35");
assert.equal(pack.runtime, "vue@3.5.35");
assert.equal(pack.typescript, "6.0.3");
assert.equal(typescript.version, "6.0.3");
assert(typeof pack.filename === "string");
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 7);
const fixtures: ModelSsrFixture[] = pack.fixtures.map((value: unknown) => {
  assertFixture(value);
  return value;
});
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 7);
const contextRaw = fs.readFileSync(
  new URL("native_original_input_model_ssr_contexts_vue_3_5_35.json", fixtureRoot),
  "utf8",
);
const contextPack: unknown = JSON.parse(contextRaw);
object(contextPack);
assert.equal(contextPack.schema, "vize.native-sfc.original-input-model-ssr-contexts");
assert.equal(contextPack.runtime, "vue@3.5.35");
assert.equal(contextPack.renderer, "@vue/server-renderer@3.5.35");
assert.equal(contextPack.sourceInputHash, hash(inputRaw));
assert(Array.isArray(contextPack.contexts) && contextPack.contexts.length === 8);
const contexts: ModelSsrContext[] = contextPack.contexts.map((value: unknown) => {
  assertContext(value);
  return value;
});
assert.equal(new Set(contexts.map((context) => context.id)).size, 8);
assert(Array.isArray(contextPack.refusals) && contextPack.refusals.length === 12);
const actualMode = process.env.NODE_ENV ?? "development";
assert(actualMode === "development" || actualMode === "production");
const mode: ModelSsrMode = actualMode;
const goldens: unknown = JSON.parse(
  fs.readFileSync(
    new URL("native_original_input_model_ssr_sfc_vue_3_5_35.json", fixtureRoot),
    "utf8",
  ),
);
object(goldens);
assert.equal(goldens.schema, "vize.native-sfc.original-input-model-ssr-complete-goldens");
assert.equal(goldens.inputHash, hash(inputRaw));
assert.equal(goldens.contextHash, hash(contextRaw));
assert.deepEqual(goldens.inputPack, pack);
assert.deepEqual(goldens.contextPack, contextPack);
object(goldens.provenance);
assert.equal(goldens.provenance.independentlyReviewedBeforeFreeze, true);
const expectedNative = goldens.native;
const expectedPrimary = goldens.primary;
const expectedRuntime = goldens.runtimeResults;
object(expectedNative);
object(expectedPrimary);
object(expectedRuntime);
const expectedPrimaryRows = expectedPrimary[mode];
const expectedRuntimeRows = expectedRuntime[mode];
assert(Array.isArray(expectedPrimaryRows) && expectedPrimaryRows.length === 7);
assert(Array.isArray(expectedRuntimeRows) && expectedRuntimeRows.length === 7);
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_REQUIRE_CAPTURE === "1";
const primaryPrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_SSR_RUNTIME_CAPTURE;
if (requireCapture) {
  assert(capturePath && primaryPrefix && runtimePrefix);
  assert(
    typeof process.env.GITHUB_SHA === "string" && /^[0-9a-f]{40}$/.test(process.env.GITHUB_SHA),
  );
}
let captured: unknown;
let captureRaw: string | null = null;
const captureObservation: ObjectValue = { state: "absent" };
if (capturePath) {
  try {
    captureRaw = fs.readFileSync(capturePath, "utf8");
    captured = JSON.parse(captureRaw) as unknown;
    captureObservation.state = "read";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelSsrError(error) });
  }
}
type NativeRow = {
  id: string;
  source: string;
  code: string;
  nativeMapRaw: string;
  nativeMap: ObjectValue;
};
const nativeRows = new Map<string, NativeRow>();
const primaryRows = new Map<string, ReturnType<typeof compileInputModelSsrPrimary>>();
const primaryAttempts: ObjectValue[] = [];
const runtimeAttempts: ObjectValue[] = [];
let captureComplete = false;
let nativeGoldenComparisonPerformed = false;
let nativeGoldensMatched = false;
let primaryGoldenMatches = 0;
let runtimeGoldenMatches = 0;
let runtimeMatches = 0;
let failed = false;
let firstFailure: unknown;
function remember(error: unknown) {
  if (!failed) {
    failed = true;
    firstFailure = error;
  }
}
function assertNativeRow(value: unknown): asserts value is NativeRow {
  object(value);
  for (const key of ["id", "source", "code", "nativeMapRaw"])
    assert(typeof value[key] === "string" && value[key].length > 0);
  object(value.nativeMap);
}

test("original SSR Model capture retains seven real owners and all twelve outside refusals", () => {
  if (!capturePath && !requireCapture) return;
  try {
    object(captured);
    assert.equal(captured.schema, "vize.native-sfc.original-input-model-ssr-capture");
    assert.equal(captured.adapter, "vize_atelier_sfc::compile_native_setup_ssr_sfc");
    assert.equal(captured.qualification, "current-complete-native-goldens-matched");
    assert.equal(captured.nativeGoldenComparisonPerformed, true);
    assert.equal(captured.nativeGoldensMatched, true);
    assert.deepEqual(captured.failures, []);
    assert.deepEqual(captured.inputPack, pack);
    if (requireCapture) assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
    const identities = fixtures.map(({ id, source }) => ({ id, source }));
    for (const field of ["fixtures", "custody", "attempts"]) {
      const rows: unknown = captured[field];
      assert(Array.isArray(rows));
      assert.deepEqual(
        rows.map((value: unknown) => {
          object(value);
          return { id: value.id, source: value.source };
        }),
        identities,
      );
    }
    assert(Array.isArray(captured.fixtures));
    for (const row of captured.fixtures) {
      assertNativeRow(row);
      assert.deepEqual(JSON.parse(row.nativeMapRaw), row.nativeMap);
      assert.equal(row.nativeMap.version, 3);
      assert.equal(row.nativeMap.file, pack.filename);
      assert.deepEqual(row.nativeMap.sources, [pack.filename]);
      assert.deepEqual(row.nativeMap.sourcesContent, [row.source]);
      assert(Array.isArray(row.nativeMap.names));
      assert(typeof row.nativeMap.mappings === "string" && row.nativeMap.mappings.length > 0);
      nativeRows.set(row.id, row);
    }
    assert.equal(nativeRows.size, 7);
    assert(Array.isArray(captured.attempts));
    for (const row of captured.attempts) {
      object(row);
      object(row.mapped);
      object(row.plain);
      object(row.defaultOptions);
      assert.equal(row.mapped.state, "emitted");
      assert.equal(row.plain.state, "emitted");
      assert.equal(row.defaultOptions.state, "emitted");
      assert.equal(row.mapped.code, row.plain.code);
      assert.deepEqual(row.plain, row.defaultOptions);
      assert.equal(row.plain.nativeMapRaw, null);
      assert.equal(row.plain.nativeMap, null);
      assert(typeof row.id === "string");
      const actual = nativeRows.get(row.id);
      assert(actual);
      assert.equal(row.mapped.code, actual.code);
      assert.equal(row.mapped.nativeMapRaw, actual.nativeMapRaw);
      assert.deepEqual(row.mapped.nativeMap, actual.nativeMap);
    }
    assert(Array.isArray(captured.refusals));
    assert.deepEqual(
      captured.refusals.map((row: unknown) => {
        object(row);
        object(row.result);
        assert.equal(row.result.state, "refused");
        assert(typeof row.result.error === "string" && row.result.error.length > 0);
        return { id: row.id, source: row.source };
      }),
      contextPack.refusals,
    );
    nativeGoldenComparisonPerformed = true;
    for (const field of ["fixtures", "custody", "attempts", "refusals"])
      assert.deepEqual(
        captured[field],
        expectedNative[field],
        `whole ordered current native ${field}`,
      );
    nativeGoldensMatched = true;
    captureComplete = true;
    captureObservation.state = "complete-current-native-goldens-matched";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelSsrError(error) });
    remember(error);
    throw error;
  }
});

for (const fixture of fixtures) {
  test(`official original SSR Model primary ${fixture.id}/${mode}`, () => {
    const attempt: ObjectValue = {
      id: fixture.id,
      mode,
      source: fixture.source,
      state: "attempting",
    };
    primaryAttempts.push(attempt);
    try {
      assert(typeof pack.filename === "string");
      const primary = compileInputModelSsrPrimary(fixture, attempt, pack.filename, mode);
      attempt.result = primary;
      attempt.state = "observed-unqualified";
      primaryRows.set(fixture.id, primary);
      assert.deepEqual(
        primary,
        expectedPrimaryRows[fixtures.indexOf(fixture)],
        "whole official SSR primary",
      );
      primaryGoldenMatches += 1;
    } catch (error) {
      Object.assign(attempt, { state: "failed", error: modelSsrError(error) });
      remember(error);
      throw error;
    }
  });
  test(
    `actual native/reference SSR Model contexts ${fixture.id}/${mode}`,
    {
      skip: !capturePath && !requireCapture,
    },
    async () => {
      const reference: ObjectValue = { state: "attempting" };
      const native: ObjectValue = { state: "attempting" };
      const attempt: ObjectValue = {
        id: fixture.id,
        mode,
        source: fixture.source,
        state: "attempting",
        reference,
        native,
      };
      runtimeAttempts.push(attempt);
      let pairFailed = false;
      let pairFailure: unknown;
      let referenceOutcome: unknown;
      let nativeOutcome: unknown;
      try {
        const primary = primaryRows.get(fixture.id);
        assert(primary, "genuine official primary output required");
        reference.module = primary.runtimeModule;
        referenceOutcome = await executeInputModelSsr(
          primary.runtimeModule,
          fixture,
          contexts,
          reference,
        );
        Object.assign(reference, { state: "observed-unqualified", outcome: referenceOutcome });
      } catch (error) {
        Object.assign(reference, { state: "failed", error: modelSsrError(error) });
        pairFailed = true;
        pairFailure = error;
      }
      try {
        const actual = nativeRows.get(fixture.id);
        assert(actual, "genuine source-built native module required");
        native.module = actual.code;
        nativeOutcome = await executeInputModelSsr(actual.code, fixture, contexts, native);
        Object.assign(native, { state: "observed-unqualified", outcome: nativeOutcome });
      } catch (error) {
        Object.assign(native, { state: "failed", error: modelSsrError(error) });
        if (!pairFailed) {
          pairFailed = true;
          pairFailure = error;
        }
      }
      try {
        if (pairFailed) throw pairFailure;
        assert.deepEqual(
          nativeOutcome,
          referenceOutcome,
          "whole actual SSR outcome, no normalization",
        );
        runtimeMatches += 1;
        // Preserve the reviewed row's original observation label; current
        // qualification is derived only from all mandatory comparisons below.
        attempt.state = "matched-observation-unqualified";
        assert.deepEqual(
          attempt,
          expectedRuntimeRows[fixtures.indexOf(fixture)],
          "whole actual SSR attempt/observations/outcomes",
        );
        runtimeGoldenMatches += 1;
      } catch (error) {
        Object.assign(attempt, { state: "failed", error: modelSsrError(error) });
        remember(error);
        throw error;
      }
    },
  );
}

after(() => {
  const primaryComplete = primaryRows.size === 7 && primaryGoldenMatches === 7;
  const runtimeComplete =
    captureComplete &&
    nativeGoldensMatched &&
    primaryComplete &&
    runtimeMatches === 7 &&
    runtimeGoldenMatches === 7 &&
    !failed;
  const qualified = requireCapture && runtimeComplete;
  const common = {
    schema: "vize.native-sfc.original-input-model-ssr-primary",
    mode,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    inputHash: hash(inputRaw),
    contextHash: hash(contextRaw),
    inputPack: pack,
    contextPack,
    qualification: qualified
      ? "complete-current-oracles-matched"
      : failed
        ? "failed-current-runtime-comparison"
        : "partial-current-observation-unqualified",
    nativeGoldenComparisonPerformed,
    nativeGoldensMatched,
    primaryGoldenMatches,
    runtimeGoldenMatches,
    wholePacketQualification: qualified,
    captureComplete,
    primaryComplete,
    runtimeComplete,
    runtimeMatches,
    finalFailure: failed ? modelSsrError(firstFailure) : null,
  };
  const primary = { ...common, fixtures: [...primaryRows.values()], attempts: primaryAttempts };
  const runtime = {
    ...common,
    schema: "vize.native-sfc.original-input-model-ssr-runtime",
    sourceCapture: captured ?? null,
    sourceCaptureRawHash: captureRaw ? hash(captureRaw) : null,
    captureObservation,
    fixtures: runtimeAttempts,
  };
  let writeFailed = false;
  let writeFailure: unknown;
  for (const [prefix, value] of [
    [primaryPrefix, primary],
    [runtimePrefix, runtime],
  ] as const) {
    if (!prefix) continue;
    try {
      fs.writeFileSync(`${prefix}.${mode}.json`, JSON.stringify(value, null, 2) + "\n");
    } catch (error) {
      if (!writeFailed) {
        writeFailed = true;
        writeFailure = error;
      }
    }
  }
  if (writeFailed) throw writeFailure;
  if (requireCapture)
    assert(
      primaryComplete && runtimeComplete,
      "all current whole native/primary/runtime comparisons must pass",
    );
});
