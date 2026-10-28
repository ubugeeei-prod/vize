import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  attributesPacketReference,
  exposureReferenceCustody,
} from "../native/sfc-parse-attributes-reference.ts";

const captured = JSON.parse(
  readFileSync(
    new URL("./fixtures/script-setup-default-expose-packets.json", import.meta.url),
    "utf8",
  ),
) as {
  packets: { source: string; before: Record<string, unknown>; after: Record<string, unknown> }[];
  emptySetupModules: {
    packets: { source: string; before: Record<string, unknown>; after: Record<string, unknown> }[];
  };
};
const rows = [...captured.packets, ...captured.emptySetupModules.packets];

test("parsed-attribute compile packets advance only the reviewed exposure reference", () => {
  assert.equal(rows.length, 6);
  assert.equal(exposureReferenceCustody.fixSource, "9157b9fa83dbdc98100be084be288f21d0d1bc18");
  for (const row of rows) {
    const before = { kind: "return", json: JSON.stringify(row.before) };
    const retained = structuredClone(before);
    assert.deepEqual(attributesPacketReference(before, row.source, "compile"), {
      kind: "return",
      json: JSON.stringify(row.after),
    });
    assert.deepEqual(before, retained);
    for (const mode of ["omitted", "filename", "disabled"]) {
      assert.deepEqual(attributesPacketReference(before, row.source, mode), before);
    }
  }
});

test("parsed-attribute errors and ordinary sources keep complete original packets", () => {
  const error = { kind: "throw", code: "Original", message: "original", stack: "retained" };
  assert.deepEqual(attributesPacketReference(error, "<script setup/>", "compile"), error);
  const ordinary = { kind: "return", json: '{"code":"export default {}","errors":[]}' };
  assert.deepEqual(attributesPacketReference(ordinary, "<script/>", "compile"), ordinary);
});

test("the attribute adapter still rejects unrelated whole-packet changes", () => {
  for (const row of captured.packets) {
    const reference = attributesPacketReference(
      { kind: "return", json: JSON.stringify(row.before) },
      row.source,
      "compile",
    );
    for (const key of ["code", "map", "moduleShape", "errors", "macroArtifacts"]) {
      const changed: Record<string, unknown> = structuredClone(row.after);
      changed[key] = key === "code" ? String(changed[key]) + "\nvoid 1;" : "unexpected";
      assert.throws(() =>
        assert.deepEqual({ kind: "return", json: JSON.stringify(changed) }, reference),
      );
    }
  }
});
