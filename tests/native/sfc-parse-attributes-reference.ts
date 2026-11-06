import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { readFileSync } from "node:fs";
import {
  closedSetupReference,
  type ExposurePacket,
} from "../tooling/support/script-setup-default-expose.ts";
import {
  isTypeScriptSource,
  moduleShapeFactsReference,
  type ParseModule,
  typescriptErasureReference,
} from "../tooling/support/typescript-erasure-reference.ts";
import { emptyCompilePacketReference } from "./sfc-parse-attributes-empty-component.ts";

export const exposureReferenceCustody = {
  fixSource: "9157b9fa83dbdc98100be084be288f21d0d1bc18",
  referenceBlob: "9de66fef47542ca09981eea7f7fda685d5932f85",
  rawEmitterSource: "8a90268d4cd202e577406cf152b173524f8fa064",
  rawEmitterEvidenceRun: 42168437970,
  rawBasePacketsRetained: true,
  candidatePacketsUnmodified: true,
} as const;

const referenceBytes = readFileSync(
  new URL("../tooling/support/script-setup-default-expose.ts", import.meta.url),
);
const referenceHash = createHash("sha1")
  .update(Buffer.from(`blob ${referenceBytes.length}\0`))
  .update(referenceBytes)
  .digest("hex");
assert.equal(referenceHash, exposureReferenceCustody.referenceBlob);

const digest = (value: string): string => createHash("sha256").update(value).digest("hex");

function formattedCode(name: string): string {
  return [
    'import { ssrInterpolate as _ssrInterpolate } from "@vue/server-renderer";',
    "function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {",
    "  _push(`${_ssrInterpolate($setup.n)}`);",
    "}",
    "const n = 1;",
    "export default {",
    `  __name: "${name}",`,
    "  ssrRender,",
    "  __vapor: true,",
    "  setup(__props, { expose: __expose }) {",
    "    __expose();",
    "    const __returned__ = { n };",
    '    Object.defineProperty(__returned__, "__isScriptSetup", {',
    "      enumerable: false,",
    "      value: true",
    "    });",
    "    return __returned__;",
    "  }",
    "};",
  ].join("\n");
}

function rawCode(name: string): string {
  return [
    'import { ssrInterpolate as _ssrInterpolate } from "@vue/server-renderer"',
    "",
    "",
    "function ssrRender(_ctx, _push, _parent, _attrs, $props, $setup, $data, $options) {",
    "  _push(`${_ssrInterpolate($setup.n)}`)",
    "}",
    "",
    "const n = 1;",
    "",
    "export default {",
    `  __name: '${name}',`,
    "  ssrRender: ssrRender,",
    "  __vapor: true,",
    "  setup(__props, { expose: __expose }) {",
    "  __expose();",
    "",
    "",
    "const __returned__ = { n }",
    "Object.defineProperty(__returned__, '__isScriptSetup', { enumerable: false, value: true })",
    "return __returned__",
    "}",
    "",
    "}",
  ].join("\n");
}

function compileValue(
  code: string,
  file: string,
  source: string,
  mappings: string,
  defaultExportStart: number,
  defaultExportKeywordEnd: number,
  defaultExportEnd: number,
): ExposurePacket {
  return {
    code,
    errors: [],
    warnings: [],
    hasScoped: false,
    styles: [],
    customBlocks: [],
    macroArtifacts: [],
    map: JSON.stringify({
      file,
      mappings,
      names: [],
      sources: [file],
      sourcesContent: [source],
      version: 3,
    }),
    templateHash: "91da54536279c8ca",
    scriptHash: "a447290fed98e441",
    moduleShape: {
      hasDefaultExport: true,
      hasSfcMainDefined: false,
      hasNamedRenderExport: false,
      hasNamedSsrRenderExport: false,
      defaultExportIsSfcMain: false,
      defaultExportStart,
      defaultExportKeywordEnd,
      defaultExportEnd,
    },
  };
}

function rawEmitterReference(
  name: string,
  source: string,
  mapping: string,
  formattedEnd: number,
  rawEnd: number,
  beforeJsonHash: string,
  afterJsonHash: string,
) {
  const file = `parsed-attributes/${name}.vue`;
  const before = compileValue(
    formattedCode(name),
    file,
    source,
    `;;;;${mapping}`,
    214,
    228,
    formattedEnd,
  );
  const after = compileValue(rawCode(name), file, source, `;;;;;;;${mapping}`, 216, 230, rawEnd);
  assert.equal(digest(JSON.stringify(before)), beforeJsonHash, `${name} formatted reference`);
  assert.equal(digest(JSON.stringify(after)), afterJsonHash, `${name} raw emitter reference`);
  return { source, before, after, beforeJsonHash, afterJsonHash } as const;
}

export const rawEmitterCompileReferences = [
  rawEmitterReference(
    "setup",
    '<script setup vapor lang="ts">const n: number = 1;</script><template>{{n}}</template>',
    "AAA8B,MAAM,IAAY",
    512,
    492,
    "0a4e497cc68fca2ae4f943acb725f5ac7459f1a96cf5110d2aaabc7631730ba6",
    "3b1ff26ff4e6fa21a77bcc139117956622513387fcbc0663018c5063af0625a1",
  ),
  rawEmitterReference(
    "duplicates",
    '<script setup vapor="first" lang="ts" vapor="last">const n: number = 1;</script><template>{{n}}</template>',
    "AAAmD,MAAM,IAAY",
    517,
    497,
    "210e6ddcd7ee0c9c02f7050ccbf4e006c046ac9ccd56bdd02ff4766a34232470",
    "829b65987b14f954d90009f537b74e851cc05d9fe6ff4eb8f0b476d90c73a125",
  ),
] as const;

const rawEmitterReferencesBySource = new Map(
  rawEmitterCompileReferences.map((reference) => [reference.source, reference]),
);

export function rawEmitterCompileReference(packet: ExposurePacket, source: string): ExposurePacket {
  const reference = rawEmitterReferencesBySource.get(source);
  if (!reference) return structuredClone(packet);
  assert.equal(
    digest(JSON.stringify(packet)),
    reference.beforeJsonHash,
    "whole formatted compile reference",
  );
  const after = structuredClone(reference.after);
  assert.equal(
    digest(JSON.stringify(after)),
    reference.afterJsonHash,
    "whole raw emitter successor",
  );
  return after;
}

export function attributesPacketReference(
  packet: Record<string, unknown>,
  source: string,
  mode: string,
  current?: Record<string, unknown>,
  parse?: ParseModule,
): Record<string, unknown> {
  if (mode !== "compile" || packet.kind !== "return") return structuredClone(packet);
  if (source === "") return emptyCompilePacketReference(packet);
  assert.equal(typeof packet.json, "string");
  const value: unknown = JSON.parse(String(packet.json));
  assert(value !== null && typeof value === "object" && !Array.isArray(value));
  assert("code" in value && typeof value.code === "string");
  const exposure = closedSetupReference(value as ExposurePacket, source);
  let reference = rawEmitterCompileReference(exposure, source);
  if (reference.moduleShape) {
    assert.ok(parse, "module facts require the independent module parser");
    if (isTypeScriptSource(source)) {
      assert(current);
      assert.equal(current.kind, "return");
      assert(typeof current.json === "string");
      const currentValue: unknown = JSON.parse(current.json);
      assert(
        currentValue !== null && typeof currentValue === "object" && !Array.isArray(currentValue),
      );
      reference = typescriptErasureReference(reference, currentValue as ExposurePacket, parse);
    } else {
      reference = moduleShapeFactsReference(reference, parse);
    }
  }
  return { ...packet, json: JSON.stringify(reference) };
}
