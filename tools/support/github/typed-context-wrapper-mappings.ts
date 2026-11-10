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
  projection: "wrapper-successor" | "component-hover-gaps-12461" = "wrapper-successor",
): void {
  const digest = (bytes: string | Uint8Array) => createHash("sha256").update(bytes).digest("hex");
  const producerText = Buffer.from(producer).toString("utf8");
  const componentExportAdditions = [
    "mod component_export;\n",
    "    component_export::insert_spans(&mut spans, mappings, metadata);\n",
  ];
  assert.equal(
    digest(producer),
    "271dfc983bf196cd118b521e7ac401824b19c423d2cacfa32dcfec29bdec922b",
    "unqualified content-mapper producer",
  );
  const componentHoverOwnerSuccessor =
    "    span_features::retain_component_prop_hover_owner(generated, &mut spans);\n";
  assert.equal(
    producerText.split(componentHoverOwnerSuccessor).length,
    2,
    "finite component-hover owner producer successor",
  );
  const componentHoverOwnerPredecessor = producerText.replace(componentHoverOwnerSuccessor, "");
  assert.equal(
    digest(componentHoverOwnerPredecessor),
    "9c3faec15ca7c73c4a8a3f0fd1fcae7bb6f9d70c8e71a7046994d373e8b4e038",
    "finite component-hover owner producer successor",
  );
  const hoverOwnerSuccessor =
    "    span_features::retain_template_hover_owner(generated, &mut spans);\n";
  assert.equal(
    componentHoverOwnerPredecessor.split(hoverOwnerSuccessor).length,
    2,
    "finite template-hover owner producer successor",
  );
  const hoverOwnerPredecessor = componentHoverOwnerPredecessor.replace(hoverOwnerSuccessor, "");
  assert.equal(
    digest(hoverOwnerPredecessor),
    "6a060e4d0c5d16cd3071f76ca35558abcecde9fcd5458346d54287eacdef03fd",
    "finite template-hover owner producer successor",
  );
  const templateUseProjectionSuccessor = "            template_use_navigation: true,\n";
  assert.equal(
    hoverOwnerPredecessor.split(templateUseProjectionSuccessor).length,
    2,
    "finite template-use projection producer successor",
  );
  const projectionPredecessor = hoverOwnerPredecessor.replace(templateUseProjectionSuccessor, "");
  assert.equal(
    digest(projectionPredecessor),
    "d51c0acbed93e899ec5b634825bd3d7d2ffb2d47908f5bbc575e68feb909a937",
    "finite template-use projection producer successor",
  );
  const localOwnershipSuccessor = "            local_component_ownership_helpers: false,\n";
  assert.equal(
    projectionPredecessor.split(localOwnershipSuccessor).length,
    2,
    "finite local-ownership producer successor",
  );
  let historicalProducer = projectionPredecessor.replace(localOwnershipSuccessor, "");
  assert.equal(
    digest(historicalProducer),
    "f75096741294e3f99e5fedf6accad2daf49ee402c72af2bc684b20b0f4310e20",
    "finite local-ownership producer successor",
  );
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
  const historicalRows = {
    "wrapper-successor": [-2754, -2752, -2743, -951, -939, -937, -928, -918, -59, 0],
    // Identifier-safe public props re-anchor their three symbol rows by six
    // bytes. Setup dedent keeps the next five authored spans on their original
    // identifiers rather than moving them into the removed indentation.
    "component-hover-gaps-12461": [-2748, -2746, -2737, -949, -937, -935, -926, -916, -59, 0],
  } as const;
  const fields = [
    [2, 37, 2, 0, 1048575],
    [9, 39, 9, 0, 1048575],
    [10, 48, 10, 0, 1048575],
    [12, 25, 12, 0, 1048575],
    [2, 37, 2, 0, 1048575],
    [9, 39, 9, 0, 1048575],
    [10, 48, 10, 0, 1048575],
    [4, 58, 4, 0, 1048575],
    [9, 39, 9, 0, 1048575],
    [9, 89, 9, 0, 1048575],
  ] as const;
  const rows = historicalRows[projection].map((relative, index) => [
    start + relative,
    ...fields[index],
  ]);
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
