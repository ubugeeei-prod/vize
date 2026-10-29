import assert from "node:assert/strict";
import { closedSetupReference, type ExposurePacket } from "./script-setup-default-expose.ts";

type ComponentModuleShape = NonNullable<ExposurePacket["moduleShape"]> & {
  defaultExportStart?: number;
  defaultExportKeywordEnd?: number;
};

type RuntimeOptionRow = {
  target: string;
  options: { inlineTemplate?: unknown };
  single?: ExposurePacket[];
  batch?: { results: ExposurePacket[] };
  stats?: { outputBytes: number };
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
  const moduleShape = out.moduleShape as ComponentModuleShape | undefined;
  for (const key of [
    "defaultExportStart",
    "defaultExportKeywordEnd",
    "defaultExportEnd",
  ] as const) {
    const offset = moduleShape?.[key];
    if (typeof offset === "number" && offset > start) moduleShape[key] = offset + byteDelta;
  }
  return out;
}

export function advanceRuntimeOptionReference<T extends RuntimeOptionRow>(
  row: T,
  sourceFiles: readonly string[],
): T {
  const out = structuredClone(row);
  if (out.single)
    out.single = out.single.map((packet, index) =>
      closedSetupReference(packet, sourceFiles[index]),
    );
  if (out.batch)
    out.batch.results = out.batch.results.map((packet, index) =>
      closedSetupReference(packet, sourceFiles[index]),
    );
  if (row.target === "dom" && row.options.inlineTemplate === undefined && out.single && out.batch) {
    out.single[7] = setupComponentBracketReference(out.single[7], "Lazy");
    out.batch.results[7] = setupComponentBracketReference(out.batch.results[7], "Lazy");
  }
  const beforePackets = row.single;
  if (out.stats && out.single && beforePackets) {
    out.stats.outputBytes += out.single.reduce(
      (bytes, packet, index) =>
        bytes + Buffer.byteLength(packet.code) - Buffer.byteLength(beforePackets[index].code),
      0,
    );
  }
  return out;
}
