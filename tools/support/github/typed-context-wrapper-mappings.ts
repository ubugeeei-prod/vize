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
  const producerText = Buffer.from(producer).toString("utf8");
  const componentExportAdditions = [
    "mod component_export;\n",
    "    component_export::insert_spans(&mut spans, mappings, metadata);\n",
  ];
  assert.equal(
    digest(producer),
    "f75096741294e3f99e5fedf6accad2daf49ee402c72af2bc684b20b0f4310e20",
    "unqualified content-mapper producer",
  );
  let historicalProducer = producerText;
  for (const addition of componentExportAdditions) {
    assert.equal(
      historicalProducer.split(addition).length,
      2,
      "finite component-export producer successor",
    );
    historicalProducer = historicalProducer.replace(addition, "");
  }
  assert.equal(
    digest(historicalProducer),
    "25abafce233364f0b11f71202d32f87290ef7d231f939063938f8649e3448b67",
    "finite component-export producer successor",
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
  // The integrated component-export producer also owns the generated default
  // component name without claiming an authored source range.
  const componentDeclaration = Buffer.from("declare const Route: typeof __vize_component__;");
  const declaration = generated.indexOf(componentDeclaration);
  assert(
    declaration >= 0 && generated.indexOf(componentDeclaration, declaration + 1) < 0,
    "unique generated component declaration",
  );
  const componentExport = [
    declaration + Buffer.byteLength("declare const "),
    Buffer.byteLength("Route"),
    0,
    0,
    1,
    248,
  ];
  // Keep actual packets untouched; only the finite expected successors change.
  const expected = [
    ...projected.slice(0, 9),
    atom(start - 1),
    projected[9],
    atom(start + 9),
    componentExport,
  ];
  assert.deepEqual(current.mappings, expected, "whole ordered current mapping successor");
}
