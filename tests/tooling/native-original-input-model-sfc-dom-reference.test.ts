// First genuine whole Model source/primary/DOM observations; no guessed goldens.
import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  assertModelFixture,
  hash,
  openModelDom,
  type ModelFixture,
  type ModelMode,
} from "./support/native-original-input-model-sfc-dom-environment.ts";
import {
  compileInputModelPrimary,
  typescript,
} from "./support/native-original-input-model-sfc-dom-primary.ts";
import {
  executeInputModel,
  modelError,
} from "./support/native-original-input-model-sfc-dom-runtime.ts";

type ObjectValue = Record<string, unknown>;
function object(value: unknown): asserts value is ObjectValue {
  assert(value && typeof value === "object" && !Array.isArray(value));
}
const packUrl = new URL(
  "../../crates/vize_atelier_sfc/tests/fixtures/native_original_input_model_inputs_vue_3_5_35.json",
  import.meta.url,
);
const inputRaw = fs.readFileSync(packUrl, "utf8");
const pack: unknown = JSON.parse(inputRaw);
object(pack);
assert.equal(pack.schema, "vize.native-sfc.original-input-model-inputs");
assert.equal(pack.compiler, "@vue/compiler-sfc@3.5.35");
assert.equal(pack.runtime, "vue@3.5.35");
assert.equal(pack.dom, "happy-dom@21.0.4");
assert.equal(pack.typescript, typescript.version);
assert.equal(pack.filename, "OriginalInputModel雪🌸.vue");
assert.equal(pack.qualification, "new-original-inputs-not-historical-native-evidence");
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 7);
const fixtures: ModelFixture[] = pack.fixtures.map((value: unknown) => {
  assertModelFixture(value);
  return value;
});
assert.equal(new Set(fixtures.map((row) => row.id)).size, 7);
const expectedIds = [
  "let-string",
  "var-number",
  "let-boolean",
  "var-null",
  "let-bigint",
  "ts-unicode-crlf",
  "unquoted-void",
];
assert.deepEqual(
  fixtures.map((row) => row.id),
  expectedIds,
);
const actualMode = process.env.NODE_ENV ?? "development";
assert(actualMode === "development" || actualMode === "production");
const mode: ModelMode = actualMode;
const capturePath = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_REQUIRE_CAPTURE === "1";
const primaryPrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_ORIGINAL_INPUT_MODEL_SFC_DOM_RUNTIME_CAPTURE;
if (requireCapture) assert(capturePath && primaryPrefix && runtimePrefix);

let captureRaw: string | null = null;
let captured: unknown;
const captureObservation: ObjectValue = { state: "absent" };
if (capturePath) {
  try {
    captureRaw = fs.readFileSync(capturePath, "utf8");
    captured = JSON.parse(captureRaw) as unknown;
    captureObservation.state = "read";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelError(error) });
  }
}
type NativeRow = {
  id: string;
  mode: ModelMode;
  source: string;
  code: string;
  nativeMap: ObjectValue;
  nativeMapRaw: string;
};
const rows = new Map<string, NativeRow>();
const primaryRows = new Map<string, ReturnType<typeof compileInputModelPrimary>>();
const primaryAttempts: ObjectValue[] = [];
const runtimeAttempts: ObjectValue[] = [];
// This is the genuine first load of Vue runtime-dom after actual DOM globals.
const dom = await openModelDom();

test("original Model capture keeps all fourteen source/mode/owner/attempt rows", () => {
  if (!captured && !requireCapture && !capturePath) return;
  try {
    object(captured);
    assert.equal(captured.schema, "vize.native-sfc.original-input-model-capture");
    assert.equal(
      captured.adapter,
      "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
    );
    assert.equal(captured.qualification, "observed-unqualified-current-oracles-pending");
    assert.equal(captured.nativeGoldenComparisonPerformed, false);
    assert.deepEqual(captured.failures, []);
    if (requireCapture) assert.equal(captured.sourceRevision, process.env.GITHUB_SHA);
    const identities = (["development", "production"] as const).flatMap((entryMode) =>
      fixtures.map((row) => ({ id: row.id, mode: entryMode, source: row.source })),
    );
    for (const field of ["fixtures", "custody", "attempts"]) {
      const entries = captured[field];
      assert(Array.isArray(entries));
      assert.deepEqual(
        entries.map((entry: unknown) => {
          object(entry);
          return { id: entry.id, mode: entry.mode, source: entry.source };
        }),
        identities,
        field,
      );
    }
    assert(Array.isArray(captured.fixtures));
    for (const row of captured.fixtures) {
      object(row);
      assert(typeof row.code === "string" && row.code.length > 0);
      assert(typeof row.nativeMapRaw === "string");
      object(row.nativeMap);
      assert.deepEqual(JSON.parse(row.nativeMapRaw), row.nativeMap);
      assert.equal(row.nativeMap.version, 3);
      assert.equal(row.nativeMap.file, pack.filename);
      assert.deepEqual(row.nativeMap.sources, [pack.filename]);
      assert.deepEqual(row.nativeMap.sourcesContent, [row.source]);
      assert(Array.isArray(row.nativeMap.names));
      assert(typeof row.nativeMap.mappings === "string" && row.nativeMap.mappings.length > 0);
      if (row.mode === mode) rows.set(String(row.id), row as NativeRow);
    }
    assert.equal(rows.size, fixtures.length);
    assert(Array.isArray(captured.attempts));
    for (const entry of captured.attempts) {
      object(entry);
      object(entry.mapped);
      object(entry.plain);
      assert.equal(entry.mapped.state, "emitted");
      assert.equal(entry.plain.state, "emitted");
      assert.equal(entry.mapped.code, entry.plain.code);
      assert.equal(entry.plain.nativeMapRaw, null);
      assert.equal(entry.plain.nativeMap, null);
      const actual = captured.fixtures.find((candidate: unknown) => {
        object(candidate);
        return candidate.id === entry.id && candidate.mode === entry.mode;
      });
      object(actual);
      assert.equal(entry.mapped.code, actual.code);
      assert.equal(entry.mapped.nativeMapRaw, actual.nativeMapRaw);
      assert.deepEqual(entry.mapped.nativeMap, actual.nativeMap);
      if (entry.mode === "development") assert.deepEqual(entry.defaultDevelopment, entry.mapped);
      else assert.equal(entry.defaultDevelopment, null);
    }
    assert(Array.isArray(captured.refusals));
    const refusalIds = [
      "const",
      "unresolved",
      "member",
      "modifier",
      "argument",
      "late-header",
      "sibling",
      "nested",
      "unknown-initializer",
      "nul-string",
      "cr-string",
      "lone-surrogate",
    ];
    assert.deepEqual(
      captured.refusals.map((entry: unknown) => {
        object(entry);
        assert(typeof entry.source === "string" && entry.source.length > 0);
        object(entry.result);
        assert.equal(entry.result.state, "refused");
        assert(typeof entry.result.error === "string");
        return { id: entry.id, mode: entry.mode };
      }),
      refusalIds.flatMap((id) => ["development", "production"].map((mode) => ({ id, mode }))),
    );
    captureObservation.state = "complete-original-observation-unqualified";
  } catch (error) {
    Object.assign(captureObservation, { state: "failed", error: modelError(error) });
    throw error;
  }
});

for (const fixture of fixtures) {
  test(`${fixture.id}: complete pinned official primary/${mode}`, () => {
    const attempt: ObjectValue = { id: fixture.id, mode, source: fixture.source, state: "started" };
    primaryAttempts.push(attempt);
    try {
      const primary = compileInputModelPrimary(fixture, attempt, String(pack.filename), mode);
      primaryRows.set(fixture.id, primary);
      Object.assign(attempt, { state: "compiled", primary });
    } catch (error) {
      Object.assign(attempt, { state: "failed", error: modelError(error) });
      throw error;
    }
  });
  test(
    `${fixture.id}: actual DOM input/setter/IME/${mode}`,
    {
      skip: !captured && !capturePath && !requireCapture,
    },
    async () => {
      const referenceObservations: unknown[] = [];
      const nativeObservations: unknown[] = [];
      const attempt: ObjectValue = {
        id: fixture.id,
        mode,
        source: fixture.source,
        state: "started",
        referenceObservations,
        nativeObservations,
      };
      runtimeAttempts.push(attempt);
      const row = rows.get(fixture.id);
      const primary = primaryRows.get(fixture.id);
      Object.assign(attempt, {
        nativeRow: row ?? null,
        referenceModule: primary?.runtimeModule ?? null,
      });
      const failures: { actor: string; error: unknown }[] = [];
      let reference: unknown;
      let native: unknown;
      try {
        try {
          assert(primary, "actual whole official primary required");
          attempt.referenceCodeHash = hash(primary.runtimeModule);
          reference = await executeInputModel(
            dom,
            primary.runtimeModule,
            fixture,
            mode,
            "official",
            (entry) => referenceObservations.push(entry),
          );
          attempt.reference = reference;
        } catch (error) {
          failures.push({ actor: "official", error });
        }
        try {
          assert(row, "actual whole native row required");
          Object.assign(attempt, {
            nativeCodeHash: hash(row.code),
            nativeMapRawHash: hash(row.nativeMapRaw),
          });
          native = await executeInputModel(dom, row.code, fixture, mode, "native", (entry) =>
            nativeObservations.push(entry),
          );
          attempt.native = native;
        } catch (error) {
          failures.push({ actor: "native", error });
        }
        attempt.failures = failures.map(({ actor, error }) => ({
          actor,
          error: modelError(error),
        }));
        if (failures.length) throw failures[0]!.error;
        assert.deepEqual(
          native,
          reference,
          "all actual DOM/event/typed binding/render/unmount outcomes",
        );
        attempt.state = "matched-observation-unqualified";
      } catch (error) {
        Object.assign(attempt, { state: "failed", error: modelError(error) });
        throw error;
      }
    },
  );
}

after(async () => {
  const primaryComplete =
    primaryAttempts.length === fixtures.length &&
    primaryAttempts.every((row) => row.state === "compiled");
  const runtimeMatched =
    runtimeAttempts.length === fixtures.length &&
    runtimeAttempts.every((row) => row.state === "matched-observation-unqualified");
  let failed = false;
  let firstFailure: unknown;
  const environmentCleanup: ObjectValue = { state: "pending" };
  try {
    assert(primaryComplete);
    if (requireCapture || captured || capturePath) {
      assert.equal(captureObservation.state, "complete-original-observation-unqualified");
      assert(runtimeMatched);
      assert.equal(runtimeAttempts.length, fixtures.length);
    }
  } catch (error) {
    failed = true;
    firstFailure = error;
  }
  try {
    await dom.close();
    environmentCleanup.state = "closed-globals-restored";
  } catch (error) {
    Object.assign(environmentCleanup, { state: "failed", error: modelError(error) });
    if (!failed) {
      failed = true;
      firstFailure = error;
    }
  }
  const common = {
    adapter: "vize_atelier_sfc::compile_native_selected_setup_sfc_dom_with_mode",
    qualification: "observed-unqualified-current-oracles-pending",
    nativeGoldenComparisonPerformed: false,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    runId: process.env.GITHUB_RUN_ID ?? null,
    runAttempt: process.env.GITHUB_RUN_ATTEMPT ?? null,
    mode,
    inputHash: hash(inputRaw),
    inputPack: pack,
    sourceCaptureHash: captureRaw ? hash(captureRaw) : null,
    captureObservation,
    primaryComplete,
    runtimeMatched,
    environmentCleanup,
    finalFailure: failed ? modelError(firstFailure) : null,
  };
  // Keep complete actual packets and cleanup before rethrowing the first error.
  if (primaryPrefix)
    fs.writeFileSync(
      `${primaryPrefix}.${mode}.json`,
      JSON.stringify(
        {
          ...common,
          schema: "vize.native-sfc.original-input-model-primary-capture",
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
          schema: "vize.native-sfc.original-input-model-runtime-capture",
          host: "vue-runtime-dom+happy-dom",
          sourceCapture: captured ?? null,
          fixtures: runtimeAttempts,
        },
        null,
        2,
      ) + "\n",
    );
  if (failed) throw firstFailure;
});
