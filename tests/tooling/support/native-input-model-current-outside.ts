// One explicit current whole-array successor; historical positive packs stay frozen.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import fs from "node:fs";
type ObjectValue = Record<string, unknown>;
function object(value: unknown): asserts value is ObjectValue {
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
}
export function isLazyOriginalOutside(id: unknown, source: unknown): boolean {
  return (
    id === "modifier" &&
    source ===
      "<script setup>let draft='x'</script><template><input v-model.lazy='draft'/></template>"
  );
}
export function currentModelOutside(family: "dom" | "ssr" | "vapor" | "trim"): unknown[] {
  const root = new URL("../../../crates/vize_atelier_sfc/tests/fixtures/", import.meta.url);
  const input = fs.readFileSync(new URL("native_original_input_model_modifier_inputs.json", root));
  const expected: unknown = JSON.parse(
    fs.readFileSync(new URL("native_original_input_model_current_outside.json", root), "utf8"),
  );
  object(expected);
  assert.equal(expected.schema, "vize.native-sfc.input-model-current-outside");
  assert.equal(expected.inputHash, createHash("sha256").update(input).digest("hex"));
  assert.deepEqual(expected.inputPack, JSON.parse(input.toString("utf8")) as unknown);
  object(expected.provenance);
  assert.equal(expected.provenance.independentlyReviewedBeforeFreeze, true);
  object(expected.arrays);
  const rows: unknown = expected.arrays[family];
  assert(Array.isArray(rows));
  assert.equal(rows.length, family === "dom" ? 24 : family === "trim" ? 36 : 12);
  return rows;
}
