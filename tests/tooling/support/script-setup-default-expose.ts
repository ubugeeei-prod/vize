import assert from "node:assert/strict";

export type ExposurePacket = {
  code: string;
  map?: string | null;
  moduleShape?: { defaultExportEnd: number; [key: string]: unknown };
  [key: string]: unknown;
};

// Advance the preserved before-source packet by the independently specified
// #10891 edit. The candidate packet is never rewritten: every code byte, map
// segment, descriptor and diagnostic is still compared against this reference.
export function closedSetupReference(packet: ExposurePacket, source: string): ExposurePacket {
  const out = structuredClone(packet);
  if (!source.startsWith("<script setup") || !packet.code.includes("const __returned__ = ")) {
    return out;
  }
  const signature = /^([ \t]*)(async )?setup\(__props(?:, \{ ([^}]+) \})?\) \{\n/m;
  const match = signature.exec(packet.code);
  assert.ok(match, "the preserved producer must have its known setup signature");
  if (match[3]?.includes("expose: __expose")) return out;
  const context = match[3] ? `, ${match[3]}` : "";
  const indent = source.startsWith('<script setup lang="ts">') ? match[1] + "  " : "  ";
  const replacement = `${match[1]}${match[2] ?? ""}setup(__props, { expose: __expose${context} }) {\n${indent}__expose();\n`;
  const callLine = packet.code.slice(0, match.index + match[0].length).split("\n").length - 1;
  out.code =
    packet.code.slice(0, match.index) +
    replacement +
    packet.code.slice(match.index + match[0].length);
  const byteDelta = Buffer.byteLength(out.code) - Buffer.byteLength(packet.code);
  if (out.map) {
    const map = JSON.parse(out.map);
    const lines = map.mappings.split(";");
    // The inserted runtime glue has no authored mapping. Do not extend a map
    // whose final mapped source already precedes the insertion.
    if (callLine < lines.length) lines.splice(callLine, 0, "");
    map.mappings = lines.join(";");
    out.map = JSON.stringify(map);
  }
  if (out.moduleShape) out.moduleShape.defaultExportEnd += byteDelta;
  return out;
}
