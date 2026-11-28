import assert from "node:assert/strict";
import { test } from "vite-plus/test";
import { stripCatalogRegions } from "../../scripts/catalog-regions.ts";

test("catalog labels are removed without touching literals, legal comments or pure annotations", () => {
  for (const newline of ["\n", "\r\n", "\r"]) {
    const source = [
      "//#region src/catalog/a.ts",
      "const literal = `line\n//#region literal\n//#endregion\nend`;",
      "/*! @license retain */",
      "const value = /* @__PURE__ */ make(literal);",
      "//#endregion",
      "//#regional ordinary comment",
      "export { value };",
      "//#endregion",
    ].join(newline);
    const result = stripCatalogRegions(source, "catalog.mjs");
    const expected = [
      "const literal = `line\n//#region literal\n//#endregion\nend`;",
      "/*! @license retain */",
      "const value = /* @__PURE__ */ make(literal);",
      "//#regional ordinary comment",
      "export { value };",
      "",
    ].join(newline);
    assert.equal(result.code, expected);
    const map = JSON.parse(result.map) as {
      sources: string[];
      sourcesContent: string[];
      mappings: string;
    };
    assert.deepEqual(map.sources, ["catalog.mjs"]);
    assert.deepEqual(map.sourcesContent, [source]);
    assert.ok(map.mappings.length > 0);
  }
});
