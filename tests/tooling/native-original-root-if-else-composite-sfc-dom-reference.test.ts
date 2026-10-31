import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  compiler as actualCompiler,
  runtime,
  hash,
} from "./support/native-selected-sfc-dom-runtime.ts";
import {
  assertRootIfElseCompositeFixture,
  executeRootIfElseCompositeComponent,
  rootIfElseCompositeErrorDetails,
  type RootIfElseCompositeFixture,
  type RootIfElseCompositeMode,
} from "./support/native-original-root-if-else-composite-sfc-dom-runtime.ts";
import {
  compileRootIfElseCompositePrimary,
  typescript as ts,
} from "./support/native-original-root-if-else-composite-sfc-dom-primary.ts";

type ObjectValue = Record<string, unknown>;
function object(value: unknown): asserts value is ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
}
function readObject(raw: string): ObjectValue {
  const value: unknown = JSON.parse(raw);
  object(value);
  return value;
}
const compiler = actualCompiler as typeof import("@vue/compiler-sfc");
const inputRaw = fs.readFileSync(
  new URL(
    "../../crates/vize_atelier_sfc/tests/fixtures/native_original_root_if_else_composite_inputs_vue_3_5_35.json",
    import.meta.url,
  ),
  "utf8",
);
const pack = readObject(inputRaw);
const modes = ["development", "production"] as const;
const selectedMode = process.env.NODE_ENV ?? "development";
assert(selectedMode === "development" || selectedMode === "production");
const mode: RootIfElseCompositeMode = selectedMode;
assert.equal(pack.schema, "vize.native-sfc.original-root-if-else-composite-inputs");
assert.equal(pack.filename, "OriginalRootIfElseComposite雪🌸.vue");
const filename = pack.filename as string;
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 7);
const fixtures: RootIfElseCompositeFixture[] = [];
for (const value of pack.fixtures) {
  assertRootIfElseCompositeFixture(value);
  fixtures.push(value);
}
for (const [index, fixture] of fixtures.slice(0, 5).entries()) {
  const successor = (fixture as RootIfElseCompositeFixture & { successorOf?: unknown }).successorOf;
  object(successor);
  assert.equal(successor.revision, "c0dc33d29c1c162ae5fe428a8e7c61381b94d0d3");
  assert.equal(successor.completeOriginalRetained, true);
  assert.equal(successor.sourceSha256, hash(fixture.source));
  assert.equal(
    successor.path,
    index < 4
      ? "davinci/vize_l2/src/file/region/native/pair/text_tests/refusal.rs"
      : "crates/vize_atelier_sfc/src/native_selected_setup/tests/original_root_if_else_text/refusal.rs",
  );
  assert.equal(
    successor.blob,
    index < 4
      ? "b80941545aefc95bc639429d05d2dc6439102ea0"
      : "429cbb8e65ff442dce0ebe20040dca16faa48777",
  );
}
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 7);
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_CAPTURE;
const requireCapture =
  process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_REQUIRE_CAPTURE === "1";
const captureRaw = capturePath ? fs.readFileSync(capturePath, "utf8") : null;
const captured = captureRaw ? readObject(captureRaw) : null;
const primaryPrefix =
  process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_PRIMARY_CAPTURE;
const runtimePrefix =
  process.env.VIZE_NATIVE_ORIGINAL_ROOT_IF_ELSE_COMPOSITE_SFC_DOM_RUNTIME_CAPTURE;
type NativeRow = {
  id: string;
  mode: string;
  source: string;
  code: string;
  nativeMap: ObjectValue;
  nativeMapRaw: string;
};
const nativeRows = new Map<string, NativeRow>();
const primaryRows = new Map<string, ReturnType<typeof compilePrimary>>();
const primaryAttempts: ObjectValue[] = [];
const runtimeAttempts: ObjectValue[] = [];
const captureObservation: ObjectValue = { state: "not-validated", sourceCapture: captured };

function compilePrimary(fixture: RootIfElseCompositeFixture, attempt: ObjectValue) {
  return compileRootIfElseCompositePrimary(fixture, attempt, filename, mode);
}

test("seven original composite If/Else inputs pin providers and fourteen genuine mode-major Rust rows", () => {
  try {
    assert.equal(compiler.version, "3.5.35");
    assert.equal(runtime.version, "3.5.35");
    assert.equal(ts.version, "6.0.3");
    assert.deepEqual(pack.compiler, { version: compiler.version });
    assert.deepEqual(pack.runtime, { version: runtime.version });
    assert.deepEqual(pack.typescript, { version: ts.version });
    assert.equal(
      pack.qualification,
      "input-only; no native code, source-map or primary capture credit",
    );
    for (const fixture of fixtures) {
      for (const field of [
        "expectedCode",
        "nativeMap",
        "nativeMapRaw",
        "referenceScript",
        "referenceRender",
      ])
        assert.equal(
          Object.hasOwn(fixture, field),
          false,
          "goldens require genuine independent capture",
        );
    }
    if (requireCapture) {
      assert(captured, "hosted acceptance requires genuine source-built Rust output");
      assert(primaryPrefix && runtimePrefix, "both complete evidence prefixes are required");
      assert(
        typeof process.env.GITHUB_SHA === "string" && /^[0-9a-f]{40}$/.test(process.env.GITHUB_SHA),
      );
      assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
    }
    if (!captured) {
      captureObservation.state = "absent-native-not-executed";
      return;
    }
    assert.equal(captured.schema, "vize.native-sfc.original-root-if-else-composite-capture");
    assert.equal(
      captured.adapter,
      "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
    );
    const expected = modes.flatMap((current) =>
      fixtures.map((fixture) => ({ id: fixture.id, mode: current })),
    );
    for (const field of ["fixtures", "custody", "attempts"]) {
      const rows: unknown = captured[field];
      assert(Array.isArray(rows));
      assert.deepEqual(
        rows.map((row: unknown) => {
          object(row);
          return { id: row.id, mode: row.mode };
        }),
        expected,
        `whole ordered mode-major ${field}`,
      );
    }
    const rows = captured.fixtures as unknown[];
    const attempts = captured.attempts as unknown[];
    for (const [index, value] of rows.entries()) {
      object(value);
      const fixture = fixtures[index % fixtures.length];
      assert(fixture);
      assert.equal(value.source, fixture.source);
      assert(typeof value.code === "string" && value.code.length > 0);
      assert(typeof value.nativeMapRaw === "string");
      object(value.nativeMap);
      const map = value.nativeMap;
      assert.deepEqual(JSON.parse(value.nativeMapRaw), map);
      assert.equal(map.version, 3);
      assert.equal(map.file, filename);
      assert.deepEqual(map.sources, [filename]);
      assert.deepEqual(map.sourcesContent, [fixture.source]);
      assert(
        Array.isArray(map.names) && map.names.every((name: unknown) => typeof name === "string"),
      );
      assert.equal(new Set(map.names).size, map.names.length);
      assert(typeof map.mappings === "string" && map.mappings.length > 0);
      const attempt = attempts[index];
      object(attempt);
      assert.equal(attempt.source, fixture.source);
      assert.deepEqual(attempt.mapped, {
        state: "emitted",
        code: value.code,
        nativeMapRaw: value.nativeMapRaw,
        nativeMap: map,
      });
      assert.deepEqual(attempt.plain, {
        state: "emitted",
        code: value.code,
        nativeMapRaw: null,
        nativeMap: null,
      });
      assert.deepEqual(
        attempt.defaultDevelopment,
        value.mode === "development" ? attempt.mapped : null,
      );
      if (value.mode === mode) nativeRows.set(fixture.id, value as NativeRow);
    }
    assert.deepEqual(
      [...nativeRows.keys()],
      fixtures.map((fixture) => fixture.id),
    );
    assert.equal(captured.qualification, "observed-unqualified-current-oracles-pending");
    assert.equal(captured.nativeGoldenComparisonPerformed, false);
    assert.deepEqual(captured.failures, []);
    captureObservation.state = "observed-source-output-unqualified";
  } catch (error) {
    Object.assign(captureObservation, {
      state: "failed",
      error: rootIfElseCompositeErrorDetails(error),
    });
    throw error;
  }
});

for (const fixture of fixtures) {
  test(`${fixture.id}: retain complete official primary output for ${mode}`, () => {
    const attempt: ObjectValue = { id: fixture.id, mode, source: fixture.source };
    primaryAttempts.push(attempt);
    try {
      const primary = compilePrimary(fixture, attempt);
      primaryRows.set(fixture.id, primary);
      Object.assign(attempt, { state: "compiled", primary });
    } catch (error) {
      Object.assign(attempt, { state: "failed", error: rootIfElseCompositeErrorDetails(error) });
      throw error;
    }
  });
}

for (const fixture of fixtures) {
  test(`${fixture.id}: whole native and reference If/Else branch update and unmount in ${mode}`, async (t) => {
    if (!captured && !requireCapture) {
      t.skip("official primary-only tooling; native was not executed and receives no credit");
      return;
    }
    const referenceObservations: unknown[] = [];
    const nativeObservations: unknown[] = [];
    const attempt: ObjectValue = {
      id: fixture.id,
      mode,
      source: fixture.source,
      sourceHash: hash(fixture.source),
      referenceObservations,
      nativeObservations,
    };
    runtimeAttempts.push(attempt);
    try {
      assert(captured, "there is no desired-native fallback");
      const row = nativeRows.get(fixture.id);
      const primary = primaryRows.get(fixture.id);
      Object.assign(attempt, { nativeRow: row, referenceModule: primary?.runtimeModule });
      const failures: { phase: string; error: unknown }[] = [];
      let reference: unknown;
      let native: unknown;
      // Each side is attempted once, even if its peer failed or lacks a module.
      try {
        assert(primary, "whole observed reference module required");
        attempt.referenceCodeHash = hash(primary.runtimeModule);
        reference = await executeRootIfElseCompositeComponent(
          primary.runtimeModule,
          fixture,
          false,
          mode,
          (observation) => referenceObservations.push(observation),
        );
        attempt.reference = reference;
      } catch (error) {
        failures.push({ phase: "reference-attempt", error });
      }
      try {
        assert(row, "whole observed native module required");
        Object.assign(attempt, {
          nativeCodeHash: hash(row.code),
          nativeMapHash: hash(JSON.stringify(row.nativeMap)),
          nativeMapRawHash: hash(row.nativeMapRaw),
        });
        native = await executeRootIfElseCompositeComponent(
          row.code,
          fixture,
          true,
          mode,
          (observation) => nativeObservations.push(observation),
        );
        attempt.native = native;
      } catch (error) {
        failures.push({ phase: "native-attempt", error });
      }
      attempt.failures = failures.map(({ phase, error }) => ({
        phase,
        error: rootIfElseCompositeErrorDetails(error),
      }));
      if (failures.length) throw failures[0]!.error;
      assert.deepEqual(native, reference);
      attempt.state = "matched";
    } catch (error) {
      Object.assign(attempt, { state: "failed", error: rootIfElseCompositeErrorDetails(error) });
      throw error;
    }
  });
}

after(() => {
  const primaryComplete =
    primaryAttempts.length === fixtures.length &&
    primaryAttempts.every((attempt) => attempt.state === "compiled");
  const runtimeMatched =
    runtimeAttempts.length === fixtures.length &&
    runtimeAttempts.every((attempt) => attempt.state === "matched");
  const common = {
    adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
    qualification:
      !captured && primaryComplete && !requireCapture
        ? "official-primary-only-native-not-executed"
        : captureObservation.state === "observed-source-output-unqualified" &&
            primaryComplete &&
            runtimeMatched
          ? "observed-unqualified-current-oracles-pending"
          : "failed-original-observation",
    nativeGoldenComparisonPerformed: false,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    runId: process.env.GITHUB_RUN_ID ?? null,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    mode,
    compiler: pack.compiler,
    runtime: pack.runtime,
    typescript: pack.typescript,
    inputHash: hash(inputRaw),
    inputPack: pack,
    sourceCaptureHash: captureRaw ? hash(captureRaw) : null,
    captureObservation,
    primaryComplete,
    runtimeMatched,
  };
  // Both complete packets are persisted before any final acceptance verdict.
  if (primaryPrefix)
    fs.writeFileSync(
      `${primaryPrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-root-if-else-composite-primary-capture",
          fixtures: primaryAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  if (runtimePrefix)
    fs.writeFileSync(
      `${runtimePrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-root-if-else-composite-runtime-capture",
          host: "vue-createRenderer-custom-host",
          sourceCapture: captured,
          fixtures: runtimeAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  assert.equal(primaryRows.size, fixtures.length, "all whole official primary outputs retained");
  assert(primaryComplete);
  if (requireCapture || captured) {
    assert.equal(captureObservation.state, "observed-source-output-unqualified");
    assert.equal(runtimeAttempts.length, fixtures.length);
    assert(
      runtimeMatched,
      "both original modules must actually mount/update/unmount and wholly agree",
    );
  }
});
