import assert from "node:assert/strict";

export type ExposurePacket = {
  code: string;
  map?: string | { mappings: string; [key: string]: unknown } | null;
  moduleShape?: {
    defaultExportStart?: number;
    defaultExportKeywordEnd?: number;
    defaultExportEnd: number;
    [key: string]: unknown;
  };
  [key: string]: unknown;
};

// Advance one preserved non-inline setup component read by the independently
// specified #10877 edit. The reference owns the complete packet: code and each
// affected byte offset are reconstructed, while maps and all other fields must
// remain byte-for-byte equal to the preserved producer.
export function setupComponentBracketReference(
  packet: ExposurePacket,
  component: string,
): ExposurePacket {
  assert.match(component, /^[$A-Z_a-z][$\w]*$/, "the setup component must be an identifier");
  const out = structuredClone(packet);
  const before = `$setup.${component}`;
  const start = packet.code.indexOf(before);
  if (start === -1) return out;
  assert.equal(
    packet.code.indexOf(before, start + before.length),
    -1,
    `the preserved packet must contain one ${before} read`,
  );
  const after = `$setup[${JSON.stringify(component)}]`;
  out.code = packet.code.slice(0, start) + after + packet.code.slice(start + before.length);
  const byteDelta = Buffer.byteLength(after) - Buffer.byteLength(before);
  for (const key of [
    "defaultExportStart",
    "defaultExportKeywordEnd",
    "defaultExportEnd",
  ] as const) {
    const offset = out.moduleShape?.[key];
    if (typeof offset === "number" && offset > start) out.moduleShape![key] = offset + byteDelta;
  }
  return out;
}

// Advance the preserved before-source packet by the independently specified
// #10891 edit. The candidate packet is never rewritten: every code byte, map
// segment, descriptor and diagnostic is still compared against this reference.
export function closedSetupReference(packet: ExposurePacket, source: string): ExposurePacket {
  const out = structuredClone(packet);
  if (
    !/<script\b[^>]*\bsetup\b/.test(source) ||
    !/(?:const __returned__ = |(?:^|\n)[ \t]*return \{\};?\n[ \t]*\})/.test(packet.code)
  ) {
    return out;
  }
  const signature = /^([ \t]*)(async )?setup\((__props(?:: any)?)(?:, \{ ([^}]+) \})?\) \{\n/m;
  const match = signature.exec(packet.code);
  assert.ok(match, "the preserved producer must have its known setup signature");
  if (match[4]?.includes("expose: __expose")) return out;
  const context = match[4] ? `, ${match[4]}` : "";
  const indent = packet.code.includes('\n  __name: "') ? match[1] + "  " : "  ";
  const replacement = `${match[1]}${match[2] ?? ""}setup(${match[3]}, { expose: __expose${context} }) {\n${indent}__expose();\n`;
  const callLine = packet.code.slice(0, match.index + match[0].length).split("\n").length - 1;
  out.code =
    packet.code.slice(0, match.index) +
    replacement +
    packet.code.slice(match.index + match[0].length);
  const byteDelta = Buffer.byteLength(out.code) - Buffer.byteLength(packet.code);
  if (out.map) {
    const encoded = typeof out.map === "string";
    const map = encoded ? JSON.parse(out.map as string) : out.map;
    const lines = map.mappings.split(";");
    // The inserted runtime glue has no authored mapping. Do not extend a map
    // whose final mapped source already precedes the insertion.
    if (callLine < lines.length) lines.splice(callLine, 0, "");
    map.mappings = lines.join(";");
    out.map = encoded ? JSON.stringify(map) : map;
  }
  if (out.moduleShape) out.moduleShape.defaultExportEnd += byteDelta;
  return out;
}

// Preserve pre-fix checksum evidence without replacing the recorded hash.
// Require the exact new prefix, then prove the edit is reversible byte for byte.
export function historicalSetupCode(code: string, source: string): string {
  const signature =
    /^([ \t]*)(async )?setup\((__props(?:: any)?), \{ expose: __expose(?:, ([^}]+))? \}\) \{\n[ \t]*__expose\(\);\n/m;
  const match = signature.exec(code);
  assert.ok(match, "default exposure must precede authored setup in the current module");
  const context = match[4] ? `, { ${match[4]} }` : "";
  const before =
    code.slice(0, match.index) +
    `${match[1]}${match[2] ?? ""}setup(${match[3]}${context}) {\n` +
    code.slice(match.index + match[0].length);
  assert.equal(
    closedSetupReference({ code: before }, source).code,
    code,
    "the historical edit must round-trip exactly",
  );
  return before;
}
