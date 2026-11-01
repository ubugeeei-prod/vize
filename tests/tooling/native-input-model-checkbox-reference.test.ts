// Strict whole checkbox replay; origin observation rows remain immutable.
import assert from "node:assert/strict";
import fs from "node:fs";
import { after, test } from "node:test";
import {
  hash,
  object,
  type ObjectValue,
  type TrimMode,
  type TrimTarget,
} from "./support/native-input-model-trim-boundary.ts";
import {
  assertCheckboxFixture,
  assertCheckboxCustody,
  type CheckboxFixture,
} from "./support/native-input-model-checkbox-boundary.ts";
import { openModelDom } from "./support/native-original-input-model-sfc-dom-environment.ts";
import { compileInputModelPrimary } from "./support/native-original-input-model-sfc-dom-primary.ts";
import { compileInputModelSsrPrimary } from "./support/native-original-input-model-sfc-ssr-primary.ts";
import { compileInputModelVaporPrimary } from "./support/native-original-input-model-vapor-primary.ts";
import {
  executeCheckboxModelDom,
  modelError,
} from "./support/native-input-model-checkbox-dom-runtime.ts";
import { executeCheckboxModelVapor } from "./support/native-input-model-checkbox-vapor-runtime.ts";
import { executeInputModelSsr } from "./support/native-original-input-model-sfc-ssr-runtime.ts";
import {
  assertContext,
  type ModelSsrContext,
} from "./support/native-original-input-model-sfc-ssr-boundary.ts";

const root = new URL("../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
const inputRaw = fs.readFileSync(
  new URL("native_original_input_model_checkbox_inputs.json", root),
  "utf8",
);
const pack: unknown = JSON.parse(inputRaw);
object(pack);
assert.equal(pack.schema, "vize.native-sfc.original-input-model-checkbox-inputs");
assert.equal(pack.qualification, "new-current-inputs-not-native-execution-evidence");
assert.equal(pack.typescript, "6.0.3");
assert.equal(pack.filename, "OriginalInputModelCheckbox.vue");
assert.deepEqual(pack.targets, {
  dom: { compiler: "3.5.35", runtime: "3.5.35" },
  ssr: { compiler: "3.5.35", runtime: "3.5.35", renderer: "3.5.35" },
  vapor: { compiler: "3.6.0-rc.9", runtime: "3.6.0-rc.9" },
});
assert(Array.isArray(pack.fixtures) && pack.fixtures.length === 3);
const fixtures: CheckboxFixture[] = pack.fixtures.map((value: unknown) => {
  assertCheckboxFixture(value);
  return value;
});
assert.equal(new Set(fixtures.map((fixture) => fixture.id)).size, 3);
assert.deepEqual(pack.groups, [
  { kind: "checkbox", filename: pack.filename, ids: fixtures.map((fixture) => fixture.id) },
]);
const refusalRaw = fs.readFileSync(
  new URL("native_original_input_model_checkbox_refusal_inputs.json", root),
  "utf8",
);
const refusalPack: unknown = JSON.parse(refusalRaw);
object(refusalPack);
assert.equal(refusalPack.schema, "vize.native-sfc.original-input-model-checkbox-outside-inputs");
assert(Array.isArray(refusalPack.cases) && refusalPack.cases.length === 16);
const refusalInputs: unknown[] = refusalPack.cases;
const contextRaw = fs.readFileSync(
  new URL("native_original_input_model_ssr_contexts_vue_3_5_35.json", root),
  "utf8",
);
const contextPack: unknown = JSON.parse(contextRaw);
object(contextPack);
assert(Array.isArray(contextPack.contexts) && contextPack.contexts.length === 8);
const contexts: ModelSsrContext[] = contextPack.contexts.map((value: unknown) => {
  assertContext(value);
  return value;
});
const modeValue = process.env.NODE_ENV ?? "development";
assert(modeValue === "development" || modeValue === "production");
const mode: TrimMode = modeValue;
const targetValue = process.env.VIZE_NATIVE_MODEL_CHECKBOX_TARGET ?? "dom";
assert(targetValue === "dom" || targetValue === "ssr" || targetValue === "vapor");
const target: TrimTarget = targetValue;
const goldens: unknown = JSON.parse(
  fs.readFileSync(new URL("native_original_input_model_checkbox_all_targets.json", root), "utf8"),
);
object(goldens);
assert.equal(goldens.schema, "vize.native-sfc.original-input-model-checkbox-complete-goldens");
assert.equal(goldens.inputHash, hash(inputRaw));
assert.deepEqual(goldens.inputPack, pack);
assert.deepEqual(goldens.refusalInputPack, refusalPack);
object(goldens.provenance);
assert.equal(goldens.provenance.independentlyReviewedBeforeFreeze, true);
object(goldens.native);
object(goldens.primary);
object(goldens.runtimeResults);
const expectedNative = goldens.native;
const primaryTarget = goldens.primary[target],
  runtimeTarget = goldens.runtimeResults[target];
object(primaryTarget);
object(runtimeTarget);
const expectedPrimary = primaryTarget[mode],
  expectedRuntime = runtimeTarget[mode];
assert(Array.isArray(expectedPrimary) && expectedPrimary.length === 3);
assert(Array.isArray(expectedRuntime) && expectedRuntime.length === 3);
function whole(actual: unknown, expected: unknown, label: string) {
  assert.deepEqual(actual, expected, label);
  assert.equal(JSON.stringify(actual), JSON.stringify(expected), label + " fields/order");
}
const capturePath = process.env.VIZE_NATIVE_MODEL_CHECKBOX_CAPTURE;
const requireCapture = process.env.VIZE_NATIVE_MODEL_CHECKBOX_REQUIRE_CAPTURE === "1";
const primaryPrefix = process.env.VIZE_NATIVE_MODEL_CHECKBOX_PRIMARY_CAPTURE;
const runtimePrefix = process.env.VIZE_NATIVE_MODEL_CHECKBOX_RUNTIME_CAPTURE;
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
const primaryAttempts: ObjectValue[] = [],
  runtimeAttempts: ObjectValue[] = [];
const captureObservation: ObjectValue = { state: "absent" };
let captured: unknown,
  captureRaw: string | null = null;
let sourceCaptureComplete = false,
  failed = false,
  firstFailure: unknown;
let runtimeMatches = 0;
let nativeGoldensMatched = false,
  primaryGoldenMatches = 0,
  runtimeGoldenMatches = 0;
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
  "complete original checkbox source capture and actual target owners",
  { skip: !capturePath && !requireCapture },
  () => {
    try {
      object(captured);
      assert.equal(captured.schema, "vize.native-sfc.original-input-model-checkbox-capture");
      assert.equal(captured.inputHash, hash(inputRaw));
      assert.deepEqual(captured.inputPack, pack);
      assert.deepEqual(captured.refusalInputPack, refusalPack);
      assert.equal(captured.qualification, "complete-current-native-oracles-matched");
      assert.equal(captured.nativeGoldenComparisonPerformed, true);
      assert.equal(captured.nativeGoldensMatched, true);
      assert.equal(captured.wholePacketQualification, false);
      assert.equal(captured.nativeCaptureComplete, true);
      assert.deepEqual(captured.failures, []);
      object(captured.layouts);
      assert.equal(captured.layouts.declaration, 96);
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
      assert(Array.isArray(captured.custody));
      for (const row of captured.custody) {
        assertCheckboxCustody(row);
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
      const outside = (["dom", "ssr", "vapor"] as const).flatMap((kind) =>
        refusalInputs.map((value: unknown) => {
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
        outside,
      );
      for (const field of ["fixtures", "custody", "attempts", "refusals"])
        whole(captured[field], expectedNative[field], `whole current checkbox native ${field}`);
      nativeGoldensMatched = true;
      sourceCaptureComplete = true;
      captureObservation.state = "complete-current-native-oracles-matched";
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
          ? compileInputModelPrimary(fixture, attempt, "OriginalInputModelCheckbox.vue", mode)
          : target === "ssr"
            ? compileInputModelSsrPrimary(fixture, attempt, "OriginalInputModelCheckbox.vue", mode)
            : compileInputModelVaporPrimary(
                fixture,
                attempt,
                "OriginalInputModelCheckbox.vue",
                mode,
              );
      attempt.primary = primary;
      primaries.set(fixture.id, primary);
      attempt.state = "observed-unqualified";
      const expected = expectedPrimary.find((value: unknown) => {
        object(value);
        return value.id === fixture.id;
      });
      assert(expected);
      whole(attempt, expected, `whole current ${target}/${mode}/${fixture.id} primary`);
      primaryGoldenMatches++;
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
      const actors: ObjectValue[] = [];
      const attempt: ObjectValue = {
        id: fixture.id,
        target,
        mode,
        source: fixture.source,
        state: "attempting",
        actors,
      };
      runtimeAttempts.push(attempt);
      let actorFailed = false,
        firstActorFailure: unknown;
      for (const actor of ["official", "native"] as const) {
        const row: ObjectValue = { actor, state: "attempting", observations: [] };
        actors.push(row);
        try {
          const code =
            actor === "official"
              ? primaries.get(fixture.id)?.runtimeModule
              : nativeRows.get(fixture.id)?.code;
          assert(typeof code === "string", `${actor}: actual complete module required`);
          row.code = code;
          if (target === "dom") {
            dom ??= await openModelDom();
            cleanup.dom = "opened";
            const observations: unknown[] = [];
            row.observations = observations;
            row.result = await executeCheckboxModelDom(dom, code, fixture, mode, actor, (value) =>
              observations.push(value),
            );
          } else if (target === "ssr")
            row.result = await executeInputModelSsr(code, fixture, contexts, row);
          else row.result = await executeCheckboxModelVapor(code, fixture, mode, row);
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
        const expected = expectedRuntime.find((value: unknown) => {
          object(value);
          return value.id === fixture.id;
        });
        assert(expected);
        whole(attempt, expected, `whole current ${target}/${mode}/${fixture.id} runtime`);
        runtimeGoldenMatches++;
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
  let primaryArrayMatched = false,
    runtimeArrayMatched = false;
  try {
    whole(primaryAttempts, expectedPrimary, `whole ordered ${target}/${mode} primary array`);
    primaryArrayMatched = true;
  } catch (error) {
    remember(error);
  }
  if (capturePath || requireCapture) {
    try {
      whole(runtimeAttempts, expectedRuntime, `whole ordered ${target}/${mode} runtime array`);
      runtimeArrayMatched = true;
    } catch (error) {
      remember(error);
    }
  }
  const qualified =
    requireCapture &&
    sourceCaptureComplete &&
    nativeGoldensMatched &&
    primaryGoldenMatches === 3 &&
    runtimeGoldenMatches === 3 &&
    primaryArrayMatched &&
    runtimeArrayMatched &&
    runtimeMatches === 3 &&
    !failed;
  const common = {
    schema: "vize.native-sfc.original-input-model-checkbox-runtime-capture",
    target,
    mode,
    sourceRevision: process.env.GITHUB_SHA ?? null,
    inputHash: hash(inputRaw),
    inputPack: pack,
    refusalInputHash: hash(refusalRaw),
    refusalInputPack: refusalPack,
    contextHash: hash(contextRaw),
    contextPack,
    sourceCapture: captured ?? null,
    sourceCaptureRawSha256: captureRaw === null ? null : hash(captureRaw),
    captureObservation,
    sourceCaptureComplete,
    captureComplete:
      sourceCaptureComplete &&
      primaryAttempts.length === 3 &&
      runtimeAttempts.length === 3 &&
      runtimeMatches === 3 &&
      !failed,
    qualification: qualified
      ? "complete-current-oracles-matched"
      : !requireCapture
        ? "primary-only-or-unforced-observation"
        : "failed-current-observation",
    nativeGoldenComparisonPerformed: nativeGoldensMatched,
    nativeGoldensMatched,
    primaryGoldenMatches,
    runtimeGoldenMatches,
    primaryArrayMatched,
    runtimeArrayMatched,
    wholePacketQualification: qualified,
    runtimeMatches,
    cleanup,
    failed,
    failure: failed ? modelError(firstFailure) : null,
  };
  if (primaryPrefix)
    fs.writeFileSync(
      `${primaryPrefix}.${target}.${mode}.json`,
      JSON.stringify({ ...common, primary: primaryAttempts }, null, 2) + "\n",
    );
  if (runtimePrefix)
    fs.writeFileSync(
      `${runtimePrefix}.${target}.${mode}.json`,
      JSON.stringify({ ...common, runtimeResults: runtimeAttempts }, null, 2) + "\n",
    );
  if (requireCapture) {
    assert(
      sourceCaptureComplete &&
        primaryAttempts.length === 3 &&
        runtimeAttempts.length === 3 &&
        runtimeMatches === 3,
    );
    assert.equal(failed, false, "all raw attempts and cleanup retained before verdict");
    assert.equal(qualified, true, "all whole current arrays, source and cleanup must match");
  }
});
