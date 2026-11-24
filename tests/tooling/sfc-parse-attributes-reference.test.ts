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
  templateMapReference,
  withoutTemplateSegments,
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

// The mappings of four attribute fixtures before the render mapped back
// to the template, and after. `null` is a packet without a map.
const templateMapRows: [source: string, before: string | null, after: string][] = [
  ['<template lang="html"><p>🐱</p></template>', null, ";;AAAsB;UAAC,4BAAE"],
  [
    "<script vapor>export default {}</script><template><div/></template>",
    ";;;;;;AAAc,kBAAe,CAAC",
    ";;AAAkD;UAAC;;;AAArC,kBAAe,CAAC",
  ],
  [
    '<script setup vapor lang="ts">const n: number = 1;</script><template>{{n}}</template>',
    ";;;;;;;AAA8B,MAAM,IAAY",
    ";;;AAAqE;2BAAE;;;AAAzC,MAAM,IAAY",
  ],
  [
    '<!-- <script vapor>ignored</script> --><script setup>const text = "vapor";</script><template>{{text}}</template>',
    ";;;;;;;AAAqD,MAAM,KAAK,EAAE,CAAC,KAAK,CAAC",
    ";;;AAA6F;2BAAE;;;AAA1C,MAAM,KAAK,EAAE,CAAC,KAAK,CAAC",
  ],
];

test("template provenance preserves repeated anchors and validates the authored block and line", () => {
  const source =
    "<script>const text = '<template>decoy</template>'</script>\n<template>\n{{n}}\n</template>";
  const packet = (text: string, mappings: string) => ({
    code: "",
    map: JSON.stringify({ sources: ["Case.vue"], sourcesContent: [text], mappings }),
  });
  const before = packet(source, "AAAA,AAAA");
  const after = packet(source, "AAAA,AAAA,CAEA");
  assert.deepEqual(templateMapReference(before, after, source), after);
  assert.deepEqual(withoutTemplateSegments(after, source), before);
  assert.throws(() => templateMapReference(before, packet(source, "AAAA,CAEA"), source));
  const noMap = { code: "" };
  // Column 32 is in the script string. Line 0 of the second fixture has eight columns.
  assert.throws(() => templateMapReference(noMap, packet(source, "AAAgC"), source));
  const shortLine = "<!-- -->\n<template>{{n}}</template>";
  const beyondLine = packet(shortLine, "AAAqB");
  assert.throws(() => templateMapReference(noMap, beyondLine, shortLine));
  assert.deepEqual(withoutTemplateSegments(beyondLine, shortLine), beyondLine);
});

test("a compile packet's map gains only the segments authored in its template", () => {
  for (const [source, before, after] of templateMapRows) {
    const packet = (mappings: string | null, file = "Case.vue"): ExposurePacket => ({
      code: "export default {}",
      macroArtifacts: [],
      ...(mappings === null
        ? {}
        : {
            map: JSON.stringify({
              file,
              mappings,
              names: [],
              sources: [file],
              sourcesContent: [source],
              version: 3,
            }),
          }),
      templateHash: "retained",
    });
    const preserved = packet(before);
    const retained = structuredClone(preserved);
    // Byte for byte: a gained map sits where the candidate packet lists it.
    assert.equal(
      JSON.stringify(templateMapReference(preserved, packet(after), source)),
      JSON.stringify(packet(after)),
    );
    assert.deepEqual(preserved, retained);
    // The view the earlier references read is the packet as it was.
    assert.equal(
      JSON.stringify(withoutTemplateSegments(packet(after), source)),
      JSON.stringify(preserved),
    );
    assert.equal(templateMapReference(preserved, preserved, source), preserved);
    assert.equal(templateMapReference(preserved, undefined, source), preserved);
    // A segment authored outside the template is not gained: the first byte
    // of the file, or the script's own segments on a packet that had no map.
    assert.throws(() => templateMapReference(packet(null), packet(";;AAAA"), source));
    if (before !== null) {
      assert.throws(() => templateMapReference(packet(null), packet(after), source));
      // Every preserved segment and every other map field is kept.
      assert.throws(() => templateMapReference(preserved, packet(";;AAAA"), source));
      assert.throws(() => templateMapReference(preserved, packet(after, "Other.vue"), source));
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
    assert.deepEqual(
      JSON.parse(
        String(attributesPacketReference(before, source, "compile", undefined, parse).json),
      ),
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
