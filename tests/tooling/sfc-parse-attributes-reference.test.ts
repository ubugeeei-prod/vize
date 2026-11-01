import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import {
  attributesPacketReference,
  exposureReferenceCustody,
} from "../native/sfc-parse-attributes-reference.ts";
import {
  emptyCompileReference,
  emptyCompileSourceBlobs,
  validateEmptyCompileSources,
} from "../native/sfc-parse-attributes-empty-component.ts";

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

await test("only the original empty compile packet advances by the authenticated build law", () => {
  const before = structuredClone(emptyCompileReference.before);
  const retained = structuredClone(before);
  assert.deepEqual(attributesPacketReference(before, "", "compile"), emptyCompileReference.after);
  assert.deepEqual(before, retained);
  for (const mode of ["omitted", "filename", "disabled", "enabled"])
    assert.deepEqual(attributesPacketReference(before, "", mode), before);
  for (const source of [" ", "<!--comment-->", "<style/>", "<docs/>", "<template/>", "<script/>"])
    assert.deepEqual(attributesPacketReference(before, source, "compile"), before);
  for (const field of ["code", "errors", "warnings", "styles", "moduleShape", "unknown"]) {
    const alteredBefore = JSON.parse(String(before.json)) as Record<string, unknown>;
    alteredBefore[field] = "unexpected";
    assert.throws(() =>
      attributesPacketReference({ ...before, json: JSON.stringify(alteredBefore) }, "", "compile"),
    );
    const alteredAfter = JSON.parse(String(emptyCompileReference.after.json)) as Record<
      string,
      unknown
    >;
    alteredAfter[field] = "unexpected";
    assert.throws(() =>
      assert.deepEqual(
        { kind: "return", json: JSON.stringify(alteredAfter) },
        attributesPacketReference(before, "", "compile"),
      ),
    );
  }
  assert.throws(() => attributesPacketReference({ ...before, unknown: true }, "", "compile"));
});

await test("empty compile references reject altered or incomplete complete product source laws", () => {
  const sources = Object.fromEntries(
    Object.keys(emptyCompileSourceBlobs).map((file) => [
      file,
      readFileSync(new URL(`../../${file}`, import.meta.url)),
    ]),
  );
  validateEmptyCompileSources(sources);
  for (const file of Object.keys(sources)) {
    assert.throws(() =>
      validateEmptyCompileSources({
        ...sources,
        [file]: Buffer.concat([sources[file], Buffer.from("\n")]),
      }),
    );
    const missing = { ...sources };
    delete missing[file];
    assert.throws(() => validateEmptyCompileSources(missing));
  }
  assert.throws(() => validateEmptyCompileSources({ ...sources, unknown: Buffer.from("") }));
});
