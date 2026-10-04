import assert from "node:assert/strict";
import { readPinnedArtifact } from "./harness.mjs";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

const highlight = "textDocument/documentHighlight";
type Plan = { method: string; line: number; character: number; count: number | null };
const plan = (line: number, character: number, count: number | null): Plan => ({
  method: highlight,
  line,
  character,
  count,
});
// Original five callbacks and every authored request, including empty hover.
const plans: Record<string, Plan[]> = {
  "lsp/fix-history/highlight-original-rw": [plan(1, 6, 3)],
  "lsp/fix-history/highlight-original-pair": [
    plan(6, 6, 2),
    plan(6, 33, 2),
    plan(5, 4, 2),
    plan(8, 4, 1),
  ],
  "lsp/fix-history/highlight-original-tag": [plan(8, 5, 2)],
  "lsp/fix-history/highlight-original-crlf": [plan(1, 6, 2)],
  "lsp/fix-history/highlight-original-empty": [
    plan(0, 0, null),
    { ...plan(0, 0, null), method: "textDocument/hover" },
  ],
};

type ExpectedRequest = {
  method: string;
  position: { line: number; character: number };
  result: unknown;
};

export function loadOriginalHighlightRequests(
  packRoot: string,
  data: FixtureData,
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.method, highlight);
  assert.equal(data.provenance.fixCommit, "ddef7d9268425346f5b01b3da5ed592eb544b6dc");
  assert.deepEqual(data.initializationOptions, { editor: true, lint: false, typecheck: false });
  const witness = data.provenance.witness;
  assert(witness, "complete original test source is required");
  assert.equal(witness.path, "highlight-original/witness.test.ts.txt");
  assert.equal(witness.sha256, "6ae6e35fc14315a77062fc2cebf3f95bb38f9021273e9c81c12cc7f57838e24f");
  readPinnedArtifact(packRoot, witness);
  assert(Object.hasOwn(plans, data.id), "unregistered original highlight session");
  assert(Array.isArray(expected));
  const original = plans[data.id];
  assert.equal(expected.length, original.length, "all original requests are required");
  return (expected as ExpectedRequest[]).map((request, index) => {
    const required = original[index];
    assert.deepEqual(Object.keys(request).sort(), ["method", "position", "result"]);
    assert.equal(request.method, required.method, "original request method is immutable");
    assert.deepEqual(request.position, { line: required.line, character: required.character });
    if (required.count === null) {
      assert.equal(request.result, null, "original empty response is null, never an array");
    } else {
      assert(Array.isArray(request.result));
      assert.equal(request.result.length, required.count, "complete original highlights required");
    }
    return {
      method: request.method,
      params: { position: request.position },
      result: request.result,
    };
  });
}
