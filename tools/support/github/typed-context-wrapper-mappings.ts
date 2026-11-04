import assert from "node:assert/strict";
import { createHash } from "node:crypto";

export type MapperPacket = {
  text: string;
  mappings: number[][];
  [key: string]: unknown;
};

export function assertTypedContextWrapperMappings(
  source: string,
  before: MapperPacket,
  current: MapperPacket,
  projected: number[][],
  producer: Uint8Array,
): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  assert.equal(
    digest(producer),
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "unqualified content-mapper producer",
  );
  assert.equal(
    digest(source),
    "6a67ce805c62e2af0b6f68bd98165e6d040debc32bee8bca9ba6e41909cff13b",
    "unqualified typed-context source",
  );
  assert.deepEqual(Object.keys(before).sort(), ["diagnostics", "extension", "mappings", "text"]);
  assert.deepEqual(Object.keys(current).sort(), [
    "diagnosticDirectives",
    "diagnostics",
    "extension",
    "mappings",
    "text",
  ]);
  const generated = Buffer.from(current.text);
  const needle = Buffer.from("void (undefined); // Interpolation");
  const statement = generated.indexOf(needle);
  assert(statement >= 0 && generated.indexOf(needle, statement + 1) < 0, "unique interpolation");
  const start = statement + Buffer.byteLength("void (");
  // The preceding whole-text transition has already relocated every old row.
  // A helper prefix change therefore moves this anchor and all ten old rows
  // together; no fixed generated-coordinate or prefix-length adjustment is used.
  const rows = [
    [-2754, 2, 37, 2, 0, 1048575],
    [-2752, 9, 39, 9, 0, 1048575],
    [-2743, 10, 48, 10, 0, 1048575],
    [-951, 12, 25, 12, 0, 1048575],
    [-939, 2, 37, 2, 0, 1048575],
    [-937, 9, 39, 9, 0, 1048575],
    [-928, 10, 48, 10, 0, 1048575],
    [-918, 4, 58, 4, 0, 1048575],
    [-59, 9, 39, 9, 0, 1048575],
    [0, 9, 89, 9, 0, 1048575],
  ].map(([relative, ...fields]) => [start + relative, ...fields]);
  assert.deepEqual(projected, rows, "all projected historical tuples must remain exact");
  assert.equal(Buffer.from(source).subarray(89, 98).toString(), "undefined");
  const atom = (at: number) => [at, 1, 89, 9, 1, 0];
  // #11675 emits the two unowned parentheses as diagnostic-only Atom rows.
  // Keep actual packets untouched; only the finite expected successor changes.
  const expected = [...projected.slice(0, 9), atom(start - 1), projected[9], atom(start + 9)];
  assert.deepEqual(current.mappings, expected, "whole ordered current mapping successor");
}
