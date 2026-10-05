import assert from "node:assert/strict";
import type { FixtureData, LspFixture } from "./lsp-types.ts";

export function loadModuleResolutionLinks(
  data: FixtureData,
  expected: unknown,
): LspFixture["requests"] {
  assert.equal(data.id, "lsp/fix-history/document-link-module-resolution-original");
  assert.equal(data.method, "textDocument/documentLink");
  assert.equal(data.entry, "src/App.vue");
  assert.deepEqual(expected, [
    {
      range: { start: { line: 1, character: 19 }, end: { line: 1, character: 44 } },
      target: "${workspace}/src/pages/%5Bid%5D/Detail.vue",
    },
    {
      range: { start: { line: 2, character: 24 }, end: { line: 2, character: 31 } },
      target: "${workspace}/src/lib/index.ts",
    },
    {
      range: { start: { line: 3, character: 18 }, end: { line: 3, character: 43 } },
      target: "${workspace}/src/pages/%5Bid%5D/Detail.vue",
    },
  ]);
  return [{ params: {}, result: expected }];
}
