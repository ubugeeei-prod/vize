import { test } from "node:test";
import { stripTypeScript } from "./strip-types.ts";

void test("stripTypeScript accepts successful transforms without errors", async (t) => {
  const result = await stripTypeScript("fixture.ts", "const count: number = 1;\n", false);

  t.assert.match(result.code, /const count = 1/);
  t.assert.equal(result.map, null);
});
