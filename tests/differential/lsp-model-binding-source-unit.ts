import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import type { FixtureData } from "./lsp-types.ts";

export const MODEL_BINDING_UNIT_ID = "lsp/regression/reported-model-binding-rename-11383";
export const MODEL_BINDING_INPUT_SHA256 =
  "172d196ae0239dc816535cad59237a9ae01d75c905995746c4552586dac986d5";
export const MODEL_BINDING_UNIT_REASON =
  "This pinned input is consumed by the Rust model-binding endpoint unit test; the generic stdio adapter executes no requests and supplies no wire, native, capture or qualification credit.";
export const MODEL_BINDING_UNIT_SESSION = {
  kind: "reported-model-binding-source-unit-v1",
  state: "source-inspected",
  executor: "crates/vize_maestro/src/ide/corsa_support/canonical/model_links.rs",
  test: "ide::corsa_support::canonical::model_links::tests::model_prop_locations_follow_the_distinct_native_update_identity",
  input: "input.json",
  sourceVariants: 8,
  plannedClientRpcCount: 0,
  nativeParityChecked: 0,
  capturedSessions: 0,
  qualified: false,
} as const;
export type ModelBindingSourceUnit = typeof MODEL_BINDING_UNIT_SESSION;
export const MODEL_BINDING_UNIT_PLAN = {
  schema: "vize.lsp.reported-model-binding-source-unit",
  version: 1,
  genericAdapter: "unsupported-source-unit",
  inputSha256: MODEL_BINDING_INPUT_SHA256,
  executor: MODEL_BINDING_UNIT_SESSION.executor,
  test: MODEL_BINDING_UNIT_SESSION.test,
  sourceVariants: 8,
  genericClientRpcCount: 0,
  capturedSessions: 0,
  nativeParityChecked: 0,
  qualified: false,
} as const;

export function validateModelBindingSourceUnit(
  data: FixtureData,
  files: ReadonlyMap<string, Buffer>,
  plan: unknown,
): void {
  assert.deepEqual(
    Object.keys(data).sort(),
    [
      "version",
      "id",
      "provenance",
      "documentVersion",
      "openDocument",
      "initializationOptions",
      "files",
      "entry",
      "expected",
      "method",
      "sourceSession",
    ].sort(),
  );
  assert.equal(data.version, 1);
  assert.equal(data.id, MODEL_BINDING_UNIT_ID);
  assert.deepEqual(data.provenance, { fixCommit: "f848c7c31a9b623cd3fa7d62a2332b702f758d1b" });
  assert.equal(data.documentVersion, 1);
  assert.equal(data.openDocument, false);
  assert.deepEqual(data.initializationOptions, { typecheck: false, lint: false });
  assert.deepEqual(data.files, [
    { source: "input.json", runtimePath: "input.json", sha256: MODEL_BINDING_INPUT_SHA256 },
  ]);
  assert.equal(data.entry, "input.json");
  assert.equal(data.method, "source-session/reported-model-binding-unit");
  assert.deepEqual(data.sourceSession, MODEL_BINDING_UNIT_SESSION);
  assert.deepEqual(plan, MODEL_BINDING_UNIT_PLAN);
  const expectedBytes = Buffer.from(JSON.stringify(MODEL_BINDING_UNIT_PLAN, null, 2) + "\n");
  assert.deepEqual(data.expected, {
    source: "source-unit.expected.json",
    sha256: createHash("sha256").update(expectedBytes).digest("hex"),
  });
  modelBindingSourceUnitDialectInputs(files);
}

export function modelBindingSourceUnitDialectInputs(files: ReadonlyMap<string, Buffer>) {
  assert.deepEqual([...files.keys()], ["input.json"]);
  const input = files.get("input.json");
  assert.ok(input);
  assert.equal(createHash("sha256").update(input).digest("hex"), MODEL_BINDING_INPUT_SHA256);
  const parsed: unknown = JSON.parse(input.toString("utf8"));
  assert.ok(parsed && typeof parsed === "object" && !Array.isArray(parsed));
  const fixture = parsed as Record<string, unknown>;
  assert.deepEqual(
    Object.keys(fixture).sort(),
    ["schema", "id", "sourceTemplate", "names", "quotes"].sort(),
  );
  assert.equal(fixture.schema, 1);
  assert.equal(fixture.id, MODEL_BINDING_UNIT_ID);
  assert.deepEqual(fixture.names, ["x", "title", "valueId", "saveItem"]);
  assert.deepEqual(fixture.quotes, ["'", '"']);
  assert.equal(typeof fixture.sourceTemplate, "string");
  assert.equal((String(fixture.sourceTemplate).match(/__VIZE_MODEL_LITERAL__/g) ?? []).length, 1);
  return [{ filename: "Model.vue", source: String(fixture.sourceTemplate) }];
}
