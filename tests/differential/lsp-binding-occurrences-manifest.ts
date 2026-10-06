import assert from "node:assert/strict";
import { readPinnedArtifact } from "./harness.mjs";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export const BINDING_SESSION = "lsp/regression/binding-occurrences-original";
const position = (line: number) => ({ line, character: 6 });
const plans = [
  ...[1, 2, 3].map((line) => ({
    method: "textDocument/documentHighlight",
    params: { position: position(line) },
  })),
  ...[1, 2, 3].map((line) => ({
    method: "textDocument/references",
    params: { position: position(line), context: { includeDeclaration: true } },
  })),
  { method: "textDocument/codeLens", params: { position: { line: 0, character: 0 } } },
];
const counts = [2, 2, 4, 2, 2, 4, 3];

/** New whole vectors authored from the issue's projected locations; no old raw-RPC credit. */
export function loadBindingOccurrences(
  packRoot: string,
  data: FixtureData,
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.id, BINDING_SESSION);
  assert.equal(data.method, "textDocument/documentHighlight");
  assert.equal(data.provenance.fixCommit, "536f646ffb5773a944290351dd54827cbb1db13a");
  assert.deepEqual(data.provenance.witness, {
    path: "binding-occurrences-original/original-issue-body.md",
    sha256: "d7e5ca2ee61c895754f92af033393430087067d036debe60a6c82a6d553c47c7",
  });
  readPinnedArtifact(packRoot, data.provenance.witness);
  assert(
    Array.isArray(expected) && expected.length === 7,
    "all seven original request plans required",
  );
  return (expected as LspFixture["requests"]).map((request, index) => {
    assert.deepEqual(Object.keys(request).sort(), ["method", "params", "result"]);
    assert.equal(request.method, plans[index].method);
    assert.deepEqual(request.params, plans[index].params);
    assert(Array.isArray(request.result) && request.result.length === counts[index]);
    return request;
  });
}
