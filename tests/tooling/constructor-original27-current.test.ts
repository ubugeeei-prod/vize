import assert from "node:assert/strict";
import test from "node:test";
import { fileURLToPath } from "node:url";
import { asArray, asRecord, asString } from "../../tools/support/json-value.ts";
import { sha256 } from "../../tools/support/config/windows_consumer/common.ts";
import { captureDeclarationGraph } from "../../tools/support/release/computed_event_custody.ts";
import { reference } from "../../tools/support/release/constructor_original27_public/oracle.ts";
import {
  assertCurrentGraph,
  assertIndependentOracles,
  assertOriginalPopulation,
  assertSameGraph,
} from "../../tools/support/compat/content-mapper/constructor-original27-current.ts";

test("the current ten-file graph preserves complete source bodies, edges, order and module attributes", () => {
  const root = fileURLToPath(new URL("../../npm/cli", import.meta.url)),
    graph = captureDeclarationGraph(root);
  assertCurrentGraph(graph, root);
  assertSameGraph(graph, root, graph, root);
  const dropped = structuredClone(graph);
  dropped.files.pop();
  assert.throws(() => assertCurrentGraph(dropped, root));
  const duplicate = structuredClone(graph);
  duplicate.files.push(duplicate.files[0]);
  assert.throws(() => assertCurrentGraph(duplicate, root));
  const escaped = structuredClone(graph);
  escaped.edges[0].target = root + "/../outside.d.ts";
  assert.throws(() => assertCurrentGraph(escaped, root));
  const duplicateEdge = structuredClone(graph);
  duplicateEdge.edges[1] = duplicateEdge.edges[0];
  assert.throws(() => assertCurrentGraph(duplicateEdge, root));
  const reordered = structuredClone(graph);
  reordered.edges.reverse();
  assert.throws(() => assertSameGraph(graph, root, reordered, root));
  const changedBody = structuredClone(graph);
  changedBody.files[0].sha256 = "f".repeat(64);
  assert.throws(() => assertSameGraph(graph, root, changedBody, root));
  const changedMode = structuredClone(graph),
    imported = changedMode.edges.find((edge) => Object.keys(edge.attributes).length);
  assert.ok(imported, "the actual CJS-safe namespace type import must be represented");
  imported.attributes["resolution-mode"] =
    imported.attributes["resolution-mode"] === "import" ? "require" : "import";
  assert.throws(() => assertSameGraph(graph, root, changedMode, root));
});

test("population guards refuse missing, duplicate, reordered, misclassified or tampered original input inventories", () => {
  // Synthetic unit-only bodies exercise inventory guards; hosted loadOriginal27
  // independently authenticates all actual authored bytes against frozen a8.
  const frozen = reference(),
    authored = asArray(frozen.authoredCases).map((id) => asString(id));
  const rows = asArray(frozen.caseIds).map((id) => ({
    id: asString(id),
    component: "export {};\n",
    source: "<template/>\n",
    requiresAuthoredError: authored.includes(asString(id)),
  }));
  const files = new Map<string, string>();
  for (const row of rows) {
    files.set(row.id + "/component.ts", row.component);
    files.set(row.id + "/src/Tiny.vue", row.source);
  }
  for (const file of ["authored_h.ts", "tsconfig.json", "tsconfig.mapper.json", "tsconfig.h.json"])
    files.set(file, "{}\n");
  const manifest = [...files].map(([file, body]) => ({
    file,
    bytes: Buffer.byteLength(body),
    sha256: sha256(body),
  }));
  const input = { rows, files, manifest };
  assertOriginalPopulation(input);
  const missing = structuredClone(input);
  missing.files.delete("zero/component.ts");
  assert.throws(() => assertOriginalPopulation(missing));
  const duplicate = structuredClone(input);
  duplicate.rows[1].id = duplicate.rows[0].id;
  assert.throws(() => assertOriginalPopulation(duplicate));
  const reordered = structuredClone(input);
  reordered.rows.reverse();
  assert.throws(() => assertOriginalPopulation(reordered));
  const flags = structuredClone(input);
  flags.rows[0].requiresAuthoredError = !flags.rows[0].requiresAuthoredError;
  assert.throws(() => assertOriginalPopulation(flags));
  const body = structuredClone(input);
  body.files.set("zero/component.ts", "poison");
  assert.throws(() => assertOriginalPopulation(body));
  const hash = structuredClone(input);
  hash.manifest[0].sha256 = "a".repeat(64);
  assert.throws(() => assertOriginalPopulation(hash));
});

test("the full independent fourteen-row Vue and stock-h oracles stay exact without any current mapper count waiver", () => {
  const original = reference();
  assertIndependentOracles(original.stockH, original.official);
  for (const mutation of ["drop", "extra", "message", "continuation", "order", "status"]) {
    const official = structuredClone(asRecord(original.official)),
      rows = asArray(official.diagnostics);
    if (mutation === "drop") rows.pop();
    if (mutation === "extra") rows.push(structuredClone(rows[0]));
    if (mutation === "message") asRecord(rows[0]).message = "changed authored message";
    if (mutation === "continuation")
      asArray(asRecord(rows[0]).continuation).push("unexpected continuation");
    if (mutation === "order") rows.reverse();
    if (mutation === "status") official.status = 0;
    assert.throws(() => assertIndependentOracles(original.stockH, official));
  }
  const stock = structuredClone(asRecord(original.stockH));
  asRecord(asArray(stock.diagnostics)[0]).code = 9999;
  assert.throws(() => assertIndependentOracles(stock, original.official));
});
