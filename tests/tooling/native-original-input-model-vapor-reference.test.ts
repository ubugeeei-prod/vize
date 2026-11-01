// First whole original-input Vapor observations; no historical native golden is loaded.
import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { after, test } from "node:test";
import {
  assertModelFixture,
  hash,
  object,
  type ModelVaporFixture,
  type ModelVaporMode,
  type ObjectValue,
} from "./support/native-original-input-model-vapor-boundary.ts";
import {
  compileInputModelVaporPrimary,
  typescript,
} from "./support/native-original-input-model-vapor-primary.ts";
import {
  executeInputModelVapor,
  modelVaporError,
} from "./support/native-original-input-model-vapor-runtime.ts";

const fixtureRoot = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const inputRaw = fs.readFileSync(
  new URL("native_original_input_model_inputs_vue_3_5_35.json", fixtureRoot),
  "utf8",
);
const pack: unknown = JSON.parse(inputRaw);
object(pack);
assert.equal(pack.schema, "vize.native-sfc.original-input-model-inputs");
// These version labels identify the unchanged input origin, not this new target.
assert.equal(pack.compiler, "@vue/compiler-sfc@3.5.35");
assert.equal(pack.runtime, "vue@3.5.35");
assert.equal(pack.dom, "happy-dom@21.0.4");
assert.equal(pack.typescript, "6.0.3");
assert.equal(typescript.version, "6.0.3");
assert.equal(pack.filename, "OriginalInputModel雪🌸.vue");
assert.equal(pack.qualification, "new-original-inputs-not-historical-native-evidence");
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 7);
const fixtures: ModelVaporFixture[] = pack.fixtures.map((value: unknown) => {
  assertModelFixture(value);
  return value;
});
assert.deepEqual(
  fixtures.map((row) => row.id),
  [
    "let-string",
    "var-number",
    "let-boolean",
    "var-null",
    "let-bigint",
    "ts-unicode-crlf",
    "unquoted-void",
  ],
);
const contextRaw = fs.readFileSync(
  new URL("native_original_input_model_ssr_contexts_vue_3_5_35.json", fixtureRoot),
  "utf8",
);
const contextPack: unknown = JSON.parse(contextRaw);
object(contextPack);
assert.equal(contextPack.schema, "vize.native-sfc.original-input-model-ssr-contexts");
assert.equal(contextPack.sourceInputHash, hash(inputRaw));
assert(Array.isArray(contextPack.refusals) && contextPack.refusals.length === 12);
const rawMode = process.env.NODE_ENV ?? "development";
assert(rawMode === "development" || rawMode === "production");
const mode: ModelVaporMode = rawMode;
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_REQUIRE_CAPTURE === "1";
const primaryPrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_VAPOR_RUNTIME_CAPTURE;
const captureRequested = Boolean(capturePath || requireCapture);
let captureRaw: string | null = null;
let captured: unknown;
const captureObservation: ObjectValue = { state: "absent" };
if (capturePath) {
  try {
    captureRaw = fs.readFileSync(capturePath, "utf8");
    captured = JSON.parse(captureRaw) as unknown;
    captureObservation.state = "read";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelVaporError(error) });
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
const primaryRows = new Map<string, ReturnType<typeof compileInputModelVaporPrimary>>();
const primaryAttempts: ObjectValue[] = [],
  runtimeAttempts: ObjectValue[] = [];
let captureComplete = false;
let failed = false;
let firstFailure: unknown;
function remember(error: unknown) {
  if (!failed) {
    failed = true;
    firstFailure = error;
  }
}

test(
  "Vapor capture preserves all seven original owners, attempts and twelve actual refusal stages",
  { skip: !captureRequested },
  () => {
    try {
      if (requireCapture) {
        assert(capturePath && primaryPrefix && runtimePrefix);
        assert(
          typeof process.env.GITHUB_SHA === "string" &&
            /^[0-9a-f]{40}$/.test(process.env.GITHUB_SHA),
        );
      }
      object(captured);
      assert.equal(captured.schema, "vize.native-vapor.original-input-model-capture");
      assert.equal(captured.adapter, "compile_native_vapor_setup_sfc");
      assert.equal(captured.inputHash, hash(inputRaw));
      assert.deepEqual(captured.inputPack, pack);
      assert.deepEqual(captured.target, {
        compiler: "@vue/compiler-sfc@3.6.0-rc.9",
        runtime: "vue@3.6.0-rc.9",
      });
      assert.equal(captured.qualification, "observed-unqualified-current-oracles-pending");
      assert.equal(captured.nativeGoldenComparisonPerformed, false);
      assert.equal(captured.wholePacketQualification, false);
      assert.deepEqual(captured.failures, []);
      if (requireCapture) assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
      const identities = fixtures.map(({ id, source }) => ({ id, source }));
      for (const field of ["fixtures", "custody", "attempts"]) {
        const rows: unknown = captured[field];
        assert(Array.isArray(rows));
        assert.deepEqual(
          rows.map((row: unknown) => {
            object(row);
            return { id: row.id, source: row.source };
          }),
          identities,
          field,
        );
      }
      assert(Array.isArray(captured.fixtures));
      for (const row of captured.fixtures) {
        object(row);
        for (const field of ["id", "source", "code", "nativeMapRaw"])
          assert(typeof row[field] === "string" && row[field].length > 0);
        object(row.nativeMap);
        assert.deepEqual(JSON.parse(String(row.nativeMapRaw)), row.nativeMap);
        assert.equal(row.nativeMap.version, 3);
        assert.equal(row.nativeMap.file, pack.filename);
        assert.deepEqual(row.nativeMap.sources, [pack.filename]);
        assert.deepEqual(row.nativeMap.sourcesContent, [row.source]);
        assert(Array.isArray(row.nativeMap.names));
        assert(typeof row.nativeMap.mappings === "string" && row.nativeMap.mappings.length > 0);
        nativeRows.set(String(row.id), row as NativeRow);
      }
      assert.equal(nativeRows.size, 7);
      assert(Array.isArray(captured.attempts));
      for (const attempt of captured.attempts) {
        object(attempt);
        object(attempt.mapped);
        object(attempt.plain);
        object(attempt.defaultOptions);
        assert.equal(attempt.mapped.state, "emitted");
        assert.equal(attempt.plain.state, "emitted");
        assert.equal(attempt.defaultOptions.state, "emitted");
        assert.equal(attempt.mapped.code, attempt.plain.code);
        assert.deepEqual(attempt.plain, attempt.defaultOptions);
        assert.equal(attempt.plain.nativeMap, null);
        assert.equal(attempt.plain.nativeMapRaw, null);
        const row = nativeRows.get(String(attempt.id));
        assert(row);
        assert.equal(attempt.mapped.code, row.code);
        assert.equal(attempt.mapped.nativeMapRaw, row.nativeMapRaw);
        assert.deepEqual(attempt.mapped.nativeMap, row.nativeMap);
      }
      assert(Array.isArray(captured.refusals));
      assert.deepEqual(
        captured.refusals.map((row: unknown) => {
          object(row);
          object(row.result);
          assert.equal(row.result.state, "refused");
          assert(typeof row.result.error === "string" && row.result.error.length > 0);
          assert(
            ["source", "vapor", "analysis", "orchestration"].includes(String(row.result.stage)),
          );
          return { id: row.id, source: row.source };
        }),
        contextPack.refusals,
      );
      captureComplete = true;
      captureObservation.state = "complete-first-observation-no-native-golden-comparison";
    } catch (error) {
      Object.assign(captureObservation, { state: "failed", error: modelVaporError(error) });
      remember(error);
      throw error;
    }
  },
);

for (const fixture of fixtures) {
  test(`official original Vapor Model primary ${fixture.id}/${mode}`, () => {
    const attempt: ObjectValue = {
      id: fixture.id,
      source: fixture.source,
      mode,
      state: "attempting",
    };
    primaryAttempts.push(attempt);
    try {
      assert(typeof pack.filename === "string");
      const primary = compileInputModelVaporPrimary(fixture, attempt, pack.filename, mode);
      attempt.result = primary;
      primaryRows.set(fixture.id, primary);
      assert.equal(primary.id, fixture.id);
      assert.equal(primary.source, fixture.source);
      assert.equal(primary.sourceHash, hash(fixture.source));
      assert.equal(primary.compiler, "@vue/compiler-sfc@3.6.0-rc.9");
      assert.equal(primary.runtime, "vue@3.6.0-rc.9");
      assert.equal(primary.typescript, "6.0.3");
      attempt.state = "observed-unqualified";
    } catch (error) {
      Object.assign(attempt, { state: "failed", error: modelVaporError(error) });
      remember(error);
      throw error;
    }
  });
  test(
    `actual native/official Vapor Model DOM and diagnostic actors ${fixture.id}/${mode}`,
    { skip: !captureRequested },
    async () => {
      const attempt: ObjectValue = {
        id: fixture.id,
        source: fixture.source,
        mode,
        state: "attempting",
      };
      runtimeAttempts.push(attempt);
      const actors: ObjectValue[] = [];
      const actorFailures: { actor: string; error: unknown }[] = [];
      attempt.actors = actors;
      let official: unknown, native: unknown;
      // Each actor is independently attempted once even when its peer fails.
      for (const actor of ["official", "native"] as const) {
        const observed: ObjectValue = { actor, state: "attempting" };
        actors.push(observed);
        try {
          const primary = primaryRows.get(fixture.id);
          const row = nativeRows.get(fixture.id);
          observed.nativeRow = row ?? null;
          observed.primary = primary ?? null;
          const code = actor === "official" ? primary?.runtimeModule : row?.code;
          assert(
            typeof code === "string" && code.length > 0,
            `actual ${actor} whole module required`,
          );
          observed.code = code;
          observed.codeHash = hash(code);
          const result = await executeInputModelVapor(code, fixture, mode, observed);
          observed.result = result;
          observed.state = "observed-unqualified";
          if (actor === "official") official = result;
          else native = result;
        } catch (error) {
          Object.assign(observed, { state: "failed", error: modelVaporError(error) });
          actorFailures.push({ actor, error });
        }
      }
      attempt.failures = actorFailures.map(({ actor, error }) => ({
        actor,
        error: modelVaporError(error),
      }));
      try {
        if (actorFailures.length > 0) throw actorFailures[0]!.error;
        assert.deepEqual(
          native,
          official,
          "whole actual DOM, event and explicitly diagnostic getter/setter outcomes",
        );
        attempt.state = "matched-observation-unqualified";
      } catch (error) {
        Object.assign(attempt, { state: "failed", error: modelVaporError(error) });
        remember(error);
        throw error;
      }
    },
  );
}

after(() => {
  const primaryComplete =
    primaryAttempts.length === 7 &&
    primaryAttempts.every((row) => row.state === "observed-unqualified");
  const runtimeMatched =
    runtimeAttempts.length === 7 &&
    runtimeAttempts.every((row) => row.state === "matched-observation-unqualified");
  try {
    assert(primaryComplete, "all seven complete primary attempts");
    if (captureRequested) {
      assert(captureComplete, "the complete actual current source capture is required");
      assert(runtimeMatched, "all seven paired actual runtime attempts");
    }
  } catch (error) {
    remember(error);
  }
  const common = {
    qualification:
      !captureRequested && primaryComplete
        ? "official-primary-only-native-not-executed"
        : failed
          ? "failed-first-complete-observation"
          : "observed-unqualified-current-oracles-pending",
    adapter: "compile_native_vapor_setup_sfc",
    target: {
      compiler: "@vue/compiler-sfc@3.6.0-rc.9",
      runtime: "vue@3.6.0-rc.9",
      dom: "happy-dom@21.0.4",
      typescript: "6.0.3",
    },
    nativeGoldenComparisonPerformed: false,
    wholePacketQualification: false,
    mode,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    runId: process.env.GITHUB_RUN_ID ?? null,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    inputHash: hash(inputRaw),
    inputPack: pack,
    refusalInputHash: hash(contextRaw),
    sourceCaptureHash: captureRaw === null ? null : hash(captureRaw),
    sourceCaptureRaw: captureRaw,
    sourceCapture: captured ?? null,
    captureObservation,
    captureComplete,
    primaryComplete,
    runtimeMatched,
    finalFailure: failed ? modelVaporError(firstFailure) : null,
    limits:
      "First observations, no historical native golden; captured getter/setter actors are diagnostic, not public proxy state. Raw compiler/maps/component metadata and actor-specific helper observations remain separate from compared actual DOM/event outcomes.",
  };
  const write = (prefix: string | undefined, schema: string, rows: ObjectValue[]) => {
    if (!prefix) return;
    const destination = `${prefix}.${mode}.json`;
    fs.mkdirSync(path.dirname(destination), { recursive: true });
    fs.writeFileSync(
      destination,
      JSON.stringify({ ...common, schema, fixtures: rows }, null, 2) + "\n",
    );
  };
  // Keep the original source capture and complete attempts, including unknown fields, before verdict.
  let writeFailure: unknown;
  try {
    write(primaryPrefix, "vize.native-vapor.original-input-model-primary-capture", primaryAttempts);
  } catch (error) {
    writeFailure = error;
  }
  try {
    write(runtimePrefix, "vize.native-vapor.original-input-model-runtime-capture", runtimeAttempts);
  } catch (error) {
    if (writeFailure === undefined) writeFailure = error;
  }
  if (failed) throw firstFailure;
  if (writeFailure !== undefined) throw writeFailure;
});
