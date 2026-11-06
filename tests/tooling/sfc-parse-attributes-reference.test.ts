import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { test } from "node:test";
import {
  attributesPacketReference,
  exposureReferenceCustody,
  rawEmitterCompileReference,
  rawEmitterCompileReferences,
} from "../native/sfc-parse-attributes-reference.ts";
import {
  emptyCompileReference,
  emptyCompileSourceBlobs,
  validateEmptyCompileSources,
} from "../native/sfc-parse-attributes-empty-component.ts";
import {
  emptyCompileSourcesForReference,
  emptyProjectContextSupportingBlobs,
} from "../native/sfc-empty-project-context-source.ts";
import type { ExposurePacket } from "./support/script-setup-default-expose.ts";
import {
  moduleShapeFactsReference,
  type ParseModule,
} from "./support/typescript-erasure-reference.ts";

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
const fromTests = createRequire(new URL("../package.json", import.meta.url));
const { parse } = fromTests("@babel/parser") as { parse: ParseModule };

function withModuleFacts(value: Record<string, unknown>): Record<string, unknown> {
  return moduleShapeFactsReference(value as { code: string; [key: string]: unknown }, parse);
}

test("parsed-attribute compile packets advance only the reviewed exposure reference", () => {
  assert.equal(rows.length, 6);
  assert.equal(exposureReferenceCustody.fixSource, "9157b9fa83dbdc98100be084be288f21d0d1bc18");
  for (const row of rows) {
    const before = { kind: "return", json: JSON.stringify(row.before) };
    const current = { kind: "return", json: JSON.stringify(row.after) };
    const retained = structuredClone(before);
    const actual = attributesPacketReference(before, row.source, "compile", current, parse);
    assert.equal(actual.kind, "return");
    assert.deepEqual(JSON.parse(String(actual.json)), withModuleFacts(row.after));
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
      { kind: "return", json: JSON.stringify(row.after) },
      parse,
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

test("only the exact formatted compile packets advance to raw emitter references", () => {
  assert.equal(
    exposureReferenceCustody.rawEmitterSource,
    "8a90268d4cd202e577406cf152b173524f8fa064",
  );
  assert.equal(exposureReferenceCustody.rawEmitterEvidenceRun, 42168437970);
  assert.equal(rawEmitterCompileReferences.length, 2);
  for (const reference of rawEmitterCompileReferences) {
    const retained = structuredClone(reference.before);
    assert.deepEqual(
      rawEmitterCompileReference(reference.before, reference.source),
      reference.after,
    );
    assert.deepEqual(reference.before, retained);
    assert.deepEqual(
      rawEmitterCompileReference(reference.before, `${reference.source} `),
      reference.before,
    );
    for (const field of ["code", "map", "moduleShape", "errors", "macroArtifacts"]) {
      const changed = structuredClone(reference.before) as ExposurePacket;
      changed[field] = field === "code" ? `${String(changed[field])}\nvoid 1;` : "unexpected";
      assert.throws(() => rawEmitterCompileReference(changed, reference.source));
    }
    const changedAfter = structuredClone(reference.after) as ExposurePacket;
    changedAfter.code = `${String(changedAfter.code)}\nvoid 1;`;
    assert.notDeepEqual(
      changedAfter,
      rawEmitterCompileReference(reference.before, reference.source),
    );
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
    assert.deepEqual(
      JSON.parse(String(attributesPacketReference(before, source, "compile").json)),
      withModuleFacts(JSON.parse(String(before.json)) as Record<string, unknown>),
    );
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
  const sources = emptyCompileSourcesForReference(
    Object.fromEntries(
      Object.keys(emptyCompileSourceBlobs).map((file) => [
        file,
        readFileSync(new URL(`../../${file}`, import.meta.url)),
      ]),
    ),
    emptyCompileSourceBlobs,
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

test("the project-context successor rejects unknown, incomplete and mixed source bundles", () => {
  const read = (blobs: Readonly<Record<string, string>>) =>
    Object.fromEntries(
      Object.keys(blobs).map((file) => [
        file,
        readFileSync(new URL(`../../${file}`, import.meta.url)),
      ]),
    );
  const sources = read(emptyCompileSourceBlobs);
  const supporting = read(emptyProjectContextSupportingBlobs);
  const original = emptyCompileSourcesForReference(sources, emptyCompileSourceBlobs, supporting);
  validateEmptyCompileSources(original);
  assert.equal(emptyCompileSourcesForReference(original, emptyCompileSourceBlobs), original);
  for (const [group, input] of [
    ["sources", sources],
    ["supporting", supporting],
  ] as const) {
    const validate = (changed: Record<string, Uint8Array>) =>
      emptyCompileSourcesForReference(
        group === "sources" ? changed : sources,
        emptyCompileSourceBlobs,
        group === "supporting" ? changed : supporting,
      );
    for (const file of Object.keys(input)) {
      assert.throws(() =>
        validate({ ...input, [file]: Buffer.concat([input[file], Buffer.from("\n")]) }),
      );
      const missing = { ...input };
      delete missing[file];
      assert.throws(() => validate(missing));
    }
    assert.throws(() => validate({ ...input, unknown: Buffer.from("") }));
  }
  const mixed = { ...sources, [Object.keys(original)[0]]: Object.values(original)[0] };
  assert.throws(() => emptyCompileSourcesForReference(mixed, emptyCompileSourceBlobs, supporting));
});
