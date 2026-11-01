import assert from "node:assert/strict";
import fs from "node:fs";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import { loadLinterManifest } from "../differential/linter-api.ts";
import { expectedNativeReason } from "../differential/linter-native.ts";
import {
  ORIGINAL_PLACEHOLDER_WIRES,
  expectedPlaceholderReason,
} from "../differential/linter-placeholder-contract.ts";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "../..");
const loaded = loadLinterManifest(
  path.join(root, "tests/_fixtures/differential/linter/manifest.json"),
  root,
);
const isRecord = (value: unknown): value is Record<string, unknown> =>
  value !== null && typeof value === "object" && !Array.isArray(value);

void test("all four original placeholder wires remain whole; only direct explicit values are candidate native", () => {
  const originals: unknown = JSON.parse(
    fs.readFileSync(
      path.join(root, "crates/vize_patina/tests/fixtures/lint-history/cases.json"),
      "utf8",
    ),
  );
  assert(Array.isArray(originals));
  assert.deepEqual(
    originals.filter(
      (wire: unknown) => isRecord(wire) && wire.rule === "a11y/placeholder-label-option",
    ),
    ORIGINAL_PLACEHOLDER_WIRES,
  );
  for (const original of ORIGINAL_PLACEHOLDER_WIRES) {
    const fixture = loaded.cases.find(
      (entry: { id: string }) => entry.id === `linter/current-api/${original.id}`,
    );
    assert(fixture);
    assert.deepEqual(JSON.parse(fixture.input.toString()), original);
    const expected =
      original.id === "placeholder-label-optgroup-report"
        ? {
            api: "--native-current-api",
            entry: "template",
            kind: "UnsupportedContext",
            detail: "UnsupportedContext { span: Span { start: 17, end: 37 } }",
          }
        : null;
    assert.deepEqual(expectedNativeReason(fixture), expected);
    assert.throws(() => expectedPlaceholderReason(original, "--unprovided-api"));
    for (const [key, value] of Object.entries(original)) {
      const changed = {
        ...original,
        [key]: typeof value === "number" ? value + 1 : `${value}EXTRA`,
      };
      assert.throws(() => expectedPlaceholderReason(changed, "--native-current-api"), key);
    }
    assert.throws(() =>
      expectedPlaceholderReason({ ...original, extra: "unproved" }, "--native-current-api"),
    );
  }
});

void test("malformed placeholder wires never receive an expected native outcome", () => {
  for (const input of [null, undefined, true, 0, "wire", [], {}, { id: "unknown" }])
    assert.throws(() => expectedPlaceholderReason(input, "--native-current-api"));
});
