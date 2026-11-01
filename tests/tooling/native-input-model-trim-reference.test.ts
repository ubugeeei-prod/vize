// First genuine cross-target observations; no native golden is loaded here.
import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  assertTrimFixture,
  hash,
  object,
  type ObjectValue,
  type TrimFixture,
  type TrimMode,
  type TrimTarget,
} from "./support/native-input-model-trim-boundary.ts";
import { openModelDom } from "./support/native-original-input-model-sfc-dom-environment.ts";
import { compileInputModelPrimary } from "./support/native-original-input-model-sfc-dom-primary.ts";
import { compileInputModelSsrPrimary } from "./support/native-original-input-model-sfc-ssr-primary.ts";
import { compileInputModelVaporPrimary } from "./support/native-original-input-model-vapor-primary.ts";
import { executeTrimModelDom, modelError } from "./support/native-input-model-trim-dom-runtime.ts";
import { executeTrimModelVapor } from "./support/native-input-model-trim-vapor-runtime.ts";
import { executeInputModelSsr } from "./support/native-original-input-model-sfc-ssr-runtime.ts";
import {
  assertContext,
  type ModelSsrContext,
} from "./support/native-original-input-model-sfc-ssr-boundary.ts";

const fixtureRoot = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const inputRaw = fs.readFileSync(
  new URL("native_original_input_model_trim_inputs.json", fixtureRoot),
  "utf8",
);
const pack: unknown = JSON.parse(inputRaw);
object(pack);
assert.equal(pack.schema, "vize.native-sfc.original-input-model-trim-inputs");
assert.equal(pack.qualification, "new-inputs-not-native-execution-evidence");
assert.equal(pack.typescript, "6.0.3");
assert(typeof pack.filename === "string");
const filename = pack.filename;
assert.deepEqual(pack.targets, {
  dom: { compiler: "3.5.35", runtime: "3.5.35" },
  ssr: { compiler: "3.5.35", runtime: "3.5.35", renderer: "3.5.35" },
  vapor: { compiler: "3.6.0-rc.9", runtime: "3.6.0-rc.9" },
});
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 3);
const fixtures: TrimFixture[] = pack.fixtures.map((value: unknown) => {
  assertTrimFixture(value);
  return value;
});
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 3);
const contextRaw = fs.readFileSync(
  new URL("native_original_input_model_ssr_contexts_vue_3_5_35.json", fixtureRoot),
  "utf8",
);
const contextPack: unknown = JSON.parse(contextRaw);
object(contextPack);
assert(Array.isArray(contextPack.contexts) && contextPack.contexts.length === 8);
const contexts: ModelSsrContext[] = contextPack.contexts.map((value: unknown) => {
  assertContext(value);
  return value;
});
assert(Array.isArray(contextPack.refusals) && contextPack.refusals.length === 12);
const originalRefusals = contextPack.refusals;
const modeValue = process.env.NODE_ENV ?? "development";
assert(modeValue === "development" || modeValue === "production");
const mode: TrimMode = modeValue;
const targetValue = process.env.VIZE_NATIVE_MODEL_TRIM_TARGET ?? "dom";
assert(targetValue === "dom" || targetValue === "ssr" || targetValue === "vapor");
const target: TrimTarget = targetValue;
const capturePath = process.env.VIZE_NATIVE_MODEL_TRIM_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_MODEL_TRIM_REQUIRE_CAPTURE === "1";
const primaryPrefix = process.env.VIZE_NATIVE_MODEL_TRIM_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_MODEL_TRIM_RUNTIME_CAPTURE;
if (requireCapture) {
  assert(capturePath && primaryPrefix && runtimePrefix);
  assert(
    typeof process.env.GITHUB_SHA === "string" && /^[a-f0-9]{40}$/.test(process.env.GITHUB_SHA),
  );
}
type NativeRow = {
  id: string;
  source: string;
  target: string;
  mode: string;
  code: string;
  nativeMapRaw: string;
  nativeMap: ObjectValue;
};
function assertNativeRow(value: unknown): asserts value is NativeRow {
  object(value);
  for (const key of ["id", "source", "target", "mode", "code", "nativeMapRaw"])
    assert(typeof value[key] === "string");
  object(value.nativeMap);
}
type Primary =
  | ReturnType<typeof compileInputModelPrimary>
  | ReturnType<typeof compileInputModelSsrPrimary>
  | ReturnType<typeof compileInputModelVaporPrimary>;
const nativeRows = new Map<string, NativeRow>();
const primaries = new Map<string, Primary>();
const primaryAttempts: ObjectValue[] = [];
const runtimeAttempts: ObjectValue[] = [];
const captureObservation: ObjectValue = { state: "absent" };
let captured: unknown;
let captureRaw: string | null = null;
let captureComplete = false;
let failed = false;
let firstFailure: unknown;
let runtimeMatches = 0;
let dom: Awaited<ReturnType<typeof openModelDom>> | undefined;
const cleanup: ObjectValue = { dom: "not-used" };
function remember(error: unknown) {
  if (!failed) {
    failed = true;
    firstFailure = error;
  }
}
if (capturePath) {
  try {
    captureRaw = fs.readFileSync(capturePath, "utf8");
    captured = JSON.parse(captureRaw) as unknown;
    captureObservation.state = "read";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelError(error) });
  }
}
test(
  "complete original trim source capture and all actual target owners",
  { skip: !capturePath && !requireCapture },
  () => {
    try {
      object(captured);
      assert.equal(captured.schema, "vize.native-sfc.original-input-model-trim-capture");
      assert.equal(captured.inputHash, hash(inputRaw));
      assert.deepEqual(captured.inputPack, pack);
      assert.equal(captured.qualification, "observed-unqualified-current-oracles-pending");
      assert.equal(captured.nativeGoldenComparisonPerformed, false);
      assert.equal(captured.wholePacketQualification, false);
      assert.equal(captured.nativeCaptureComplete, true);
      assert.deepEqual(captured.failures, []);
      if (requireCapture) assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
      const identities = (["dom", "ssr", "vapor"] as const).flatMap((kind) =>
        (["development", "production"] as const).flatMap((runtimeMode) =>
          fixtures.map((fixture) => [fixture.id, kind, runtimeMode]),
        ),
      );
      for (const key of ["fixtures", "custody", "attempts"]) {
        const rows: unknown = captured[key];
        assert(Array.isArray(rows));
        assert.deepEqual(
          rows.map((row: unknown) => {
            object(row);
            return [row.id, row.target, row.mode];
          }),
          identities,
        );
        for (const row of rows) {
          object(row);
          const fixture = fixtures.find((value) => value.id === row.id);
          assert(fixture);
          assert.equal(row.source, fixture.source);
        }
      }
      assert(Array.isArray(captured.fixtures));
      for (const row of captured.fixtures) {
        assertNativeRow(row);
        assert.deepEqual(JSON.parse(row.nativeMapRaw) as unknown, row.nativeMap);
        assert.deepEqual(row.nativeMap.sourcesContent, [row.source]);
        if (row.target === target && row.mode === mode) nativeRows.set(row.id, row);
      }
      assert.equal(nativeRows.size, 3);
      assert(Array.isArray(captured.refusals));
      const expectedRefusals = (["dom", "ssr", "vapor"] as const).flatMap((kind) =>
        originalRefusals.map((value: unknown) => {
          object(value);
          return [value.id, value.source, kind];
        }),
      );
      assert.deepEqual(
        captured.refusals.map((value: unknown) => {
          object(value);
          object(value.attempt);
          assert.equal(value.attempt.state, "refused");
          assert(typeof value.attempt.error === "string");
          return [value.id, value.source, value.target];
        }),
        expectedRefusals,
      );
      captureComplete = true;
      captureObservation.state = "complete-original-source-observation-unqualified";
    } catch (error) {
      remember(error);
      throw error;
    }
  },
);

for (const fixture of fixtures) {
  test(`official ${target}/${mode}/${fixture.id} primary`, () => {
    const attempt: ObjectValue = {
      id: fixture.id,
      target,
      mode,
      source: fixture.source,
      state: "attempting",
    };
    primaryAttempts.push(attempt);
    try {
      const primary =
        target === "dom"
          ? compileInputModelPrimary(fixture, attempt, filename, mode)
          : target === "ssr"
            ? compileInputModelSsrPrimary(fixture, attempt, filename, mode)
            : compileInputModelVaporPrimary(fixture, attempt, filename, mode);
      attempt.primary = primary;
      primaries.set(fixture.id, primary);
      attempt.state = "observed-unqualified";
    } catch (error) {
      attempt.state = "failed";
      attempt.error = modelError(error);
      remember(error);
      throw error;
    }
  });
}
for (const fixture of fixtures) {
  test(
    `actual ${target}/${mode}/${fixture.id} native/reference runtime`,
    { skip: !capturePath && !requireCapture },
    async () => {
      const attempt: ObjectValue = {
        id: fixture.id,
        target,
        mode,
        source: fixture.source,
        state: "attempting",
        actors: [],
      };
      runtimeAttempts.push(attempt);
      const actors: ObjectValue[] = [];
      attempt.actors = actors;
      let firstActorFailure: unknown;
      let actorFailed = false;
      for (const actor of ["official", "native"] as const) {
        const row: ObjectValue = { actor, state: "attempting", observations: [] };
        actors.push(row);
        try {
          const primary = primaries.get(fixture.id);
          const native = nativeRows.get(fixture.id);
          const code = actor === "official" ? primary?.runtimeModule : native?.code;
          assert(typeof code === "string", `${actor}: actual complete module required`);
          row.code = code;
          if (target === "dom") {
            dom ??= await openModelDom();
            cleanup.dom = "opened";
            const observations: unknown[] = [];
            row.observations = observations;
            row.result = await executeTrimModelDom(dom, code, fixture, mode, actor, (value) =>
              observations.push(value),
            );
          } else if (target === "ssr")
            row.result = await executeInputModelSsr(code, fixture, contexts, row);
          else row.result = await executeTrimModelVapor(code, fixture, mode, row);
          row.state = "observed-unqualified";
        } catch (error) {
          row.state = "failed";
          row.error = modelError(error);
          if (!actorFailed) {
            actorFailed = true;
            firstActorFailure = error;
          }
        }
      }
      try {
        if (actorFailed) throw firstActorFailure;
        const [official, native] = actors;
        assert(official && native);
        assert.deepEqual(native.result, official.result, "entire unnormalized runtime outcome");
        attempt.state = "matched-observation-unqualified";
        runtimeMatches++;
      } catch (error) {
        attempt.state = "failed";
        attempt.error = modelError(error);
        remember(error);
        throw error;
      }
    },
  );
}
after(async () => {
  if (dom) {
    try {
      await dom.close();
      cleanup.dom = "closed-and-globals-restored";
    } catch (error) {
      cleanup.dom = "failed";
      cleanup.error = modelError(error);
      remember(error);
    }
  }
  const common = {
    schema: "vize.native-sfc.original-input-model-trim-runtime-capture",
    target,
    mode,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    inputHash: hash(inputRaw),
    inputPack: pack,
    contextHash: hash(contextRaw),
    contextPack,
    sourceCapture: captured ?? null,
    sourceCaptureRawSha256: captureRaw === null ? null : hash(captureRaw),
    captureObservation,
    captureComplete,
    qualification: "observed-unqualified-current-oracles-pending",
    nativeGoldenComparisonPerformed: false,
    wholePacketQualification: false,
    runtimeMatches,
    cleanup,
    failed,
    failure: failed ? modelError(firstFailure) : null,
  };
  if (primaryPrefix)
    fs.writeFileSync(
      `${primaryPrefix}.${target}.${mode}.json`,
      `${JSON.stringify({ ...common, primary: primaryAttempts }, null, 2)}\n`,
    );
  if (runtimePrefix)
    fs.writeFileSync(
      `${runtimePrefix}.${target}.${mode}.json`,
      `${JSON.stringify({ ...common, runtimeResults: runtimeAttempts }, null, 2)}\n`,
    );
  if (requireCapture) {
    assert(
      captureComplete &&
        primaryAttempts.length === 3 &&
        runtimeAttempts.length === 3 &&
        runtimeMatches === 3,
    );
    assert.equal(failed, false, "raw attempts retained before verdict");
  }
});
